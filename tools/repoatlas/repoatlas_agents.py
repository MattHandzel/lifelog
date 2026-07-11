#!/usr/bin/env python3
from __future__ import annotations

import argparse
import dataclasses
import json
import os
import re
import shlex
import subprocess
import tempfile
from collections import defaultdict, deque
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path
from typing import Any, Dict, Iterable, List, Optional, Set, Tuple

IGNORE_DIRS = {".git", "target", "node_modules", "worktrees", ".direnv", "dist", "build", ".next"}
CODE_EXTS = {".rs", ".py", ".ts", ".tsx", ".js", ".jsx", ".go", ".java", ".kt", ".sql", ".proto"}


@dataclasses.dataclass
class Evidence:
    file: str
    line: int


@dataclasses.dataclass
class Node:
    id: str
    kind: str
    label: str
    file: Optional[str] = None
    boundary: Optional[str] = None


@dataclasses.dataclass
class Edge:
    source: str
    target: str
    kind: str
    evidence: List[Evidence]
    confidence: float
    tags: List[str]


@dataclasses.dataclass
class Drift:
    level: str
    message: str
    source: str
    target: str
    evidence: List[Evidence]


def rel(root: Path, path: Path) -> str:
    return str(path.relative_to(root)).replace("\\", "/")


def should_skip(path: Path) -> bool:
    return any(part in IGNORE_DIRS for part in path.parts)


def iter_files(root: Path) -> Iterable[Path]:
    for base, dirs, files in os.walk(root):
        dirs[:] = [d for d in dirs if d not in IGNORE_DIRS]
        b = Path(base)
        if should_skip(b):
            continue
        for name in files:
            p = b / name
            if p.suffix in CODE_EXTS:
                yield p


def load_expected(path: Path) -> Dict[str, Any]:
    if not path.exists():
        return {}
    if path.suffix.lower() == ".json":
        return json.loads(path.read_text())
    try:
        import yaml  # type: ignore
        return yaml.safe_load(path.read_text()) or {}
    except ModuleNotFoundError:
        raise SystemExit("YAML expected model requires pyyaml; use JSON or install pyyaml")


def path_match(path: str, pattern: str) -> bool:
    regex = "^" + re.escape(pattern).replace(r"\*\*", ".*").replace(r"\*", "[^/]*") + "$"
    return re.match(regex, path) is not None


def assign_boundaries(nodes: Dict[str, Node], expected: Dict[str, Any]) -> None:
    boundaries = expected.get("boundaries", [])
    for n in nodes.values():
        if not n.file:
            continue
        for b in boundaries:
            if any(path_match(n.file, patt) for patt in b.get("includes", [])):
                n.boundary = b.get("name")
                break


def parse_rule_pair(value: str) -> Optional[Tuple[str, str]]:
    if "->" not in value:
        return None
    left, right = [x.strip() for x in value.split("->", 1)]
    if not left or not right:
        return None
    return left, right


def detect_drift(nodes: Dict[str, Node], edges: List[Edge], expected: Dict[str, Any], journeys: List[dict]) -> List[Drift]:
    allow: Set[Tuple[str, str]] = set()
    forbid: Set[Tuple[str, str]] = set()
    for rule in expected.get("rules", []):
        for x in rule.get("allow", []):
            p = parse_rule_pair(x)
            if p:
                allow.add(p)
        for x in rule.get("forbid", []):
            p = parse_rule_pair(x)
            if p:
                forbid.add(p)

    out: Dict[Tuple[str, str, str], Drift] = {}
    for e in edges:
        s = nodes.get(e.source)
        t = nodes.get(e.target)
        if not s or not t or not s.boundary or not t.boundary or s.boundary == t.boundary:
            continue
        pair = (s.boundary, t.boundary)
        if pair in forbid:
            key = ("hard", e.source, e.target)
            out.setdefault(key, Drift("hard", f"Forbidden boundary edge {pair[0]} -> {pair[1]}", e.source, e.target, []))
            out[key].evidence.extend(e.evidence)
        elif allow and pair not in allow:
            key = ("soft", e.source, e.target)
            out.setdefault(key, Drift("soft", f"Unexpected boundary edge {pair[0]} -> {pair[1]}", e.source, e.target, []))
            out[key].evidence.extend(e.evidence)

    by_entry = {j["entrypoint"]: j for j in journeys}
    for req in expected.get("required_journeys", []):
        entry = req.get("entrypoint", "")
        found = next((j for k, j in by_entry.items() if entry in k), None)
        if not found:
            out[("coverage-gap", entry, "")] = Drift(
                "coverage-gap",
                f"Required journey '{req.get('name', entry)}' not found",
                entry,
                "",
                [],
            )
            continue
        for must in req.get("must_include", []):
            if must not in found.get("nodes", []):
                out[("coverage-gap", found["entrypoint"], must)] = Drift(
                    "coverage-gap",
                    f"Required journey '{req.get('name', found['entrypoint'])}' missing '{must}'",
                    found["entrypoint"],
                    must,
                    [],
                )

    return sorted(out.values(), key=lambda d: (d.level, d.message, d.source, d.target))


