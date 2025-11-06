mod system;
mod utils;

use crate::system::serviceproviders::ServiceProvider;
use crate::utils::certificates::download_certificates;
use crate::utils::config::generate_config;
use crate::utils::configuration::{Configuration, read_config_from_file};
use crate::utils::errors::MoorenewError;
use crate::utils::logging;
use crate::utils::ssh::SSHClient;
use crate::utils::sshkeygen;
use clap::{Command, arg};
use std::env;
use std::path::Path;
use std::process::exit;
use std::time::Duration;
use system::sysinfo;
use tokio::time::sleep;
use tracing::metadata::LevelFilter;
use tracing::{error, info, instrument};

#[tokio::main]
async fn main() {
    let user_path = env::home_dir().unwrap();
    let config_path = user_path.join(".moorenew/config.toml");
    if !Path::new(config_path.as_os_str()).exists() {
        Configuration::new().write_to_file()
    }

    let args = Command::new("moorenew")
        .subcommand(
            Command::new("keygen")
                .about("Generate a SSH Keypair which MooRenew uses to fetch the certificates")
                .args([
                    arg!(-a --algorithm <algorithm> "The algorithm to use for the keypair. Defaults to ed25519. Can also be set to rsa4096 for RSA keypair"),
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
                        .arg(
                            arg!(-f --force "Forcefully overwrite existing files")
                        )
                )
        )
        .subcommand(
            Command::new("run")
                .about("Run the update process")
                .arg(
                    arg!(-d --dry "Don't actually update the certificates, just print what would happen")
                )
        )
        .subcommand(
            Command::new("config")
                .about("Edit the moorenew configuration file")
        )
        .subcommand_required(true)
        .arg_required_else_help(true)
        .get_matches();

    if args.subcommand_matches("config").is_some() {
        match edit::edit_file(config_path.as_os_str()) {
            Ok(_) => {
                info!("successfully edited config file");
            }
            Err(_) => {
                error!("failed to edit config file");
            }
        }
    }

    let config = match read_config_from_file() {
        Some(configuration) => configuration,
        None => {
            exit(1);
        }
    };

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

        let comment: String = if arg_comment.is_empty() {
            format!(
                "{}@{}",
                sysinfo::get_loggedin_user(),
                sysinfo::get_hostname()
            )
        } else {
            arg_comment.to_string()
        };

        if algorithm != "rsa4096" && algorithm != "ed25519" {
            error!("{} is not a valid algorithm, using ed25519", algorithm);
            algorithm = "ed25519";
        }

        sshkeygen::generate_rsa_keypair(
            algorithm,
            filename,
            &comment,
            &config.sftp_host,
            &config.sftp_port,
        );
    }

    if let Some(subcommand) = args.subcommand_matches("service")
        && let Some(args) = subcommand.subcommand_matches("setup")
    {
        logging::setup_basic_logging(LevelFilter::DEBUG);
        generate_config();
        let force = args.get_flag("force");
        match system::service::create_service_files("moorenew", ServiceProvider::SYSTEMD, force) {
            Ok(_) => {
                info!("successfully created service files");
                info!(
                    "move them to /etc/systemd/system and start each service with systemctl start moorenew.service/moorenew.timer"
                );
            }
            Err(e) => {
                if let MoorenewError::ServiceConfigGenerationFailed { components } = &e {
                    error!(error = %e, components = ?components, "failed to create service files");
                }
            }
        }
    }

    if let Some(args) = args.subcommand_matches("run") {
        logging::setup_run_logging(&config.logging.level, &config);
        let dry_run = args.get_flag("dry");
        if dry_run {
            info!("running in dry run mode");
        } else {
            info!("running in normal mode");
        }
        update_certificates(dry_run, &config);
    }

    sleep(Duration::from_millis(10)).await;
}

#[instrument(fields(result), skip(configuration))]
fn update_certificates(dry_run: bool, configuration: &Configuration) {
    let username = &configuration.sftp_user;
    let host = &configuration.sftp_host;
    let port = &configuration.sftp_port;
    let private_key_path = &configuration.private_key_path;
    let public_key_path = &configuration.public_key_path;

    // Download certificates

    let client =
        SSHClient::connect(username, host, port, private_key_path, public_key_path).unwrap();

    download_certificates(
        &client,
        Path::new(&configuration.mail_cert_path),
        Path::new(&configuration.npm_cert_path),
        dry_run,
    );

    let mut err_count = 0;

    if !dry_run {
        match client.execute_command("docker restart $(docker ps -qaf name=postfix-mailcow)") {
            Ok(_) => {
                info!("successfully restarted postfix");
            }
            Err(e) => {
                error!("failed to restart postfix: {:?}", e);
                err_count += 1;
            }
        }
        match client.execute_command("docker restart $(docker ps -qaf name=dovecot-mailcow)") {
            Ok(_) => {
                info!("successfully restarted dovecot");
            }
            Err(e) => {
                error!("failed to restart dovecot: {:?}", e);
                err_count += 1;
            }
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
