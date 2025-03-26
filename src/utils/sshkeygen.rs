use std::env;
use std::os::unix::process::ExitStatusExt;
use std::process::{Command, Output};
use tracing::{error, info};
use tracing_subscriber::fmt::format;

pub fn generate_rsa_keypair(algorithm: &str, filename: &str, comment: &str) {
    let key_type = match algorithm {
        "rsa4096" => "rsa4096",
        "ed25519" => "ed25519",
        _ => "ed25519",
    };

    let mut output: Output = Output {
        status: std::process::ExitStatus::from_raw(1),
        stdout: Vec::new(),
        stderr: Vec::new(),
    };
    if key_type == "rsa4096" {
        output = Command::new("ssh-keygen")
            .args(["-t", "rsa", "-b", "4096", "-f", filename, "-N", "", "-C", comment])
            .output()
            .expect("Failed to execute ssh-keygen");
    } else if key_type == "ed25519" {
        output = Command::new("ssh-keygen")
            .args(["-t", key_type, "-f", filename, "-N", "", "-C", comment])
            .output()
            .expect("Failed to execute ssh-keygen");
    } else {
        error!("unsupported key type '{}'", key_type);
    }

    let pub_key_filename = format!("{}.pub", filename);

    if output.status.success() {
        info!("generated key pair");
        info!("add the following to the authorized_keys file on the certificate server");
        info!("{}", pub_key_filename);
        info!("to do so you can execute the following command inside of this directory");
        info!("ssh-copy-id -i {} -f {}@{}", pub_key_filename, env::var("SFTP_USERNAME").unwrap(), env::var("SFTP_HOST").unwrap());
    } else {
        error!("ssh-keygen failed: {:?}", output);
    }
}