use ssh2::Session;
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::process::exit;
use tracing::{error, info, instrument, warn};

pub struct SSHClient {
    session: Session,
    socket: TcpStream,
}

#[derive(Clone, Debug)]
struct CommandOutput {
    stdout: String,
    exit_status: i32,
}

trait RemoteCommandRunner {
    fn run(&self, command: &str) -> std::io::Result<CommandOutput>;
}

fn get_remote_sha256_with_runner<R: RemoteCommandRunner>(
    runner: &R,
    remote_path: &Path,
) -> Option<String> {
    let command = format!("sha256sum {}", remote_path.display());
    match runner.run(&command) {
        Ok(output) => {
            let result = output.stdout.split_whitespace().next()?.to_string();

            if output.exit_status != 0 {
                warn!(
                    "failed to close ssh channel. closed with exit status {}",
                    output.exit_status
                );
            }

            if let Some(filename) = remote_path.file_name() {
                info!("checksum of {} is: {}", filename.display(), result);
            }

            Some(result)
        }
        Err(e) => {
            warn!("ssh command error: {}", e);
            None
        }
    }
}

impl RemoteCommandRunner for SSHClient {
    fn run(&self, command: &str) -> std::io::Result<CommandOutput> {
        let mut channel = self.session.channel_session()?;
        channel.exec(command)?;

        let mut stdout = String::new();
        channel.read_to_string(&mut stdout)?;

        channel.wait_close()?;

        let exit_status = channel.exit_status()?;

        Ok(CommandOutput {
            stdout,
            exit_status,
        })
    }
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
        get_remote_sha256_with_runner(self, remote_path)
    }
}

#[cfg(test)]
mod tests {
    use super::{CommandOutput, RemoteCommandRunner, get_remote_sha256_with_runner};
    use std::path::Path;

    struct MockRunner {
        output: CommandOutput,
    }

    impl RemoteCommandRunner for MockRunner {
        fn run(&self, _command: &str) -> std::io::Result<CommandOutput> {
            Ok(self.output.clone())
        }
    }

    #[test]
    fn test_get_remote_sha256_mocked() {
        let runner = MockRunner {
            output: CommandOutput {
                stdout: "8b31c5c518332cbd5eaa07fb8c684e929536f80d75fd7808c32c3cc40184b3d4  /etc/ssl/fullchain.pem\n"
                    .to_string(),
                exit_status: 0,
            },
        };
        let remote_path = Path::new("/etc/ssl/fullchain.pem");

        assert_eq!(
            get_remote_sha256_with_runner(&runner, remote_path).unwrap(),
            "8b31c5c518332cbd5eaa07fb8c684e929536f80d75fd7808c32c3cc40184b3d4"
        );
    }
}