def build_journeys(nodes: Dict[str, Node], edges: List[Edge], max_hops: int) -> List[dict]:
    by_src: Dict[str, List[Edge]] = defaultdict(list)
    for e in edges:
        by_src[e.source].append(e)

    journeys: List[dict] = []
    for n in nodes.values():
        if n.kind != "entrypoint":
            continue
        seen = {n.id}
        queue = deque([(n.id, 0)])
        edge_keys: Set[str] = set()
        while queue:
            cur, depth = queue.popleft()
            if depth >= max_hops:
                continue
            for e in by_src.get(cur, []):
                edge_keys.add(f"{e.kind}:{e.source}->{e.target}")
                if e.target not in seen:
                    seen.add(e.target)
                    queue.append((e.target, depth + 1))
        journeys.append({
            "name": n.id,
            "entrypoint": n.id,
            "narrative": f"{n.id} expands to {len(seen)} nodes within {max_hops} hops",
            "nodes": sorted(seen),
            "edges": sorted(edge_keys),
        })
    return sorted(journeys, key=lambda j: j["name"])


def make_prompt(role: str, expected: Dict[str, Any], snippets: List[dict]) -> str:
    schema = {
        "nodes": [{"id": "module:server::grpc_service", "kind": "module|entrypoint|resource", "label": "...", "file": "server/src/grpc_service.rs"}],
        "edges": [{"source": "entry:rpc:Query", "target": "module:server::grpc_service", "kind": "entry_invokes|import|call|resource_access|qualified_ref", "confidence": 0.9, "tags": ["network"], "evidence": [{"file": "proto/lifelog.proto", "line": 42}]}],
        "decisions": [{"claim": "...", "status": "TODO", "impact_surface": ["nodeA", "nodeB"], "evidence": [{"file": "...", "line": 1}]}],
    }

    return (
        "You are a repo architecture extraction agent. "
        f"Role: {role}.\n"
        "Return JSON ONLY with this shape:\n"
        f"{json.dumps(schema, indent=2)}\n"
        "Rules:\n"
        "- Include only claims with concrete file:line evidence.\n"
        "- Use stable ids: entry:..., module:..., resource:...\n"
        "- Avoid speculation; confidence <= 1.0.\n"
        "- Focus on high-value architecture/journey edges.\n"
        f"Expected architecture model:\n{json.dumps(expected, indent=2)}\n"
        f"Repo snippets:\n{json.dumps(snippets, indent=2)}\n"
    )


def collect_snippets(root: Path, role: str, max_files: int, max_lines: int) -> List[dict]:
    files = sorted(iter_files(root))
    selected: List[Path] = []

    priority_patterns = {
        "entrypoints": [r"/src/main\.rs$", r"/main\.py$", r"\.proto$", r"routes?", r"grpc", r"server\.rs$", r"justfile$"],
        "dependencies": [r"Cargo\.toml$", r"package\.json$", r"/src/", r"mod\.rs$"],
        "resources": [r"\.sql$", r"db", r"schema", r"migrations", r"query", r"postgres", r"surreal"],
        "drift": [r"README", r"DESIGN", r"SPEC", r"server/", r"collector/", r"common/", r"interface/"],
    }
    patterns = [re.compile(p, re.IGNORECASE) for p in priority_patterns.get(role, [])]

    for f in files:
        r = rel(root, f)
        if any(p.search(r) for p in patterns):
            selected.append(f)
    if len(selected) < max_files:
        for f in files:
            if f not in selected:
                selected.append(f)
                if len(selected) >= max_files:
                    break

    snippets: List[dict] = []
    for f in selected[:max_files]:
        text = f.read_text(errors="ignore").splitlines()
        preview = text[:max_lines]
        snippets.append({"file": rel(root, f), "line_count": len(text), "preview": preview})
    return snippets


