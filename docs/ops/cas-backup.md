# CAS Backup

The content-addressable store (CAS) holds all binary blobs (screenshots, audio, etc.). Files are immutable once written — the filename is the SHA-256 hash of the content.

## Location

Default: `/var/lib/lifelog/cas`

Configurable via `server.casPath` in `lifelog-config.toml`.

## Backup with rsync

Because files are immutable, rsync only transfers new files on each run.

```bash
# To a local backup drive
rsync -av --checksum /var/lib/lifelog/cas/ /mnt/backup/lifelog-cas/

# To a remote host
rsync -av --checksum /var/lib/lifelog/cas/ backup-host:/mnt/backup/lifelog-cas/
```

`--checksum` skips files where the checksum already matches (safe given immutability).

For large stores, add `--progress` or use `--stats` to monitor transfer size.

## Integrity Verification

Each filename is the hex SHA-256 of its content. Verify a single file:

```bash
sha256sum /var/lib/lifelog/cas/<hash>
# Output should be: <hash>  /var/lib/lifelog/cas/<hash>
```

Verify the entire store (slow on large stores, run periodically):

```bash
find /var/lib/lifelog/cas -maxdepth 1 -type f | while read f; do
  expected=$(basename "$f")
  actual=$(sha256sum "$f" | cut -d' ' -f1)
  if [ "$expected" != "$actual" ]; then
    echo "CORRUPT: $f"
  fi
done
```

## Notes

- The CAS is not backed up by `pg_dump` — database and CAS backups must be coordinated.
- A blob referenced in the database but missing from the CAS will cause retrieval errors. Back up both together.
