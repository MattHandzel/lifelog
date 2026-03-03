# PostgreSQL Migration Phase 4: Operations, Deployment, & Finalization

**Objective:** Ensure a smooth transition for existing development environments and production deployments, and remove SurrealDB dependencies where no longer needed.

- [ ] **Task 4.1: NixOS & Systemd Updates**
  - Update `flake.nix` to provision PostgreSQL (`services.postgresql.enable = true`) and automatically create the `lifelog` database/user.
  - Update `deploy/systemd/lifelog-server.service` and other related systemd files to depend on postgres instead of surrealdb (or both during transition).
- [ ] **Task 4.2: Config & Documentation**
  - Update `lifelog-config.toml` examples to use PostgreSQL as the default.
  - Update `USAGE.md` and `README.md` to reflect the new PostgreSQL requirement.
- [ ] **Task 4.3: Health & Metrics**
  - Ensure `ReportState` and observability endpoints correctly reflect PostgreSQL pool metrics (active connections, idle connections).
- [ ] **Task 4.4: Final Clean-up (Strategic)**
  - If Phase 1-3 are fully verified, consider making PostgreSQL the mandatory default and marking SurrealDB for removal in a future release.
  - Clean up any temporary "Hybrid" logic if it's no longer serving a purpose.