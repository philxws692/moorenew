# 🐮 moorenew

Automatically updates the mailcow certificates from an existing certificate location like Nginx Proxy Manager

# 🔧 Setup
The running account needs to be in the `docker` group since the script needs to restart the relevant containers (postfix and dovecot). To add your user to the docker group run the following command:
```bash
sudo usermod -aG docker $USER
```

# 💡 Planned features
- [ ] Add TOML feature to configure multiple jobs
- [ ] Move config from .env to TOML as well
- [ ] Add generation of service configuration files for RC like on alpine linux