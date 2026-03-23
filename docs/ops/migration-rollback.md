# Migration Rollback

Always dump the database before running migrations. There is no automatic rollback.

## Pre-Migration Dump

```bash
pg_dump -Fc lifelog > /var/backups/lifelog/lifelog-pre-migration-$(date +%Y%m%dT%H%M%S).dump
```

`-Fc` uses the custom format, which is compressed and supports parallel restore.

## Running Migrations

```bash
# Dump first, then migrate
pg_dump -Fc lifelog > /var/backups/lifelog/lifelog-pre-migration-$(date +%Y%m%dT%H%M%S).dump
sqlx migrate run --database-url "$LIFELOG_POSTGRES_INGEST_URL"
```

## Restoring from a Dump

Drop the existing database and restore from the dump file:

```bash
dropdb lifelog
createdb lifelog
pg_restore -Fc -d lifelog /var/backups/lifelog/lifelog-pre-migration-<timestamp>.dump
```

Or restore into an existing (empty) database without dropping:

```bash
pg_restore -Fc -d lifelog --clean /var/backups/lifelog/lifelog-pre-migration-<timestamp>.dump
```

## Notes

- Backups directory `/var/backups/lifelog/` must exist and be writable before running.
- Keep at least the last 3 pre-migration dumps.
- Dumps do not include the CAS blob store — back that up separately (see `cas-backup.md`).
