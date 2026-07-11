# Agent Teams Orchestration Prompt

This session may run overnight unattended.

## Phase 0: Orient

Before creating any tasks or spawning teammates, orient yourself:

1. Read CLAUDE.md to understand the build system, conventions, and anti-patterns
2. Run `just init-session` to get the repo map and current state
3. Read the task inventory: `~/notes/project/lifelog/all-issues-complete-inventory.md`
4. Read the session plan: `~/notes/project/lifelog/session-plans-v3-comprehensive.md` — this contains pre-made decisions, research doc references, and phase ordering
5. Check git status and recent commits to understand what's already been done

From the inventory and session plan, build your task list. Use the inventory IDs (C1, H5, M12, etc.) to track tasks. Respect the dependency ordering in the session plan.

## Phase 1: Plan

After reading the inventory and session plan:

1. Identify which tasks are blocked by unresolved decisions — flag these for the user, do not start them
2. Identify which tasks have pre-resolved decisions in the session plan — use those decisions
3. Map the dependency ordering between tasks
4. Classify which tasks touch code vs non-code files
5. For each task, assess: what's the best approach? What could go wrong? What research docs should the teammate read first? What files will be touched?

Create tasks with dependencies. Then spawn teammates.

## Metacognition

### Plan before executing
Before starting any non-trivial task, pause and plan HOW to do it well — not just WHAT to do. Ask:
- What's the best way to achieve high quality here?
- What domain knowledge am I missing that I should research first? (Check the research docs in `~/notes/project/lifelog/` — Section 12 of the session plan has the full index)
- Should this be decomposed into smaller steps?
- What are the risks — what files could break, what assumptions am I making?

### Generate alternatives before committing
For tasks with design decisions (not just mechanical fixes), generate at least 2-3 approaches before picking one. Consider different levels: is this a code fix, an architecture change, a config change, or a removal? Pick the simplest approach that fully solves the problem.

### Review your own work
After completing each task, before committing, review:
- Did I actually fix the problem, or just the symptom?
- Did I introduce new issues? (Check with `just check`)
- Did I miss anything the task description specified?
- Does this match the existing code style?

### Learn and adapt during the session
- If a pattern of errors keeps appearing (e.g., a module import that's wrong everywhere), fix the pattern once systematically rather than one-off each occurrence
- If you discover a new issue while working on a task, log it — don't silently fix unrelated things or ignore them
- If the task inventory turns out to be wrong about a file location or the current state of the code, trust what you see in the repo over what the inventory says
- After each phase, briefly assess: what went well, what was slower than expected, what should the next phase do differently?

### Store values, not rules
When you encounter a problem and find a solution, understand WHY the solution works — not just WHAT fixed it. This helps you handle similar problems later in the session without repeating the same debugging cycle.

## Team Structure

Use two teammates:

**Code teammate** — handles all source code changes (.rs, .proto, migrations, Cargo.toml). Works through tasks sequentially since concurrent edits to the same Rust workspace cause unpredictable cargo check results.

**Docs/Infra teammate** — handles everything else: markdown, CI workflows, Nix files, Docker, deploy configs. Can work in parallel with the code teammate since they never touch the same files.

If a task spans both code and non-code, assign it to whoever owns the harder part.

## Rules

### Verification
- Code teammate: run `just check` after every task. Run `just validate` at phase boundaries.
- Docs teammate: verify files are well-formed (YAML lint, markdown renders correctly, etc.)
- Both: commit after every completed task, not in batches.

### Stuck detection
If cargo check fails 5 consecutive times on the same error, or a task has taken more than 30 minutes without progress: mark it blocked with a one-paragraph note explaining what was tried, and move on.

Notify on blocked tasks and phase completions:
```
curl -s -d "BLOCKED: [task-id] - [reason]" https://ntfy.sh/lifelog-agents
curl -s -d "COMPLETED: Phase [name] - [summary]" https://ntfy.sh/lifelog-agents
```

### Model routing
Use Opus by default. Use Sonnet for tasks that are purely mechanical with no judgment calls — things like: grep and delete dead references, copy an existing pattern to a new location, add a template file, replace unwrap() with ? in a single file.

### Coordination
The code teammate exclusively owns Cargo.toml and Cargo.lock. If the docs teammate needs a dependency change, message the code teammate to handle it.

Conflict zones (coordinate, don't edit simultaneously):
- `proto/*.proto` — cascades to all crates
- `Cargo.toml` / `Cargo.lock` — workspace-wide
- `server/src/server.rs` — central server logic

### Items that require user action
Some tasks cannot be fully automated (screenshots, destructive git operations, posting to social media). For these: prepare everything possible (draft content, write scripts), flag them clearly as needing user action, and move on.

## Phase 2: Execute

Spawn teammates and begin. The code teammate starts with critical bugs and security issues (highest priority). The docs teammate starts with repository hygiene (license, contributing guide, etc.).

Work through the session plan's phases in order. When a phase is complete, notify and move to the next.

## Phase 3: End of Session

When all tasks are complete (or the session is winding down):

1. Run `just validate` one final time to confirm nothing is broken
2. Write a session summary to `~/notes/project/lifelog/session-report-YYYY-MM-DD.md`:
   - Tasks completed (by inventory ID)
   - Tasks blocked (with reasons)
   - Tasks skipped (with reasons)
   - New issues discovered during the session
   - Items flagged for user action
3. Notify: `curl -s -d "SESSION COMPLETE - [completed]/[total] tasks. [blocked] blocked. See session report." https://ntfy.sh/lifelog-agents`
4. Self-improvement reflection: What would make the next orchestrated session more effective? What was the biggest time sink? Write 2-3 concrete suggestions at the bottom of the session report.
