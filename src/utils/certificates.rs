use crate::utils::fileext::FileExt;
use crate::utils::ssh::SSHClient;
use std::fs::File;
use std::path::Path;
use std::process::exit;
use tracing::info;

pub fn download_certificates(
    client: &SSHClient,
    mail_cert_path: &Path,
    npm_cert_path: &Path,
    dry_run: bool,
) {
    let npm_fullchain_path = npm_cert_path.join("fullchain.pem");
    let npm_private_key_path = npm_cert_path.join("privkey.pem");

    let mailcow_cert_path = mail_cert_path.join("cert.pem");
    let mailcow_private_key_path = mail_cert_path.join("key.pem");

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
                    .download_file(&npm_fullchain_path, &mailcow_cert_path)
                    .unwrap();
            }
            downloads += 1;
        }

        if client.get_remote_sha256(&npm_private_key_path).unwrap() != curr_private_key_sha {
            info!("downloaded privkey.pem into key.pem");
            if !dry_run {
                client
                    .download_file(&npm_private_key_path, &mailcow_private_key_path)
                    .unwrap();
            }
            downloads += 1;
        }
    } else {
        if !dry_run {
            client
                .download_file(&npm_fullchain_path, &mailcow_cert_path)
                .unwrap();

            client
                .download_file(&npm_private_key_path, &mailcow_private_key_path)
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
