mod utils;

use crate::utils::logging;
use crate::utils::sftp::SftpClient;
use dotenv::dotenv;
use std::env;

fn main() {
    dotenv().ok();

    logging::setup_logging();

    let username = &env::var("SFTP_USERNAME").unwrap()[..];
    let host = &env::var("SFTP_HOST").unwrap()[..];
    let private_key_path = &env::var("PRIVATE_KEY_PATH").unwrap()[..];
    let public_key_path = &env::var("PUBLIC_KEY_PATH").unwrap()[..];

    let client = SftpClient::connect(username, host, private_key_path, public_key_path).unwrap();
}
