# Systems Architecture Analysis

This document captures problems found in `SPEC.md` that will likely cause rework, inconsistent implementations, or mismatched expectations during a v1 rebuild.

Scope: clarity/consistency, missing acceptance criteria, ambiguous semantics, and architecture risks.

## Findings (Problems / Ambiguities)

1. Replay granularity contradicts the record model
   - `SPEC.md` defines screen capture as a *point record* (`SPEC.md:93`), but replay says “default step granularity: screen capture interval” (`SPEC.md:305-309`).
   - Required spec fix: define the mapping from a sequence of point frames to replay intervals (e.g., frame `t_i` represents `[t_i, t_{i+1})`), or redefine screen capture records as intervals with explicit `t_end`.

2. Cross-device time semantics are underspecified
   - The spec requires backend tolerance of clock skew and storing device + ingest time (`SPEC.md:98-105`) but never defines:
     - the canonical “timeline time” used for queries/UI,
     - how per-collector offsets/drift are estimated and applied,
     - what assumptions exist (NTP required/optional, maximum skew tolerated, what happens when skew exceeds tolerance).
   - Risk: cross-modal/cross-device queries become non-deterministic or depend on accidental implementation choices.

3. Correlation operators are named but not rigorously defined
   - Operators exist (`WITHIN/OVERLAPS/DURING`) (`SPEC.md:106-114`) but the semantics for point-vs-interval combinations are incomplete.
   - Examples of missing definitions:
     - `WITHIN` when A or B are interval records (compare start times? end times? nearest point? any overlap within `Δt`?).
     - `DURING` needs a formal definition for how a predicate over point streams induces time windows.
     - `Δt_default` is required but no default value or unit is pinned down (only “configurable”) (`SPEC.md:114`).
   - Required spec fix: provide truth tables / formal semantics + at least one worked example per operator.

4. Upload “chunk” framing and offsets are not specified
   - Idempotency is defined in terms of `(collector_id, stream_id, session_id, offset, chunk_hash)` (`SPEC.md:173-177`) and resumption depends on offsets (`SPEC.md:168-172`, `SPEC.md:205-212`).
   - Missing:
     - what a “chunk” is (bytes vs records; max size; compression; encryption),
     - what `offset` measures (bytes? record count? sequence number?),
     - hash algorithm and canonicalization rules (what exactly is hashed),
     - how record ids relate to chunk ids and session ids.
   - Risk: collectors/backends can’t interoperate, retries aren’t safe, and “resume” becomes brittle.

5. Durable acknowledgement is coupled to index updates (likely problematic)
   - Backend acknowledgement requires metadata persisted, blobs persisted, *and* baseline indexes updated (`SPEC.md:161-167`).
   - This is a very strong coupling that can throttle ingestion and complicate failure recovery.
   - Required spec fix:
     - define which indexes are mandatory for “baseline queryability” per modality,
     - define behavior when indexing is delayed (is data “delivered” but “not queryable”?),
     - define recovery semantics (e.g., how to re-drive indexing without blocking ingestion).

6. “Backend pull” vs “collector upload” is conceptually mixed
   - The spec says the backend “decides when to pull data” (`SPEC.md:123-127`) and “pulls” via commands over a collector-initiated control channel (`SPEC.md:145-147`).
   - But the data plane is defined as collector calling `UploadChunks` into the backend (`SPEC.md:205-208`).
   - This is workable but needs explicit phrasing like: “collector pushes bulk data, but only within backend-authorized upload sessions.”
   - Risk: implementation drift into two competing mental models.

7. Required modality “keystrokes” has a TBD policy (product/security blocker)
   - Keystrokes are listed as required, but “content policy TBD” (`SPEC.md:70`) and indexing makes “keystroke content” optional (`SPEC.md:244-252`).
   - Required spec fix: define what is captured by default (events only vs text content), what is excluded/redacted, and what user controls exist.
   - Risk: v1 scope and threat model are not actually decided.

8. Security is not specified enough for multi-device operation
   - TLS is required (`SPEC.md:346-348`) and pairing mechanism is “TBD” (`SPEC.md:350-354`).
   - Missing concrete requirements for:
     - authN/authZ model (collector permissions, UI client permissions),
     - key/cert rotation and revocation,
     - device removal and re-pairing behavior,
     - UI access control (especially since phone browser is a supported client).
   - Risk: you can’t ship safely without retrofitting security, which is expensive.

9. Performance requirements are circular/vague
   - “target SLA set by performance suite” (`SPEC.md:368-376`) doesn’t give any numeric targets.
   - Required spec fix: set explicit, testable metrics (p50/p95 latency for representative queries; ingestion throughput; storage growth).

10. Section 17 (GitHub issues snapshot) is process data, not a stable spec
   - Embedding issue lists in a normative spec (`SPEC.md:473+`) will go stale immediately and creates merge/maintenance churn.
   - Recommended: move this content to a generated report (or a separate planning doc) and keep `SPEC.md` strictly normative.

## Summary Of What To Clarify In `SPEC.md` (High Leverage)

1. Formalize time and correlation semantics (including point/interval coercions) and provide worked examples.
2. Nail the data-plane contract: chunk framing, offset units, id/hash rules, and id lifecycles.
3. Decouple delivery acknowledgement from indexing or specify strict “queryable vs durable” states.
4. Decide keystroke capture policy and define security/authz requirements for collectors and UI clients.
5. Replace vague performance requirements with concrete SLAs.

