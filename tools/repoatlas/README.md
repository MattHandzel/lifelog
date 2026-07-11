# RepoAtlas CLI

RepoAtlas workflow is split into two stages:
1. `repoatlas.py` builds graph/journeys/decisions/drift via static analysis.
2. `viz_compose.py` builds `view_config.json` for visualization.

`viz_compose.py` always applies deterministic rule-based icon/style mapping.
Optionally, it uses an LLM agent to add layout/legend/story overlays from static outputs.

## Usage

Static graph extraction:

```bash
python3 tools/repoatlas/repoatlas.py \
  --repo . \
  --expected repoatlas/expected_arch.json \
  --out docs/repoatlas \
  --max-hops 4
```

Visualization config composition (LLM required):

```bash
python3 tools/repoatlas/viz_compose.py \
  --repo . \
  --graph docs/repoatlas/graph.json \
  --journeys docs/repoatlas/journeys.json \
  --decisions docs/repoatlas/decisions.json \
  --drift docs/repoatlas/drift.json \
  --out docs/repoatlas/view_config.json \
  --agent codex --model gpt-5 --require-llm
```

Visualization config composition (rule-based only):

```bash
python3 tools/repoatlas/viz_compose.py \
  --repo . \
  --graph docs/repoatlas/graph.json \
  --journeys docs/repoatlas/journeys.json \
  --decisions docs/repoatlas/decisions.json \
  --drift docs/repoatlas/drift.json \
  --out docs/repoatlas/view_config.json
```

## Input Model

The expected architecture file accepts JSON by default. YAML is supported when `pyyaml` is available.

Core keys:
- `boundaries`: named groups with glob-like `includes`
- `rules`: `allow` and `forbid` dependency pairs (`"a -> b"`)
- `required_journeys`: required entrypoints with `must_include` nodes

## Viewer

```bash
python3 tools/repoatlas/viewer/serve_viewer.py --root . --port 8123
```

Open:
- `http://127.0.0.1:8123/tools/repoatlas/viewer/index.html`
