use std::fs::File;
use std::io::Write;

pub fn generate_config() {
    let config_string = "STRUCTURED_LOGGING=false
PRIVATE_KEY_PATH=rustrover
PUBLIC_KEY_PATH=rustrover.pub
SFTP_USERNAME=root
SFTP_HOST=";

    let mut file = File::create(".env.moorenew.moorenew").unwrap();
    file.write_all(config_string.as_bytes()).unwrap();

}
