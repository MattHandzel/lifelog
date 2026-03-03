# PostgreSQL Migration Phase 3: Query Execution & AST Translation

**Objective:** Translate the complex temporal overlap logic currently handled in Rust into native PostgreSQL engine operations for high performance.

- [ ] **Task 3.1: Table Queries & Full-Text Search**
  - Update `ExecutionPlan::TableQuery` in `server/src/query/executor.rs` (or equivalent) to generate PostgreSQL SQL.
  - Translate text searches from SurrealDB's `SEARCH ANALYZER` to PostgreSQL's `to_tsvector` and `@@` operator using GIN indexes.
- [ ] **Task 3.2: Temporal DuringQuery Translation**
  - Implement native PostgreSQL range overlap logic (`&&` operator on `TSTZRANGE`).
  - Instead of pulling all intervals into Rust, perform an `INNER JOIN` in the database where time ranges overlap.
- [ ] **Task 3.3: Replay/Timeline Alignment**
  - Update `Replay` RPC logic to query PostgreSQL.
  - Ensure results are ordered by the lower bound of the `time_range`.
- [ ] **Task 3.4: Hybrid Query Routing**
  - Just like with Ingestion, implement a router that decides whether to send a query to SurrealDB or PostgreSQL based on the configured backend or availability of data.
  - This allows incremental migration of query capabilities.