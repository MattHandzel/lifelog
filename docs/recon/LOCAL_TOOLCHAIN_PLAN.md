# Local Toolchain Plan

All tools are local-only, no cloud services required. Platform: Linux.

## Screenshot Capture

### Tool: Playwright (headless Chromium)
- **Why:** Tauri's frontend is a Vite app accessible at `http://localhost:1420` during dev. Playwright can navigate, interact, and screenshot any view.
- **Install:** `npm install -D @playwright/test` + `npx playwright install chromium`
- **Usage:** Script navigates to each dashboard view, waits for data, takes full-page screenshots.
- **Output:** `docs/screenshots/*.png` (versioned or gitignored with a generation script)

### Alternative: `grim` / `slurp` (Wayland) or `scrot` (X11)
- For capturing the actual Tauri desktop window (with native chrome)
- Semi-manual; useful for hero images

## Slide Generation

### Tool: Marp CLI
- **Why:** Markdown-to-slides, outputs HTML/PDF/PPTX. Stays in the Markdown ecosystem.
- **Install:** `npm install -D @marp-team/marp-cli`
- **Usage:** Write `docs/slides/*.md` in Marp format. Build with `npx marp docs/slides/overview.md -o docs/slides/overview.html`
- **Output:** Self-contained HTML presentations or PDF

## Video / GIF Generation

### Tool: ffmpeg + Playwright trace
- **Why:** Combine sequential screenshots into walkthrough videos or animated GIFs.
- **Install:** Available in Nix (`pkgs.ffmpeg`)
- **Pipeline:**
  1. Playwright captures screenshots at each workflow step (numbered: `001-start.png`, `002-search.png`, etc.)
  2. `ffmpeg -framerate 1 -i docs/screenshots/workflow-%03d.png -c:v libx264 -pix_fmt yuv420p docs/videos/walkthrough.mp4`
  3. For GIFs: `ffmpeg -i walkthrough.mp4 -vf "fps=1,scale=800:-1" docs/videos/walkthrough.gif`

### Alternative: OBS Studio (manual screen recording)
- For high-fidelity demo videos with narration
- Not automatable; reserve for polished release videos

## API Documentation

### Tool: protoc-gen-doc
- **Why:** Auto-generates API reference from `.proto` files
- **Install:** Available via Nix or Go install
- **Usage:** `protoc --doc_out=docs/api --doc_opt=markdown,API.md proto/lifelog.proto proto/lifelog_types.proto`
- **Output:** `docs/api/API.md`

## CLI Documentation

### Tool: Custom script (capture --help output)
- **Pipeline:** Build binaries -> run each with `--help` -> format into markdown
- **Output:** `docs/cli/REFERENCE.md`

```bash
#!/bin/bash
for bin in lifelog-server lifelog-collector lifelog-export lifelog-mcp; do
  echo "## $bin"
  echo '```'
  nix develop --command cargo run -p $bin --bin $bin -- --help 2>&1 || true
  echo '```'
done > docs/cli/REFERENCE.md
```

## Diagram Generation

### Tool: D2 or Mermaid CLI
- **Why:** Text-to-diagram, version-controlled, renders to SVG/PNG
- **Existing:** `docs/Lifelog.drawio` (Draw.io format)
- **Recommendation:** Keep Draw.io for complex diagrams, use Mermaid for inline docs
- **Install:** `npm install -D @mermaid-js/mermaid-cli`

## Rust Documentation

### Tool: rustdoc (built-in)
- **Usage:** `nix develop --command cargo doc --no-deps --document-private-items`
- **Output:** `target/doc/` (browsable HTML)

## Summary: Package.json additions

```json
{
  "devDependencies": {
    "@playwright/test": "^1.49.0",
    "@marp-team/marp-cli": "^4.0.0",
    "@mermaid-js/mermaid-cli": "^11.0.0"
  },
  "scripts": {
    "docs:screenshots": "npx playwright test docs/scripts/capture-screenshots.spec.ts",
    "docs:slides": "npx marp docs/slides/*.md -o docs/slides/",
    "docs:video": "bash docs/scripts/generate-videos.sh",
    "docs:api": "bash docs/scripts/generate-api-docs.sh",
    "docs:cli": "bash docs/scripts/generate-cli-docs.sh",
    "docs:all": "npm run docs:screenshots && npm run docs:slides && npm run docs:video && npm run docs:api && npm run docs:cli"
  }
}
```
