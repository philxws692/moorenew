# Installation

Choose one of the options below.

## Release binary

Download the latest release for your platform from GitHub Releases and place it in your PATH (for example `/usr/local/bin`).

## Build from source

```bash
cargo build --release
```

The binary will be at `target/release/moorenew`.

## Setup certificate provisioning server
Since the certificates managed by Nginx Proxy Manager are stored with restrictive permissions, copy them to a location that moorenew can access and pull over SFTP. One simple approach is a cron job that syncs the files and adjusts group ownership.

Add these cron entries on the certificate provisioning server:

```cron
*/5 * * * * rsync -L -u -v /home/docker/docker-containers/nginx/letsencrypt/live/npm-7/fullchain.pem /home/docker/fullchain.pem && chgrp docker /home/docker/fullchain.pem
*/5 * * * * rsync -L -u -v /home/docker/docker-containers/nginx/letsencrypt/live/npm-7/privkey.pem /home/docker/privkey.pem && chgrp docker /home/docker/privkey.pem
```

This cron job assumes the user pulling the certificates is in the `docker` group.

Point `npm_cert_path` in `config.toml` to `/home/docker/fullchain.pem` and `private_key_path` to `/home/docker/privkey.pem` so moorenew can download them over SFTP.
