---
name: gen-test-data
description: Generate realistic test data fixtures for a data modality
disable-model-invocation: true
---

Generate test data for the `$ARGUMENTS` modality.

1. Look up the proto message definition in `proto/lifelog.proto`
2. Look up any `Distribution<T> for StandardUniform` implementations in `common/lifelog-proto/src/lib.rs` for existing patterns
3. Generate a Rust function that produces realistic test fixtures:
   - Random but plausible field values (real-looking URLs for browser, sensible timestamps, etc.)
   - A builder pattern or factory function
   - Both single-item and batch generators
4. Place the function in the appropriate test module or `server/tests/harness/mod.rs`
5. Run `just test` to verify it compiles and works

Follow the existing `Distribution<ScreenFrame>` pattern in lifelog-proto as a template.
