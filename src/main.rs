mod utils;
mod args;

use crate::utils::logging;
use crate::utils::sftp::SftpClient;
use crate::utils::sshkeygen;
use crate::utils::sysinfo;
use crate::utils::config;
use dotenv::dotenv;
use std::{env, fs};
use std::path::Path;
use std::process::exit;
use args::MooRenewArgs;
use clap::Parser;
use tracing::{error, info};
use crate::args::{Action, GenerateKeyCommand};
use crate::utils::config::generate_config;

fn main() {
    logging::setup_logging();
    if !Path::new(".env.moorenew").exists() {
        info!("assuming first run due to missing config");
        info!("generating default config in .env.moorenew");
        generate_config();
    }
    dotenv::from_filename(".env.moorenew.moorenew").ok();

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
            let mut comment = format!("{}@{} (moorenew service)", sysinfo::get_loggedin_user(), sysinfo::get_hostname());
            if let Some(comm) = keygen.comment {
                comment = comm;
            }
            sshkeygen::generate_rsa_keypair(&algo, &keyfilename, &comment)

        }
        Some(Action::Run(run)) => {
            if run.dry_run == false {
                info!("starting update process...");
                update_certificates()
            } else {
                info!("starting dry run...");
            }
        }
        _ => {}
    }

}

fn update_certificates() {
    let username = &env::var("SFTP_USERNAME").unwrap()[..];
    let host = &env::var("SFTP_HOST").unwrap()[..];
    let private_key_path = &env::var("PRIVATE_KEY_PATH").unwrap()[..];
    let public_key_path = &env::var("PUBLIC_KEY_PATH").unwrap()[..];

    let client = SftpClient::connect(username, host, private_key_path, public_key_path).unwrap();
}
