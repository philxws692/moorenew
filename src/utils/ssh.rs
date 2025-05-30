use ssh2::Session;
use std::fs::File;
use std::io::{Error, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::process::exit;
use tracing::{error, info, instrument, warn};

pub struct SSHClient {
    session: Session,
    socket: TcpStream,
}

impl SSHClient {
    #[instrument(skip(private_key, public_key))]
    pub fn connect(
        username: &str,
        host: &str,
        port: &u16,
        private_key: &str,
        public_key: &str,
    ) -> std::io::Result<Self> {
        let private_key_path = Path::new(private_key);
        let public_key_path = Path::new(public_key);

        // Connect to SFTP
        match TcpStream::connect(format!("{}:{}", host, port)) {
            Ok(tcp) => {
                match Session::new() {
                    Ok(mut session) => {
                        match tcp.try_clone() {
                            Ok(tcp_clone) => {
                                session.set_tcp_stream(tcp_clone);
                            }
                            Err(e) => {
                                return Err(e);
                            }
                        }

                        match session.handshake() {
                            Ok(_) => {}
                            Err(e) => {
                                error!("Failed to handshake: {}", e);
                                exit(1);
                            }
                        }

                        // Login with publickey
                        match session.userauth_pubkey_file(
                            username,
                            Some(public_key_path),
                            private_key_path,
                            None,
                        ) {
                            Ok(_) => {}
                            Err(e) => {
                                error!("Failed to authenticate: {}", e);
                                exit(1)
                            }
                        }

                        if !session.authenticated() {
                            error!("authentication failed");
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::PermissionDenied,
                                "ssh_authentication failed",
                            ));
                        }

                        info!("connected to ssh server at {}", host);

                        Ok(Self {
                            session,
                            socket: tcp,
                        })
                    }
                    Err(e) => {
                        error!("Failed to create session: {}", e);
                        exit(1);
                    }
                }
            }
            Err(e) => Err(e),
        }
    }

    pub fn disconnect(self) {
        drop(self.session);
        if let Err(e) = self.socket.shutdown(std::net::Shutdown::Both) {
            warn!("ssh_disconnect error: {}", e);
        }
    }

    pub fn download_file(&self, remote_path: &Path, local_path: &Path) -> std::io::Result<()> {
        if let Ok(peer_addr) = self.socket.peer_addr() {
            info!(
                "downloading {} to {} from {}",
                remote_path.display(),
                local_path.display(),
                peer_addr
            );
        }

        let sftp = self.session.sftp();

        match sftp {
            Ok(sftp) => {
                let mut remote_file;
                let mut local_file;

                match sftp.open(remote_path) {
                    Ok(file) => remote_file = file,
                    Err(e) => {
                        warn!("sftp open error: {}", e);
                        exit(1)
                    }
                }

                match File::create(local_path) {
                    Ok(file) => local_file = file,
                    Err(e) => {
                        warn!("can not create local file: {}", e);
                        exit(1)
                    }
                }

                let mut buffer = Vec::new();
                if let Err(e) = remote_file.read_to_end(&mut buffer) {
                    warn!("sftp read error: {}", e);
                    return Err(e);
                }

                if let Err(e) = local_file.write_all(&buffer) {
                    warn!("sftp write error: {}", e);
                    return Err(e);
                }
            }
            Err(e) => {
                error!("sftp error: {}", e);
                exit(1)
            }
        }

        Ok(())
    }

    pub fn get_remote_sha256(&self, remote_path: &Path) -> Option<String> {
        let channel = self.session.channel_session();
        match channel {
            Ok(mut channel) => {
                let command = format!("sha256sum {}", remote_path.display());
                channel.exec(&command).unwrap();

                let mut result = String::new();
                channel.read_to_string(&mut result).unwrap();
                result = result.split(" ").nth(0).unwrap().to_string();

                let filename = remote_path.file_name().unwrap();

                info!("checksum of {} is: {}", filename.display(), result);

                channel.wait_close().unwrap();

                let exit_status = channel.exit_status().unwrap();

                if exit_status != 0 {
                    warn!(
                        "failed to close ssh channel. closed with exit status {}",
                        exit_status
                    );
                }

                Some(result)
            }
            Err(e) => {
                warn!("ssh channel creation error: {}", e);
                None
            }
        }
    }

    pub fn execute_command(&self, command: &str) -> Result<(), Error> {
        let channel = self.session.channel_session();
        match channel {
            Ok(mut channel) => {
                if let Err(e) = channel.exec(command) {
                    return Err(Error::other(
                        format!("failed to execute command: {}", e).as_str(),
                    ));
                }

                let mut result = String::new();
                channel
                    .read_to_string(&mut result)
                    .expect("failed to read command output");

                if let Err(e) = channel.wait_close() {
                    return Err(Error::other(
                        format!("failed to close ssh channel, {}", e).as_str(),
                    ));
                }

                let exit_status = channel.exit_status()?;

                if exit_status != 0 {
                    Err(Error::other(
                        format!("exit status {}", exit_status).as_str(),
                    ))
                } else {
                    Ok(())
                }
            }
            Err(e) => Err(Error::other(
                format!("ssh channel creation error, {}", e).as_str(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::configuration::read_config_from_file;
    use crate::utils::ssh::SSHClient;
    use std::path::Path;

    #[test]
    fn test_get_remote_sha256() {
        let config = read_config_from_file().unwrap();

        let username = &config.sftp_user;
        let host = &config.sftp_host;
        let port = &config.sftp_port;
        let private_key_path = &config.private_key_path;
        let public_key_path = &config.public_key_path;
        let npm_cert_path = Path::new(&config.npm_cert_path);
        let client =
            SSHClient::connect(username, host, port, private_key_path, public_key_path).unwrap();

        assert_eq!(
            client
                .get_remote_sha256(&npm_cert_path.join("fullchain.pem"))
                .unwrap(),
            "8b31c5c518332cbd5eaa07fb8c684e929536f80d75fd7808c32c3cc40184b3d4"
        );

        assert_eq!(
            client
                .get_remote_sha256(&npm_cert_path.join("privkey.pem"))
                .unwrap(),
            "02d3b98743154ed6bbd463a4c36b154d84f88635a8c6092f2d73b1afe25eee65"
        );

        client.disconnect();
    }
}
