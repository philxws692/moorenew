use std::process::Command;
use tracing::{error, info};

use crate::utils::errors::MoorenewError;

pub fn generate_rsa_keypair(
    algorithm: &str,
    filename: &str,
    comment: &str,
    hostname: &str,
    port: &u16,
) -> Result<(), MoorenewError> {
    let key_type = match algorithm {
        "rsa4096" => "rsa4096",
        "ed25519" => "ed25519",
        _ => "ed25519",
    };

    let output = if key_type == "rsa4096" {
        Command::new("ssh-keygen")
            .args([
                "-t", "rsa", "-b", "4096", "-f", filename, "-N", "", "-C", comment,
            ])
            .output()
            .map_err(|e| MoorenewError::LocalCommandExecutionError {
                command: "ssh-keygen".to_owned(),
                error: e,
            })?
    } else {
        Command::new("ssh-keygen")
            .args(["-t", key_type, "-f", filename, "-N", "", "-C", comment])
            .output()
            .map_err(|e| MoorenewError::LocalCommandExecutionError {
                command: "ssh-keygen".to_owned(),
                error: e,
            })?
    };

    let pub_key_filename = format!("{}.pub", filename);

    if output.status.success() {
        info!("generated key pair");
        info!(
            "add the content of the following file to the authorized_keys file on the certificate server: {pub_key_filename}"
        );
        info!("to do so you can execute the following command inside of this directory");
        info!(
            "ssh-copy-id -i {} -f {}@{}",
            pub_key_filename, hostname, port
        );
    } else {
        error!("ssh-keygen failed: {:?}", output);
    }

    Ok(())
}
