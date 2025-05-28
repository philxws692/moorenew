# 🐮 moorenew
<div align="center">
<img src="images/moorenew_banner.jpg" alt="moorenew banner" width="100%">
</div>

Automatically updates the mailcow certificates from an existing certificate location like Nginx Proxy Manager

# 🔧 Setup

The script needs to be run as `root` or as a user which is in the root group. You might think, "Eww, I don't want to have a unknown script messing with my valuable certificates." Right! This is a legit concern. The point is you don't have to. But then you'll have to manage the certificate update process by yourself.
You may be asking, why?
Here's why:
```bash
ls -ll /opt/mailcow-dockerized/data/assets/ssl
```
You'll see that `cert.pem` and `key.pem` have only write permissions for user `root` and group `root`. Hence, it is necessary to run the script as root.

# 💡 Planned features
- [x] ~~Move config from .env to TOML as well~~
- [ ] Enable configuration editing via command
- [ ] Add TOML feature to configure multiple jobs
- [ ] Add generation of service configuration files for RC like on alpine linux