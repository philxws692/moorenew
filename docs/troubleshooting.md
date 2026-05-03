# Troubleshooting

## Permission denied writing certs

moorenew must run as `root` or a user in the `root` group because Mailcow writes `cert.pem` and `key.pem` with root-only permissions:

```bash
ls -ll /opt/mailcow-dockerized/data/assets/ssl
```

## Container restarts fail

Verify the container names in your `containers` list match the Mailcow services in Docker:

```bash
docker ps --format "{{.Names}}"
```
