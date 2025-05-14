mod args;
mod utils;
mod system;

use crate::args::Action;
use crate::utils::certificates::download_certificates;
use crate::utils::config::generate_config;
use crate::utils::logging;
use crate::utils::ssh::SSHClient;
use crate::utils::sshkeygen;
use crate::utils::sysinfo;
use args::MooRenewArgs;
use clap::Parser;
use std::env;
use std::io::Error;
use std::path::Path;
use std::process::exit;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, instrument};
use crate::system::serviceproviders::ServiceProvider;

#[tokio::main]
async fn main() {
    if !Path::new(".env.moorenew").exists() {
        println!("assuming first run due to missing config");
        println!("generating default config in .env.moorenew");
        generate_config();
    }
    if !Path::new("moorenew.timer").exists() {
        println!("assuming first run due to missing timer");
        println!("generating default timer in moorenew.timer");
        match system::services::create_service_files("moorenew", ServiceProvider::SYSTEMD) {
            Ok(_) => {
                println!("successfully created service files");
            }
            Err(_) => {
                eprintln!("failed to create service files");
            }
        }
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
    
    sleep(Duration::from_millis(10)).await;
}

#[instrument(fields(result))]
fn update_certificates(dry_run: bool) {
    let username = &env::var("SFTP_USERNAME").unwrap()[..];
    let host = &env::var("SFTP_HOST").unwrap()[..];
    let private_key_path = &env::var("PRIVATE_KEY_PATH").unwrap()[..];
    let public_key_path = &env::var("PUBLIC_KEY_PATH").unwrap()[..];

    // Download certificates

    let client = SSHClient::connect(username, host, private_key_path, public_key_path).unwrap();

    download_certificates(&client, dry_run);

    let mut err_count = 0;

    match client.execute_command("docker restart $(docker ps -qaf name=postfix-mailcow)") {
        Ok(_) => {
            info!("successfully restarted postfix");
        }
        Err(e) => {
            error!("failed to restart postfix: {}\nTry restarting manually", e);
            err_count += 1;
        }
    }
    match client.execute_command("docker restart $(docker ps -qaf name=dovecot-mailcow)") {
        Ok(_) => {
            info!("successfully restarted dovecot");
        }
        Err(e) => {
            error!("failed to restart dovecot: {}\nTry restarting manually", e);
            err_count += 1;
        }
    }

    if err_count == 0 {
        tracing::Span::current().record("result", "success");
    } else {
        tracing::Span::current().record("result", "partially successful");
    }

    info!("finished update process. see result field for more details");

    client.disconnect()
}
