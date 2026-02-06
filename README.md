# Lifelog

The vision for the project is a software system that allows users to store information about themselves from various data sources locally, process their data into more meaningful representations, and finally have an interface to be able to interact with it in an intuitive manner to help them complete their tasks.

## AI Token-Efficient Workflow

For coding agents (Claude Code, etc.), the repo includes a small context surface and output digests to reduce token usage.

1. Read `docs/REPO_MAP.md` (stable navigation).
2. If scouting, use `prompts/agent_repo_scout.md` to produce a short file shortlist.
3. Open only the minimum files needed.
4. Run noisy commands through `tools/ai/run_and_digest.sh` and share only the digest.
5. Summarize diffs with `tools/ai/git_diff_digest.sh`.

## Installation

#### Build Dependencies

- Rustc (1.86)
- Cargo 1.82.0
- [Tesseract](https://tesseract-ocr.github.io/tessdoc/Installation.html) for OCR

Run `cargo build --release` to build the project. This will create binaries in `target/release/` folder. It will create three binaries, one for the server, one for the collector and one for the interface. The server binary is `lifelog-server`, the client binary is `lifelog-collector` and the interface binary is `lifelog-interface`.

Optionally, if you would like to only build a specific binary, you can run `cargo build --release -p <binary_name>` where `<binary_name>` is one of the three binaries mentioned above.

##### NixOS

If you are on NixOS I have graciously provided the `flake.nix` to include in your configuration. 😀

## System Diagram

![System Diagram](./docs/Lifelog.drawio.svg)

Currently our system is composed of three main components:

#### Server [./docs/server.md]

A Server is a component that is a local (but can be remote) server that receives data from the collectors and allowing the user to manage the collectors from one centralized way.

The server also is able to do transformations on data (such as OCR) which allows better retrieval of data. It also has a web interface to allow the user to manage the collectors, view the data, and query their data.

#### Collectors [./docs/collectors.md]

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
