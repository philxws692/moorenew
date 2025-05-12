mod args;
mod utils;

use crate::args::Action;
use crate::utils::config::generate_config;
use crate::utils::fileext::FileExt;
use crate::utils::logging;
use crate::utils::ssh::SSHClient;
use crate::utils::sshkeygen;
use crate::utils::sysinfo;
use args::MooRenewArgs;
use clap::Parser;
use std::env;
use std::fs::File;
use std::path::Path;
use std::process::exit;
use tracing::{error, info};
use crate::utils::certificates::download_certificates;

fn main() {
    if !Path::new(".env.moorenew").exists() {
        println!("assuming first run due to missing config");
        println!("generating default config in .env.moorenew");
        generate_config();
    }
    dotenv::from_filename(".env.moorenew").ok();
    logging::setup_logging();

    let args = MooRenewArgs::parse();

    match args.action {
        Some(Action::Keygen(keygen)) => {
            // Checking for algorithm
            let mut algo = "ed25519".to_string();
            if let Some(algorithm) = keygen.algorithm {
                if algorithm == "rsa4096" {
                    algo = algorithm;
                } else if algorithm != "ed25519" {
                    error!("{} is not a valid algorithm", algorithm);
                    exit(1);
                }
            }
            info!("using {} for key generation", algo);

            // Getting filename
            let mut keyfilename = "moorenew".to_string();
            if let Some(keyname) = keygen.filename {
                keyfilename = keyname;
            }
            info!("using {} for key filename", keyfilename);

            // Getting comment
            let mut comment = format!(
                "{}@{} (moorenew service)",
                sysinfo::get_loggedin_user(),
                sysinfo::get_hostname()
            );
            if let Some(comm) = keygen.comment {
                comment = comm;
            }
            sshkeygen::generate_rsa_keypair(&algo, &keyfilename, &comment)
        }
        Some(Action::Run(run)) => {
            if run.dry_run == false {
                info!("starting update process...");
                update_certificates(false);
            } else {
                info!("starting dry run...");
                update_certificates(true);
            }
        }
        _ => {}
    }
}

fn update_certificates(dry_run: bool) {
    let username = &env::var("SFTP_USERNAME").unwrap()[..];
    let host = &env::var("SFTP_HOST").unwrap()[..];
    let private_key_path = &env::var("PRIVATE_KEY_PATH").unwrap()[..];
    let public_key_path = &env::var("PUBLIC_KEY_PATH").unwrap()[..];

    // Download certificates

    let client = SSHClient::connect(username, host, private_key_path, public_key_path).unwrap();

    download_certificates(&client, dry_run);
    
    

    client.disconnect()
}
