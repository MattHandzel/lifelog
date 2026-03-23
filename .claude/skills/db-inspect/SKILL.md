---
name: db-inspect
description: Inspect the PostgreSQL schema, tables, and data for a running lifelog instance
disable-model-invocation: true
allowed-tools: Bash, Read, Grep
---

Inspect the lifelog PostgreSQL instance. Focus on: $ARGUMENTS

Available commands (PostgreSQL must be running, default DSN: postgresql://lifelog@127.0.0.1:5432/lifelog):

```bash
# List all tables
psql "$LIFELOG_POSTGRES_INGEST_URL" -c "\dt"

# Describe the frames table
psql "$LIFELOG_POSTGRES_INGEST_URL" -c "\d frames"

# Count records per modality
psql "$LIFELOG_POSTGRES_INGEST_URL" -c "SELECT modality, count(*) FROM frames GROUP BY modality ORDER BY count DESC;"

# Sample records
psql "$LIFELOG_POSTGRES_INGEST_URL" -c "SELECT id, modality, origin_id, created_at FROM frames ORDER BY created_at DESC LIMIT 10;"

# Check catalog
psql "$LIFELOG_POSTGRES_INGEST_URL" -c "SELECT * FROM catalog LIMIT 20;"
```

Steps:
1. Check if PostgreSQL is reachable (try connecting)
2. List all tables and their schemas
3. Count records per modality in the frames table
4. If $ARGUMENTS specifies a modality or query, show sample data with payload details
5. Report any anomalies (orphaned blobs, missing catalog entries, etc.)
