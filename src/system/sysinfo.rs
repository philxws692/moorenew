use std::io::Error;
use std::process::Command;

use crate::utils::errors::MoorenewError;

pub fn get_loggedin_user() -> Result<String, MoorenewError> {
    let out = Command::new("whoami")
        .output()
        .map_err(|e| MoorenewError::LocalCommandExecutionError {
            command: "whoami".to_owned(),
            error: e,
        })?
        .stdout;
    Ok(String::from_utf8(out)
        .map_err(|e| MoorenewError::Utf8Output {
            command: "whoami".to_owned(),
            error: e,
        })?
        .replace("\n", ""))
}

pub fn get_hostname() -> Result<String, MoorenewError> {
    let out = Command::new("hostname")
        .output()
        .map_err(|e| MoorenewError::LocalCommandExecutionError {
            command: "hostname".to_owned(),
            error: e,
        })?
        .stdout;
    Ok(String::from_utf8(out)
        .map_err(|e| MoorenewError::Utf8Output {
            command: "hostname".to_owned(),
            error: e,
        })?
        .replace("\n", ""))
}

pub fn get_binary_path() -> Result<String, Error> {
    match std::env::current_exe() {
        Ok(path) => Ok(format!("{}", path.display())),
        Err(e) => Err(Error::other(format!("Could not get binary path: {e}"))),
    }
}
