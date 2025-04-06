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
    let mailcow_cert_base_path = &env::var("MAILCOW_CERT_PATH").unwrap()[..];

    let npm_cert_path = &env::var("NPM_CERT_PATH").unwrap()[..];
    let npm_fullchain_path = format!("{npm_cert_path}/fullchain.pem");
    let npm_private_key_path = format!("{npm_cert_path}/privkey.pem");

    let mailcow_cert_path = format!("{mailcow_cert_base_path}/cert.pem");
    let mailcow_private_key_path = format!("{mailcow_cert_base_path}/key.pem");

    let client = SSHClient::connect(username, host, private_key_path, public_key_path).unwrap();

    let curr_cert_sha;
    let curr_private_key_sha;

    match File::open(mailcow_cert_path.clone()) {
        Ok(curr_cert_file) => {
            curr_cert_sha = curr_cert_file.sha256().unwrap();
        }
        Err(_) => {
            curr_cert_sha = "".to_owned();
        }
    }

    match File::open(mailcow_private_key_path.clone()) {
        Ok(curr_private_key_file) => {
            curr_private_key_sha = curr_private_key_file.sha256().unwrap();
        }
        Err(_) => {
            curr_private_key_sha = "".to_owned();
        }
    }

    let mut downloads = 0;

    if curr_cert_sha != "" && curr_private_key_sha != "" {
        if client.get_remote_sha256(&npm_fullchain_path).unwrap() != curr_cert_sha {
            info!("downloaded fullchain.pem into cert.pem");
            if dry_run {
                client
                    .download_file(
                        &format!("{}{}", npm_cert_path, "/fullchain.pem"),
                        &*mailcow_cert_path,
                    )
                    .unwrap();
            }
            downloads += 1;
        }

        if client.get_remote_sha256(&npm_private_key_path).unwrap() != curr_private_key_sha {
            info!("downloaded privkey.pem into key.pem");
            if !dry_run {
                client
                    .download_file(
                        &format!("{}{}", npm_cert_path, "/privkey.pem"),
                        &*mailcow_private_key_path,
                    )
                    .unwrap();
            }
            downloads += 1;
        }
    } else {
        if !dry_run {
            client
                .download_file(
                    &format!("{}{}", npm_cert_path, "/fullchain.pem"),
                    &*mailcow_cert_path,
                )
                .unwrap();

            client
                .download_file(
                    &format!("{}{}", npm_cert_path, "/privkey.pem"),
                    &*mailcow_private_key_path,
                )
                .unwrap();
        }
        downloads += 2
    }

    if downloads == 0 {
        info!("no new certificates available, exiting");
        exit(0)
    } else {
        info!("downloaded {} certificates", downloads);
    }

    client.disconnect()
}
