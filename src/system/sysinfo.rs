use std::io::Error;
use std::process::Command;

pub fn get_loggedin_user() -> String {
    let out = Command::new("whoami").output().ok().unwrap().stdout;
    String::from_utf8(out).unwrap().replace("\n", "")
}

pub fn get_hostname() -> String {
    let out = Command::new("hostname").output().ok().unwrap().stdout;
    String::from_utf8(out).unwrap().replace("\n", "")
}

pub fn get_binary_path() -> Result<String, Error> {
    match std::env::current_exe() {
        Ok(path) => Ok(format!("{}", path.display())),
        Err(e) => Err(Error::other(format!("Could not get binary path: {e}"))),
    }
}
