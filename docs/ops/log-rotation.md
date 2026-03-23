# Log Rotation

The lifelog server can produce high-volume logs, especially during active collection (screenshots, OCR, audio transcription). Without limits, journald can consume gigabytes of disk.

## Recommended journald Config

Create `/etc/systemd/journald.conf.d/lifelog.conf`:

```ini
[Journal]
# Cap persistent storage for all journal logs
SystemMaxUse=2G
SystemKeepFree=500M

# Cap in-memory (volatile) journal
RuntimeMaxUse=256M
RuntimeKeepFree=128M

# Rotate individual journal files at 128 MB
SystemMaxFileSize=128M

# Keep at most 2 weeks of logs
MaxRetentionSec=2week
```

Apply without reboot:

```bash
systemctl restart systemd-journald
```

## Filtering Lifelog Logs

```bash
# Follow lifelog-server logs only
journalctl -u lifelog-server -f

# Last 1000 lines
journalctl -u lifelog-server -n 1000

# Errors only
journalctl -u lifelog-server -p err
```

## Notes

- Adjust `SystemMaxUse` based on available disk. 2 GB is a reasonable default for a home server.
- If running collector and server on the same host, the combined log volume is higher — consider reducing `MaxRetentionSec` to 1 week.
