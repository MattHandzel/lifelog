#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any, Dict, List, Set, Tuple


def load_json(path: Path) -> Any:
    return json.loads(path.read_text())


def check_file_line(repo: Path, file: str, line: int) -> Tuple[bool, str]:
    p = (repo / file).resolve()
    try:
        if not p.exists() or not p.is_file():
            return False, f"missing file: {file}"
        if line <= 0:
            return False, f"invalid line <= 0 for {file}:{line}"
        # bounded read for line count
        with p.open("r", errors="ignore") as f:
            count = 0
            for _ in f:
                count += 1
        if line > count:
            return False, f"line out of range {file}:{line} > {count}"
        return True, "ok"
    except Exception as exc:
        return False, f"error reading {file}:{line}: {exc}"


def main() -> None:
    p = argparse.ArgumentParser(description="Validate RepoAtlas artifacts")
    p.add_argument("--repo", default=".")
    p.add_argument("--graph", default="docs/repoatlas/graph.json")
    p.add_argument("--journeys", default="docs/repoatlas/journeys.json")
    p.add_argument("--drift", default="docs/repoatlas/drift.json")
    p.add_argument("--decisions", default="docs/repoatlas/decisions.json")
    p.add_argument("--out", default="docs/repoatlas/audit.json")
    args = p.parse_args()

    repo = Path(args.repo).resolve()

    def rpath(x: str) -> Path:
        y = Path(x)
        return y if y.is_absolute() else (repo / y)

    graph = load_json(rpath(args.graph))
    journeys = load_json(rpath(args.journeys))
    drift = load_json(rpath(args.drift))
    decisions = load_json(rpath(args.decisions))

    nodes = graph.get("nodes", [])
    edges = graph.get("edges", [])
    node_ids: Set[str] = {n.get("id", "") for n in nodes}
    edge_keys: Set[str] = {f"{e.get('kind')}:{e.get('source')}->{e.get('target')}" for e in edges}

    issues: List[Dict[str, Any]] = []

    # 1) graph edge endpoints
    bad_edge_endpoints = 0
    for e in edges:
        s = e.get("source", "")
        t = e.get("target", "")
        if s not in node_ids or t not in node_ids:
            bad_edge_endpoints += 1
            issues.append({
                "type": "edge_endpoint",
                "message": f"edge endpoint missing: {s} -> {t}",
            })

    # 2) evidence integrity
    evidence_total = 0
    evidence_bad = 0
    for container_name, items in [("edges", edges), ("drift", drift), ("decisions", decisions)]:
        for it in items:
            for ev in it.get("evidence", []) or []:
                evidence_total += 1
                file = ev.get("file", "")
                line = int(ev.get("line", 0))
                ok, msg = check_file_line(repo, file, line)
                if not ok:
                    evidence_bad += 1
                    issues.append({
                        "type": "evidence",
                        "container": container_name,
                        "message": msg,
                    })

    # 3) journeys
    journey_bad = 0
    for j in journeys:
        entry = j.get("entrypoint", "")
        if entry not in node_ids:
            journey_bad += 1
            issues.append({"type": "journey", "message": f"entrypoint missing: {entry}"})
        for nid in j.get("nodes", []) or []:
            if nid not in node_ids:
                journey_bad += 1
                issues.append({"type": "journey", "message": f"journey node missing: {nid}"})
        for ek in j.get("edges", []) or []:
            if ek not in edge_keys:
                journey_bad += 1
                issues.append({"type": "journey", "message": f"journey edge missing in graph: {ek}"})

    # 4) drift endpoints
    drift_bad = 0
    for d in drift:
        s = d.get("source", "")
        t = d.get("target", "")
        level = d.get("level", "")
        if s and s not in node_ids:
            drift_bad += 1
            issues.append({"type": "drift", "message": f"drift source missing: {s}"})
        if level != "coverage-gap" and t and t not in node_ids:
            drift_bad += 1
            issues.append({"type": "drift", "message": f"drift target missing: {t}"})

    pass_checks = {
        "edge_endpoints": bad_edge_endpoints == 0,
        "evidence_integrity": evidence_bad == 0,
        "journey_integrity": journey_bad == 0,
        "drift_integrity": drift_bad == 0,
    }

    audit = {
        "summary": {
            "nodes": len(nodes),
            "edges": len(edges),
            "journeys": len(journeys),
            "drift": len(drift),
            "decisions": len(decisions),
            "evidence_total": evidence_total,
        },
        "checks": pass_checks,
        "error_counts": {
            "edge_endpoints": bad_edge_endpoints,
            "evidence": evidence_bad,
            "journey": journey_bad,
            "drift": drift_bad,
        },
        "issues": issues[:200],
        "status": "pass" if all(pass_checks.values()) else "fail",
    }

    out = rpath(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(audit, indent=2))

    print(f"Artifact audit written to {out}")
    print(json.dumps({"status": audit["status"], "checks": pass_checks, "errors": audit["error_counts"]}, indent=2))


if __name__ == "__main__":
    main()
