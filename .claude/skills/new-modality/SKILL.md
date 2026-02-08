---
name: new-modality
description: Add a new data modality to the lifelog system (proto, schema, collector module)
disable-model-invocation: true
---
Add a new data modality called: $ARGUMENTS.

Follow this checklist in order:

1. **Proto definition**: Add message type to `proto/lifelog.proto` and enum variant to `DataModality` in `proto/lifelog_types.proto`
2. **Rebuild proto**: `just check` to regenerate code
3. **Proto impls**: Add `DataType` + `Modality` trait impls in `common/lifelog-proto/src/lib.rs` (follow existing patterns like ScreenFrame)
4. **DB schema**: Add `TableSchema` entry in `server/src/schema.rs` with fields and indexes
5. **Collector module**: Create `collector/src/modules/<name>.rs` with the collection logic
6. **Register module**: Add `pub mod <name>;` in `collector/src/modules/mod.rs` and wire into collector startup
7. **Validate**: Run `just validate` to ensure everything compiles and tests pass
