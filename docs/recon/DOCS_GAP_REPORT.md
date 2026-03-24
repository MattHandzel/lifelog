# Documentation Gap Report

## Existing Documentation Inventory

### Top-Level
- README.md - Project overview + build instructions
- USAGE.md - Full usage guide (config, run, test, deploy)
- CLAUDE.md - AI agent instructions
- AGENTS.md - Repo guidelines

### docs/ (55 files)
- Architecture: server.md, collector.md, interface.md, database.md, grpc.md, protobuf.md
- Design: vision.md, concepts.md, concepts-level-1.md, policy.md, search.md, querying.md
- Data: data-modality-representation.md, internal-data-representation.md, buffering.md, sinks.md
- Communication: server-device-communication.md, server-interface-communication.md, tcp-optimizations.md
- Plans: 12 implementation plans in docs/plans/
- Ops: 3 runbooks in docs/ops/ (migration-rollback, cas-backup, log-rotation)
- Research: 4 files in docs/research/
- Testing: tier3-real-device-design.md
- Config: CONFIG.md, SETUP_TLS.md
- Meta: REPO_MAP.md, AI_CONTEXT.md, TOKEN_OPT_REPORT.md, DEPLOYMENT_AGENT_PROMPT.md
- Archive: LESSONS.md, STATE_HISTORY.md
- Ideas: ideas.md, problems-to-solve.md, features-roadmap.md, research-challenges.md

## Identified Gaps

### Critical (blocks user onboarding)
1. **No quickstart guide** - README jumps to build deps; no "try it in 5 minutes" path
2. **No screenshot/visual tour** - Zero screenshots of the UI anywhere in docs
3. **No end-to-end walkthrough** - No guide showing "install -> configure -> collect -> search -> export"
4. **No CLI reference** - No `--help` output captured; no man pages or structured CLI docs
5. **No MCP server docs** - `lifelog-mcp` has no documentation at all

### High Priority
6. **No API reference** - gRPC service methods undocumented outside proto comments
7. **No data model guide** - Proto types exist but no human-readable field docs
8. **No export guide** - `lifelog-export` usage is not documented
9. **No interface user guide** - UI components exist but no user-facing docs
10. **No troubleshooting expansion** - USAGE.md section 10 is minimal (5 items)
11. **No privacy/security user guide** - Privacy tiers mentioned in code but no user-facing explanation

### Medium Priority
12. **No architecture diagram update** - `Lifelog.drawio.svg` exists but may be stale
13. **No contributor guide** - CLAUDE.md is agent-focused; no human contributor onboarding
14. **No changelog** - No CHANGELOG.md or release notes
15. **No demo data / seed script** - No way to explore the UI without real collector data
16. **Plans are stale** - Many docs/plans/ files reference completed or abandoned work
17. **No NixOS module reference** - Flake module options undocumented

### Low Priority
18. **Duplicate concepts files** - Both concepts.md and concepts-level-1.md exist
19. **Stale references** - Some docs reference SurrealDB (removed)
20. **No doc generation** - No rustdoc, typedoc, or proto-doc pipeline

## Recommended Priority Order

1. Quickstart guide (README rewrite or QUICKSTART.md)
2. Screenshot gallery / visual tour
3. End-to-end walkthrough with real commands
4. CLI reference (auto-generated from --help)
5. MCP + Export documentation
6. API reference (from proto files)
7. Privacy/security user guide
