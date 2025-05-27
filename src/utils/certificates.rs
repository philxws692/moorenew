use crate::utils::fileext::FileExt;
use crate::utils::ssh::SSHClient;
use std::env;
use std::fs::File;
use std::process::exit;
use tracing::info;

pub fn download_certificates(client: &SSHClient, dry_run: bool) {
    let mailcow_cert_base_path = &env::var("MAILCOW_CERT_PATH").unwrap()[..];

    let npm_cert_path = &env::var("NPM_CERT_PATH").unwrap()[..];
    let npm_fullchain_path = format!("{npm_cert_path}/fullchain.pem");
    let npm_private_key_path = format!("{npm_cert_path}/privkey.pem");

    let mailcow_cert_path = format!("{mailcow_cert_base_path}/cert.pem");
    let mailcow_private_key_path = format!("{mailcow_cert_base_path}/key.pem");

    let curr_cert_sha = match File::open(mailcow_cert_path.clone()) {
        Ok(curr_cert_file) => curr_cert_file.sha256().unwrap(),
        Err(_) => "".to_owned(),
    };

    let curr_private_key_sha = match File::open(mailcow_private_key_path.clone()) {
        Ok(curr_private_key_file) => curr_private_key_file.sha256().unwrap(),
        Err(_) => "".to_owned(),
    };

    let mut downloads = 0;

    // Check via checksum if the certificates changed
    if !curr_cert_sha.is_empty() && !curr_private_key_sha.is_empty() {
        if client.get_remote_sha256(&npm_fullchain_path).unwrap() != curr_cert_sha {
            info!("downloaded fullchain.pem into cert.pem");
            if !dry_run {
                client
                    .download_file(
                        &format!("{}{}", npm_cert_path, "/fullchain.pem"),
                        &mailcow_cert_path,
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
                        &mailcow_private_key_path,
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
                    &mailcow_cert_path,
                )
                .unwrap();

            client
                .download_file(
                    &format!("{}{}", npm_cert_path, "/privkey.pem"),
                    &mailcow_private_key_path,
                )
                .unwrap();
        }
        downloads += 2
    }

    if downloads == 0 {
        info!("no new certificates available, exiting");
        tracing::Span::current().record("result", "up to date");
        exit(0)
    } else {
        info!("downloaded {} certificates", downloads);
    }
}
