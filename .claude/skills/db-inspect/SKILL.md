---
name: db-inspect
description: Inspect the SurrealDB schema, tables, and data for a running lifelog instance
disable-model-invocation: true
allowed-tools: Bash, Read, Grep
---

Inspect the lifelog SurrealDB instance. Focus on: $ARGUMENTS

Available commands (SurrealDB must be running at 127.0.0.1:7183):
```bash
# List all tables
surreal sql --endpoint http://127.0.0.1:7183 --username root --password root --namespace lifelog --database lifelog --hide-welcome "INFO FOR DB;"

# Show table schema
surreal sql --endpoint http://127.0.0.1:7183 --username root --password root --namespace lifelog --database lifelog --hide-welcome "INFO FOR TABLE screen;"

# Count records
surreal sql --endpoint http://127.0.0.1:7183 --username root --password root --namespace lifelog --database lifelog --hide-welcome "SELECT count() FROM screen GROUP ALL;"

# Sample records
surreal sql --endpoint http://127.0.0.1:7183 --username root --password root --namespace lifelog --database lifelog --hide-welcome "SELECT * FROM screen LIMIT 3;"
```

Steps:
1. Check if SurrealDB is running (try connecting)
2. List all tables and their schemas
3. Count records per table
4. If $ARGUMENTS specifies a table, show sample data and schema details
5. Compare actual schema against expected schema in `server/src/schema.rs`
6. Report any mismatches between code and database
