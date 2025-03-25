use std::process::Command;
use tracing::{error, info};
use tracing_subscriber::fmt::format;

pub fn generate_rsa_keypair(algorithm: &str, filename: &str, comment: &str) {
    let key_type = match algorithm {
        "rsa4096" => "rsa4096",
        "ed25519" => "ed25519",
        _ => "ed25519",
    };

    let output = Command::new("ssh-keygen")
        .args(["-t", key_type, "-f", filename, "-N", "", "-C", comment])
        .output()
        .expect("Failed to execute ssh-keygen");

    let pub_key_filename = format!("{}.pub", filename);

    if output.status.success() {
        info!("generated key pair");
        info!("add the following to the authorized_keys file on the certificate server");
        info!("{}", std::fs::read_to_string(pub_key_filename).unwrap());
    } else {
        error!("ssh-keygen failed: {:?}", output);
    }
}