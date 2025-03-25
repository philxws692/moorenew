mod utils;
mod args;

use crate::utils::logging;
use crate::utils::sftp::SftpClient;
use crate::utils::sshkeygen;
use dotenv::dotenv;
use std::env;
use std::process::exit;
use args::MooRenewArgs;
use clap::Parser;
use tracing::{error, info};
use crate::args::{Action, GenerateKeyCommand};

fn main() {
    dotenv().ok();
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
            sshkeygen::generate_rsa_keypair(&algo, "moorenew", "moorenew mailcow server")

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
