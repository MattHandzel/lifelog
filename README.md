# Lifelog

The vision for the project is a software system that allows users to store information about themselves from various data sources locally, process their data into more meaningful representations, and finally have an interface to be able to interact with it in an intuitive manner to help them complete their tasks.

## AI Token-Efficient Workflow

For coding agents (Claude Code, etc.), the repo includes a small context surface and output digests to reduce token usage.

1. Read `docs/REPO_MAP.md` (stable navigation).
2. If scouting, use `prompts/agent_repo_scout.md` to produce a short file shortlist.
3. Open only the minimum files needed.
4. Run noisy commands through `tools/ai/run_and_digest.sh` and share only the digest.
5. Summarize diffs with `tools/ai/git_diff_digest.sh`.

## RepoAtlas

RepoAtlas is a local analyzer for:
- journey discovery from entrypoints
- decision auditing from code evidence
- drift detection against an expected architecture model

Run it for this repo:

```bash
just repoatlas
```

`just repoatlas` now does:
1. static graph extraction (source of truth)
2. LLM-assisted visualization composition (`view_config.json`) from static outputs
3. rule-based icon/style mapping for nodes (deterministic)

Set visualization agent backend/model via env vars:

```bash
REPOATLAS_AGENT=codex REPOATLAS_MODEL=gpt-5 just repoatlas
```

Static-only mode (no LLM overlay):

```bash
just repoatlas-static
```

Run it for any repository path:

```bash
python3 tools/repoatlas/repoatlas.py \
  --repo /path/to/target/repo \
  --expected /path/to/expected_arch.json \
  --out /path/to/output-dir \
  --max-hops 4
```

Then compose visualization config:

```bash
python3 tools/repoatlas/viz_compose.py \
  --repo /path/to/target/repo \
  --graph /path/to/output-dir/graph.json \
  --journeys /path/to/output-dir/journeys.json \
  --decisions /path/to/output-dir/decisions.json \
  --drift /path/to/output-dir/drift.json \
  --out /path/to/output-dir/view_config.json \
  --agent codex --model gpt-5 --require-llm
```

Expected architecture model: `repoatlas/expected_arch.json` (JSON by default; YAML also supported if `pyyaml` is installed).
Outputs:
- `graph.json`
- `journeys.json`
- `decisions.json`
- `drift.json`
- `report.md`
- `view_config.json` (viewer styles/icons/layout hints)

Interactive viewer:

```bash
just repoatlas-view
# open http://127.0.0.1:8123/tools/repoatlas/viewer/index.html
```

## Installation

#### Build Dependencies

- Rustc (1.86)
- Cargo 1.82.0
- [Tesseract](https://tesseract-ocr.github.io/tessdoc/Installation.html) for OCR
- PostgreSQL 16+ (required for migrated ingest/query paths)

Run `nix develop --command cargo build --release` to build the project. This will create binaries in `target/release/` folder. It will create three binaries, one for the server, one for the collector and one for the interface. The server binary is `lifelog-server`, the client binary is `lifelog-collector` and the interface binary is `lifelog-server-frontend`.

Optionally, if you would like to only build a specific binary, you can run `nix develop --command cargo build --release -p <binary_name>` where `<binary_name>` is one of the three binaries mentioned above.

##### NixOS

If you are on NixOS, include this flake and enable the provided module:

```nix
{
  imports = [ inputs.lifelog.nixosModules.lifelog-postgres ];
  services.lifelog.postgres.enable = true;
}
```

This provisions PostgreSQL and auto-creates a `lifelog` DB/user by default.

## Persistent Deployment

For boot-resilient split deployment (server on home server + collector on laptop), use the runbook in [USAGE.md](./USAGE.md), section `11. Persistent Distributed Deployment`.

## System Diagram

![System Diagram](./docs/Lifelog.drawio.svg)

Currently our system is composed of three main components:

#### Server [./docs/server.md]

A Server is a component that is a local (but can be remote) server that receives data from the collectors and allowing the user to manage the collectors from one centralized way.

The server also is able to do transformations on data (such as OCR) which allows better retrieval of data. It also has a web interface to allow the user to manage the collectors, view the data, and query their data.

#### Collectors [./docs/collector.md]

A collector is a component that runs on the device and collects data from various data sources. It is responsible for defining the data sources available on the device, logging data from those sources, and responding to requests from the server.

#### Interface [./docs/interface.md]

The interface is how the users can interact with the system. It is a desktop application that is able to connect to the server and run queries on the server to retrieve their lifelog.

### References

Some references used for this project:

```
https://link.springer.com/article/10.1007/s11948-013-9456-1
This talked about challenges and feasibility of lifelog software

https://x.com/vin_acct/status/1876088761664385346

https://github.com/nanovin/gaze
https://github.com/openrecall/openrecall
These two are some other software that try to do the same thing. Copied some code from nanovin.

[ImageBind: One Embedding Space to Bind Them All](https://arxiv.org/pdf/2305.05665)
This paper talks about and shows some very cool examples of the benefits of having one embedding space for many different data modalities.

[LifeInsight: An interactive lifelog retrieval system with comprehensive spatial insight and query assistance](https://dl.acm.org/doi/10.1145/3592573.3593106)
This paper gave some ideas for how to do the relevance feedback. Some cool ideas are to use LLMs to refine queries and to allow the user to select data modalities to add to queries
```
