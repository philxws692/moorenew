# Service (systemd)

Generate the service files:

```bash
moorenew service setup
```

Move them into place and enable the timer:

```bash
sudo mv moorenew.service /etc/systemd/system/moorenew.service
sudo mv moorenew.timer /etc/systemd/system/moorenew.timer
sudo systemctl daemon-reload
sudo systemctl enable --now moorenew.timer
```

## Verify

```bash
systemctl status moorenew.timer
```

```bash
journalctl -u moorenew.service -n 200 --no-pager
```
