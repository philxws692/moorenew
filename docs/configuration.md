# Configuration

On first run, moorenew creates a default config file at `~/.moorenew/config.toml`.

Edit it with:

```bash
moorenew config
```

## Example configuration

```toml
sftp_host = "example.com"
sftp_port = 22
sftp_user = "mailcow"
private_key_path = "/root/.ssh/moorenew"
public_key_path = "/root/.ssh/moorenew.pub"
npm_cert_path = "/path/on/source/host/cert.pem"
mail_cert_path = "/opt/mailcow-dockerized/data/assets/ssl/cert.pem"

[logging]
level = "info"
structured_logging = false

containers = [
  "postfix-mailcow",
  "dovecot-mailcow",
  "nginx-mailcow"
]

buzz_urls = [
  "gotify://myawesome.gotify.com/myawesomepath/myawesometoken",
  "ntfy://username:password@ntfy.host/mytopic"
]
```

## Field notes

- `npm_cert_path` is the remote path moorenew downloads from.
- `mail_cert_path` is the Mailcow certificate destination path.
- Add or remove containers depending on your Mailcow deployment.
