# Documentation Pipeline Plan

## Overview

A staged plan to build automated documentation generation for Lifelog. Each stage is independently useful and builds on the previous.

## Stage 1: Foundation (CLI + API Reference)

**Goal:** Auto-generated reference docs from existing sources.

### Tasks
1. **CLI reference generator** - Script that builds binaries and captures `--help` output into `docs/cli/REFERENCE.md`
2. **Proto API reference** - Run `protoc-gen-doc` on `proto/*.proto` to produce `docs/api/API.md`
3. **Rustdoc generation** - Add `just docs` recipe: `cargo doc --no-deps`
4. **Justfile recipe** - `just docs-generate` runs all three

### Deliverables
- `docs/cli/REFERENCE.md`
- `docs/api/API.md`
- `just docs-generate` recipe

### Effort: ~2 hours

---

## Stage 2: Visual Capture (Screenshots)

**Goal:** Automated screenshot capture of every UI view.

### Prerequisites
- Server running with seed data (or existing 28k+ frames)
- Interface dev server at `http://localhost:1420`

### Tasks
1. **Install Playwright** in `interface/` devDependencies
2. **Write capture script** - `docs/scripts/capture-screenshots.spec.ts`
   - Navigate to each view (dashboard, search, network, settings)
   - Click through each modality tab
   - Capture full-page screenshots
   - Save to `docs/screenshots/`
3. **Justfile recipe** - `just docs-screenshots`

### Deliverables
- `docs/screenshots/` directory with ~15 view captures
- `docs/scripts/capture-screenshots.spec.ts`

### Effort: ~3 hours

---

## Stage 3: Golden Workflow Walkthroughs

**Goal:** Step-by-step guides with embedded screenshots for each golden workflow.

### Tasks
1. **Write walkthrough docs** for Tier 1 workflows:
   - `docs/guides/quickstart.md`
   - `docs/guides/collecting-data.md`
   - `docs/guides/searching.md`
   - `docs/guides/timeline-replay.md`
2. **Embed screenshots** from Stage 2 captures
3. **Add terminal command examples** with expected output

### Deliverables
- `docs/guides/` directory with 4 walkthrough docs
- Updated README.md linking to guides

### Effort: ~4 hours

---

## Stage 4: Slide Decks

**Goal:** Presentation-ready overview and demo decks.

### Tasks
1. **Install Marp CLI**
2. **Create slide decks:**
   - `docs/slides/overview.md` - Project overview (architecture, features, vision)
   - `docs/slides/demo.md` - Live demo walkthrough with screenshots
   - `docs/slides/developer-guide.md` - Contributing and development setup
3. **Justfile recipe** - `just docs-slides`

### Deliverables
- `docs/slides/*.md` source files
- `docs/slides/*.html` generated presentations
- `just docs-slides` recipe

### Effort: ~3 hours

---

## Stage 5: Video Generation

**Goal:** Animated walkthroughs from screenshot sequences.

### Tasks
1. **Enhance Playwright capture** to produce numbered frame sequences per workflow
2. **ffmpeg pipeline script** - `docs/scripts/generate-videos.sh`
   - Converts frame sequences to MP4 and GIF
   - Configurable framerate and resolution
3. **Justfile recipe** - `just docs-video`

### Deliverables
- `docs/videos/` directory
- `docs/scripts/generate-videos.sh`
- `just docs-video` recipe

### Effort: ~2 hours

---

## Stage 6: Unified Pipeline

**Goal:** One command generates everything.

### Tasks
1. **Master recipe** - `just docs-all` runs stages 1-5 in order
2. **Freshness checks** - Skip stages whose inputs haven't changed (file hashes)
3. **CI integration** - Add docs generation to validation gate (optional, warn-only)

### Deliverables
- `just docs-all` recipe
- `docs/scripts/build-all-docs.sh`

### Effort: ~1 hour

---

## Total Estimated Scope

| Stage | Effort | Dependencies |
|-------|--------|-------------|
| 1. CLI + API Reference | ~2h | Nix build environment |
| 2. Visual Capture | ~3h | Running server + UI |
| 3. Walkthrough Guides | ~4h | Screenshots from Stage 2 |
| 4. Slide Decks | ~3h | Screenshots from Stage 2 |
| 5. Video Generation | ~2h | Frames from Stage 2 |
| 6. Unified Pipeline | ~1h | All previous stages |
| **Total** | **~15h** | |

## Tool Versions

| Tool | Version | Install |
|------|---------|---------|
| Playwright | ^1.49 | `npm install -D @playwright/test` |
| Marp CLI | ^4.0 | `npm install -D @marp-team/marp-cli` |
| ffmpeg | system | Via Nix shell |
| protoc-gen-doc | latest | Via Nix or `go install` |
| Mermaid CLI | ^11.0 | `npm install -D @mermaid-js/mermaid-cli` (optional) |