def run_agent(agent: str, model: str, prompt: str, timeout_s: int) -> str:
    if agent == "codex":
        cmd = ["codex"]
        if model:
            cmd.extend(["--model", model])
        cmd.extend(["--dangerously-bypass-approvals-and-sandbox", "--", prompt])
    elif agent == "gemini":
        cmd = ["gemini"]
        if model:
            cmd.extend(["--model", model])
        cmd.extend(["-y", "-i", "--", prompt])
    elif agent == "claude":
        cmd = ["claude"]
        if model:
            cmd.extend(["--model", model])
        cmd.append(prompt)
    else:
        raise RuntimeError(f"Unsupported agent backend: {agent}")

    proc = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout_s)
    if proc.returncode != 0:
        raise RuntimeError(f"Agent command failed ({proc.returncode}): {' '.join(shlex.quote(x) for x in cmd)}\n{proc.stderr[:1000]}")
    return proc.stdout.strip() or proc.stderr.strip()


def extract_json(text: str) -> Dict[str, Any]:
    m = re.search(r"```json\s*(\{.*\})\s*```", text, re.DOTALL)
    if m:
        return json.loads(m.group(1))
    start = text.find("{")
    end = text.rfind("}")
    if start >= 0 and end > start:
        return json.loads(text[start : end + 1])
    raise ValueError("No JSON object found in agent output")


def normalize_node(n: Dict[str, Any]) -> Optional[Node]:
    if not n.get("id") or not n.get("kind"):
        return None
    return Node(
        id=str(n["id"]),
        kind=str(n["kind"]),
        label=str(n.get("label", n["id"])),
        file=n.get("file"),
    )


def normalize_edge(e: Dict[str, Any]) -> Optional[Edge]:
    if not e.get("source") or not e.get("target") or not e.get("kind"):
        return None
    evidence = [Evidence(str(x.get("file", "")), int(x.get("line", 1))) for x in (e.get("evidence") or []) if x.get("file")]
    return Edge(
        source=str(e["source"]),
        target=str(e["target"]),
        kind=str(e["kind"]),
        confidence=float(e.get("confidence", 0.5)),
        tags=[str(t) for t in (e.get("tags") or [])],
        evidence=evidence,
    )


def merge_results(outputs: List[Dict[str, Any]]) -> Tuple[Dict[str, Node], List[Edge], List[dict]]:
    nodes: Dict[str, Node] = {}
    edge_seen: Set[Tuple[str, str, str, str, int]] = set()
    edges: List[Edge] = []
    decisions: Dict[str, dict] = {}

    for out in outputs:
        for raw in out.get("nodes", []):
            node = normalize_node(raw)
            if not node:
                continue
            if node.id not in nodes:
                nodes[node.id] = node
            else:
                if not nodes[node.id].file and node.file:
                    nodes[node.id].file = node.file

        for raw in out.get("edges", []):
            edge = normalize_edge(raw)
            if not edge:
                continue
            ev = edge.evidence[0] if edge.evidence else Evidence("", 0)
            key = (edge.kind, edge.source, edge.target, ev.file, ev.line)
            if key in edge_seen:
                continue
            edge_seen.add(key)
            edges.append(edge)

        for d in out.get("decisions", []):
            claim = d.get("claim")
            if not claim:
                continue
            if claim not in decisions:
                decisions[claim] = {
                    "claim": claim,
                    "status": d.get("status", "TODO"),
                    "impact_surface": sorted(set(d.get("impact_surface", []))),
                    "evidence": d.get("evidence", []),
                }

    return nodes, edges, sorted(decisions.values(), key=lambda d: d["claim"])


def augment_core_nodes(root: Path, nodes: Dict[str, Node], edges: List[Edge]) -> None:
    for f in iter_files(root):
        rp = rel(root, f)
        if f.name in {"main.rs", "main.py", "__main__.py"}:
            entry = f"entry:cli:{rp.split('/')[0]}"
            mod = f"module:{rp.split('/')[0]}::root"
            nodes.setdefault(entry, Node(entry, "entrypoint", entry, rp))
            nodes.setdefault(mod, Node(mod, "module", mod, rp))
            edges.append(Edge(entry, mod, "entry_invokes", [Evidence(rp, 1)], 0.95, ["entrypoint"]))


