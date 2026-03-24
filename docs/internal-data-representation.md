# Internal Data Representation

The system represents all captured and derived data as **Streams of Records**.

## Schema Definition (Protobuf)

The authoritative schema for all records is defined in `proto/lifelog_types.proto`. 
- **Point Records**: Events at a single point in time (e.g., Keystroke, ScreenFrame).
- **Interval Records**: Events spanning a duration (e.g., AudioChunk, ActiveWindow).

## Physical Storage (PostgreSQL)

While the logic is "Proto-First", the physical storage uses **PostgreSQL** for
metadata (unified `frames` table) and a **Filesystem CAS** for large blobs.

### Table Mapping
All data modalities are stored in a single `frames` table. The `modality`
column identifies the data type; modality-specific fields go in a `payload
JSONB` column. A `catalog` table tracks registered origins (device + modality).

### Record Structure
Frame records map directly from Protobuf-generated types. The `payload` column
stores modality-specific fields as JSONB. Binary data is referenced by
`blob_hash` (SHA-256) and stored in the CAS.

## Blobs and CAS
Large payloads (images, audio) are not stored in the database. Instead:
1. Payload is hashed (SHA256).
2. Payload is stored in the Filesystem CAS at `~/.lifelog/cas/<hash_prefix>/<hash_rest>`.
3. The database record stores the hash reference.

## Future: Knowledge Graph
There is an ongoing exploration into representing relationships (e.g., "Screen X was active while Audio Y was playing") as a Knowledge Graph. Currently, this is achieved via **Temporal Correlation Queries** over the table-based storage.