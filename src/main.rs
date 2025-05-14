mod system;
mod utils;

use crate::system::serviceproviders::ServiceProvider;
use crate::utils::certificates::download_certificates;
use crate::utils::config::generate_config;
use crate::utils::logging;
use crate::utils::ssh::SSHClient;
use crate::utils::sshkeygen;
use crate::utils::sysinfo;
use clap::{arg, Command};
use std::env;
use std::path::Path;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, instrument};
use tracing::metadata::LevelFilter;

#[tokio::main]
async fn main() {
    dotenv::from_filename(".env.moorenew").ok();

    let args = Command::new("moorenew")
        .subcommand(
            Command::new("keygen")
                .about("Generate a SSH Keypair which MooRenew uses to fetch the certificates")
                .args([
                    arg!(-a --algorithm <algorithm> "The algorithm to use for the keypair. Defaults to ed25519. Can also be set to rsa4096 for RSA keypairs"),
                    arg!(-c --comment <comment> "The comment to use for the keypair. Defaults to the username@hostname of the current user"),
                    arg!(-f --filename <filename> "The filename to use for the keypair. Defaults to moorenew"),
                ])
        )
        .subcommand(
            Command::new("service")
                .about("Used to manage the moorenew service")
                .subcommand(
                    Command::new("setup")
                        .about("Used to create the needed service setup files")
                )
        )
        .subcommand(
            Command::new("run")
                .about("Run the update process")
                .arg(
                    arg!(-d --dry-run "Don't actually update the certificates, just print what would happen"),
                )
        )
        .subcommand_required(true)
        .arg_required_else_help(true)
        .get_matches();

    if let Some(args) = args.subcommand_matches("keygen") {
        logging::setup_basic_logging(LevelFilter::INFO);
        let mut algorithm = args
            .get_one::<String>("algorithm")
            .map(String::as_str)
            .unwrap_or("ed25519");
        let arg_comment = args
            .get_one::<String>("comment")
            .map(String::as_str)
            .unwrap_or("");
        let filename = args
            .get_one::<String>("filename")
            .map(String::as_str)
            .unwrap_or("moorenew");

        let comment: String;

        if arg_comment == "" {
            comment = format!(
                "{}@{}",
                sysinfo::get_loggedin_user(),
                sysinfo::get_hostname()
            );
        } else {
            comment = arg_comment.to_string();
        }

        if algorithm != "rsa4096" && algorithm != "ed25519" {
            error!("{} is not a valid algorithm, using ed25519", algorithm);
            algorithm = "ed25519";
        }

        sshkeygen::generate_rsa_keypair(&algorithm, &filename, &comment);
    }

    if let Some(subcommand) = args.subcommand_matches("service") {
        if let Some(_) = subcommand.subcommand_matches("setup") {
            logging::setup_basic_logging(LevelFilter::INFO);
            match system::services::create_service_files("moorenew", ServiceProvider::SYSTEMD) {
                Ok(_) => {
                    info!("successfully created service files");
                }
                Err(_) => {
                    error!("failed to create service files");
                }
            }
        }
    }

    if let Some(args) = args.subcommand_matches("run") {
        logging::setup_run_logging();
        let dry_run = args.get_flag("dry-run");
        if dry_run {
            info!("running in dry run mode");
        } else {
            info!("running in normal mode");
        }
        update_certificates(dry_run);
    }

    if !Path::new(".env.moorenew").exists() {
        info!("assuming first run due to missing config");
        info!("generating default config in .env.moorenew");
        generate_config();
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
