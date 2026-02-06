# Internal Data Representation

The system represents all captured and derived data as **Streams of Records**.

## Schema Definition (Protobuf)

The authoritative schema for all records is defined in `proto/lifelog_types.proto`. 
- **Point Records**: Events at a single point in time (e.g., Keystroke, ScreenFrame).
- **Interval Records**: Events spanning a duration (e.g., AudioChunk, ActiveWindow).

## Physical Storage (SurrealDB)

While the logic is "Proto-First", the physical storage remains in **SurrealDB** for metadata and a **Filesystem CAS** for large blobs.

### Table Mapping
Each `DataOrigin` (Device + Modality) maps to a unique table in SurrealDB.
Format: `<device_id_sanitized>:<modality_name>` (e.g., `DEADC0DE:screen`).

### Record Structure
Records in SurrealDB are isomorphic to their Protobuf counterparts. Thanks to the `pbjson` integration, we can store Proto-generated structs directly as SurrealDB documents.

## Blobs and CAS
Large payloads (images, audio) are not stored in the database. Instead:
1. Payload is hashed (SHA256).
2. Payload is stored in the Filesystem CAS at `~/.lifelog/cas/<hash_prefix>/<hash_rest>`.
3. The database record stores the hash reference.

## Future: Knowledge Graph
There is an ongoing exploration into representing relationships (e.g., "Screen X was active while Audio Y was playing") as a Knowledge Graph. Currently, this is achieved via **Temporal Correlation Queries** over the table-based storage.