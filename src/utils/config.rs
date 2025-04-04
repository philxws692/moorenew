use std::fs::File;
use std::io::Write;

pub fn generate_config() {
    let config_string = "STRUCTURED_LOGGING=false
PRIVATE_KEY_PATH=moorenew
PUBLIC_KEY_PATH=moorenew.pub
SFTP_USERNAME=docker
SFTP_HOST=127.0.0.1
SFTP_PORT=22
NPM_CERT_PATH=/path/to/nginx/cert
MAILCOW_CERT_PATH=/opt/mailcow-dockerized/data/assets/ssl";

    let mut file = File::create(".env.moorenew").unwrap();
    file.write_all(config_string.as_bytes()).unwrap();

}