def render_markdown(nodes: Dict[str, Node], edges: List[Edge], journeys: List[dict], decisions: List[dict], drift: List[Drift], agent_outputs: Dict[str, str]) -> str:
    lines: List[str] = []
    lines.append("# RepoAtlas Agent Report")
    lines.append("")
    lines.append(f"- Nodes: {len(nodes)}")
    lines.append(f"- Edges: {len(edges)}")
    lines.append(f"- Journeys: {len(journeys)}")
    lines.append(f"- Decisions: {len(decisions)}")
    lines.append(f"- Drift findings: {len(drift)}")
    lines.append("")
    lines.append("## Agent Runs")
    lines.append("")
    for role, status in agent_outputs.items():
        lines.append(f"- `{role}`: {status}")
    lines.append("")
    lines.append("## Drift")
    lines.append("")
    if not drift:
        lines.append("No drift findings.")
    for d in drift:
        lines.append(f"- [{d.level}] {d.message} (`{d.source}` -> `{d.target}`)")
    lines.append("")
    lines.append("## Journeys")
    lines.append("")
    for j in journeys:
        lines.append(f"- `{j['entrypoint']}` nodes={len(j['nodes'])} edges={len(j['edges'])}")
    lines.append("")
    lines.append("## Decisions")
    lines.append("")
    for d in decisions[:200]:
        lines.append(f"- [{d.get('status', 'TODO')}] {d['claim']}")
    lines.append("")
    return "\n".join(lines)


def run(repo: Path, expected_path: Path, out: Path, agent: str, model: str, max_hops: int, timeout_s: int, include_bootstrap: bool) -> None:
    expected = load_expected(expected_path)

    roles = ["entrypoints", "dependencies", "resources", "drift"]
    prompts: Dict[str, str] = {}
    for role in roles:
        snippets = collect_snippets(repo, role, max_files=18, max_lines=140)
        prompts[role] = make_prompt(role, expected, snippets)

    outputs: List[Dict[str, Any]] = []
    statuses: Dict[str, str] = {}

    with ThreadPoolExecutor(max_workers=4) as pool:
        futures = {
            pool.submit(run_agent, agent, model, prompts[role], timeout_s): role
            for role in roles
        }
        for fut in as_completed(futures):
            role = futures[fut]
            try:
                raw = fut.result()
                parsed = extract_json(raw)
                outputs.append(parsed)
                statuses[role] = "ok"
            except Exception as exc:
                statuses[role] = f"failed ({exc})"

    if include_bootstrap:
        temp = Path(tempfile.mkdtemp(prefix="repoatlas-bootstrap-"))
        cmd = [
            "python3",
            str((repo / "tools/repoatlas/repoatlas.py").resolve()),
            "--repo",
            str(repo),
            "--expected",
            str(expected_path),
            "--out",
            str(temp),
            "--max-hops",
            str(max_hops),
        ]
        proc = subprocess.run(cmd, capture_output=True, text=True)
        if proc.returncode == 0:
            bootstrap = json.loads((temp / "graph.json").read_text())
            outputs.append({"nodes": bootstrap.get("nodes", []), "edges": bootstrap.get("edges", []), "decisions": []})
            statuses["bootstrap"] = "ok"
        else:
            statuses["bootstrap"] = f"failed ({proc.stderr[:300]})"

    nodes, edges, decisions = merge_results(outputs)
    augment_core_nodes(repo, nodes, edges)
    assign_boundaries(nodes, expected)

    journeys = build_journeys(nodes, edges, max_hops)
    drift = detect_drift(nodes, edges, expected, journeys)

    out.mkdir(parents=True, exist_ok=True)
    (out / "graph.json").write_text(json.dumps({"nodes": [dataclasses.asdict(n) for n in nodes.values()], "edges": [dataclasses.asdict(e) for e in edges]}, indent=2))
    (out / "journeys.json").write_text(json.dumps(journeys, indent=2))
    (out / "decisions.json").write_text(json.dumps(decisions, indent=2))
    (out / "drift.json").write_text(json.dumps([dataclasses.asdict(d) for d in drift], indent=2))
    (out / "report.md").write_text(render_markdown(nodes, edges, journeys, decisions, drift, statuses))

    print(f"RepoAtlas agent report generated in {out}")


def main() -> None:
    p = argparse.ArgumentParser(description="RepoAtlas agent orchestrator")
    p.add_argument("--repo", default=".")
    p.add_argument("--expected", default="repoatlas/expected_arch.json")
    p.add_argument("--out", default="docs/repoatlas")
    p.add_argument("--agent", default="codex", choices=["codex", "gemini", "claude"])
    p.add_argument("--model", default="")
    p.add_argument("--max-hops", type=int, default=4)
    p.add_argument("--timeout-seconds", type=int, default=240)
    p.add_argument("--bootstrap-static", action="store_true", help="Merge static bootstrap graph as fallback")
    args = p.parse_args()

    repo = Path(args.repo).resolve()
    expected = Path(args.expected)
    if not expected.is_absolute():
        expected = (repo / expected).resolve()
    out = Path(args.out)
    if not out.is_absolute():
        out = (repo / out).resolve()

    run(repo, expected, out, args.agent, args.model, args.max_hops, args.timeout_seconds, args.bootstrap_static)


if __name__ == "__main__":
    main()
