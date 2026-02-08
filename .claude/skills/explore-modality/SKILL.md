---
name: explore-modality
description: Trace a data modality end-to-end from proto definition through collector, server, and storage
context: fork
agent: Explore
---

Trace the `$ARGUMENTS` data modality through the entire system. Report:

1. **Proto definition**: Find the message type in `proto/lifelog.proto` and enum in `proto/lifelog_types.proto`
2. **Generated code**: Check `common/lifelog-proto/src/lib.rs` for DataType/Modality trait impls
3. **Collector module**: Find the collector module that produces this data type in `collector/src/modules/`
4. **Data flow**: How does the collector capture → serialize → send this data?
5. **Server ingestion**: How does the server receive and store this data? Check `server/src/ingest.rs` and `server/src/schema.rs`
6. **DB schema**: What table, fields, and indexes exist for this modality?
7. **Query path**: Can this data be queried? Check `server/src/query.rs`
8. **Transform**: Is there a transform pipeline for this data? Check `server/src/transform.rs`

Return a concise map showing the data's journey through the system with specific file:line references.
