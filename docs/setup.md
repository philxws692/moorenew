# Setup

This guide covers installation, initial configuration, and service setup for scheduled certificate updates.

## Prerequisites

- Mailcow installed (default SSL path: `/opt/mailcow-dockerized/data/assets/ssl`)
- Docker available on the system
- SSH access to the system hosting your source certificates (for example Nginx Proxy Manager)
- Run as `root` (or a user in the `root` group) to overwrite `cert.pem` and `key.pem`

## Install

See [Installation](installation.md).

## Create an SSH key

Generate a keypair for the SFTP connection used to fetch certificates:

```bash
moorenew keygen
```

Copy the public key to your certificate source host (for example `~/.ssh/authorized_keys`).

## Configure

See [Configuration](configuration.md).

## Run a manual update

See [Running](running.md).

## Set up as a service

See [Service](service.md).
