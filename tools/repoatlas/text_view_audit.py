#!/usr/bin/env python3
from __future__ import annotations

import argparse
import collections
import json
from pathlib import Path
from typing import Any, Dict, List, Set, Tuple


def load_json(path: Path) -> Any:
    return json.loads(path.read_text())


def edge_key(e: Dict[str, Any]) -> str:
    return f"{e.get('kind')}:{e.get('source')}->{e.get('target')}"


def evidence_str(e: Dict[str, Any]) -> str:
    ev = (e.get("evidence") or [{}])[0]
    file = ev.get("file", "n/a")
    line = ev.get("line", "n/a")
    return f"{file}:{line}"


def file_exists(repo: Path, rel: str) -> bool:
    return (repo / rel).exists()


def journey_steps(graph: Dict[str, Any], journey: Dict[str, Any], max_steps: int = 120) -> List[Tuple[int, Dict[str, Any]]]:
    edges = graph.get("edges", [])
    journey_edge_set = set(journey.get("edges") or [])
    relevant = [e for e in edges if edge_key(e) in journey_edge_set]

    by_source: Dict[str, List[Dict[str, Any]]] = collections.defaultdict(list)
    for e in relevant:
        by_source[e.get("source", "")].append(e)
    for src in by_source:
        by_source[src].sort(key=lambda x: (x.get("kind", ""), x.get("target", "")))

    entry = journey.get("entrypoint", "")
    queue = collections.deque([(entry, 0)])
    seen_nodes = {entry}
    seen_edges = set()
    out: List[Tuple[int, Dict[str, Any]]] = []

    while queue and len(out) < max_steps:
        cur, depth = queue.popleft()
        for e in by_source.get(cur, []):
            k = edge_key(e)
            if k not in seen_edges:
                seen_edges.add(k)
                out.append((depth, e))
            tgt = e.get("target", "")
            if tgt and tgt not in seen_nodes:
                seen_nodes.add(tgt)
                queue.append((tgt, depth + 1))

    return out


def edge_priority(e: Dict[str, Any], entrypoint: str = "") -> int:
    kind = e.get("kind", "")
    tags = set(e.get("tags") or [])
    source = e.get("source", "")
    target = e.get("target", "")
    base = {
        "entry_invokes": 100,
        "assoc_call": 92,
        "method_call": 90,
        "call": 82,
        "qualified_ref": 52,
        "import": 40,
        "resource_access": 20,
    }.get(kind, 30)
    if "runtime-flow" in tags:
        base += 8
    if "entrypoint" in tags:
        base += 6
    if "persistence" in tags:
        base -= 6

    if entrypoint.startswith("entry:cli:collector"):
        if target.startswith("module:collector::"):
            base += 24
        if target.startswith("module:common::"):
            base -= 16
        if target.startswith("resource:"):
            base -= 20
    elif entrypoint.startswith("entry:rpc:Query"):
        if target == "module:server::server":
            base += 32
        if target.startswith("module:server::query"):
            base += 28
        elif target.startswith("module:server::"):
            base += 8
        if kind == "resource_access":
            base -= 18
        if source == "module:server::grpc_service" and target == "module:server::server":
            base += 25

    return base


def journey_edges(graph: Dict[str, Any], journey: Dict[str, Any]) -> List[Dict[str, Any]]:
    edges = graph.get("edges", [])
    journey_edge_set = set(journey.get("edges") or [])
    return [e for e in edges if edge_key(e) in journey_edge_set]


def split_primary_secondary(
    graph: Dict[str, Any],
    journey: Dict[str, Any],
    max_primary_steps: int = 36,
) -> Tuple[List[Tuple[int, Dict[str, Any]]], List[Tuple[int, Dict[str, Any]]]]:
    relevant = journey_edges(graph, journey)
    by_source: Dict[str, List[Dict[str, Any]]] = collections.defaultdict(list)
    for e in relevant:
        by_source[e.get("source", "")].append(e)
    for src in by_source:
        by_source[src].sort(
            key=lambda x: (
                -edge_priority(x, journey.get("entrypoint", "")),
                -float(x.get("confidence", 0.0)),
                x.get("target", ""),
            )
        )

    entry = journey.get("entrypoint", "")
    primary: List[Tuple[int, Dict[str, Any]]] = []
    used_edges: Set[str] = set()
    seen_nodes: Set[str] = {entry}
    queue = collections.deque([(entry, 0)])

    while queue and len(primary) < max_primary_steps:
        cur, depth = queue.popleft()
        choices = by_source.get(cur, [])
        if not choices:
            continue

        # Primary path is a bounded branching spine:
        # include multiple high-signal edges near the entry to keep runtime siblings visible.
        if depth <= 1:
            per_source_limit = 3
        elif depth == 2:
            per_source_limit = 2
        else:
            per_source_limit = 1

        taken = 0
        for e in choices:
            if len(primary) >= max_primary_steps or taken >= per_source_limit:
                break
            k = edge_key(e)
            if k in used_edges:
                continue
            score = edge_priority(e, entry)
            if score < 55:
                continue
            used_edges.add(k)
            primary.append((depth, e))
            taken += 1
            tgt = e.get("target", "")
            if tgt and tgt not in seen_nodes:
                seen_nodes.add(tgt)
                queue.append((tgt, depth + 1))

        # Ensure we don't stall if all edges were below threshold.
        if taken == 0:
            for e in choices:
                k = edge_key(e)
                if k in used_edges:
                    continue
                used_edges.add(k)
                primary.append((depth, e))
                tgt = e.get("target", "")
                if tgt and tgt not in seen_nodes:
                    seen_nodes.add(tgt)
                    queue.append((tgt, depth + 1))
                break

    # supporting edges are everything else, grouped by BFS depth from entry
    depth_map: Dict[str, int] = {entry: 0}
    q = collections.deque([entry])
    while q:
        n = q.popleft()
        d = depth_map[n]
        for e in by_source.get(n, []):
            t = e.get("target", "")
            if t and t not in depth_map:
                depth_map[t] = d + 1
                q.append(t)

    supporting: List[Tuple[int, Dict[str, Any]]] = []
    for e in relevant:
        if edge_key(e) in used_edges:
            continue
        d = depth_map.get(e.get("source", ""), 99)
        supporting.append((d, e))
    supporting.sort(
        key=lambda t: (
            t[0],
            -edge_priority(t[1], journey.get("entrypoint", "")),
            -float(t[1].get("confidence", 0.0)),
            t[1].get("source", ""),
            t[1].get("target", ""),
        )
    )
    return primary, supporting


def compute_clutter(graph: Dict[str, Any]) -> Dict[str, Any]:
    nodes = graph.get("nodes", [])
    edges = graph.get("edges", [])
    node_ids = [n.get("id", "") for n in nodes]
    out_deg = collections.Counter()
    in_deg = collections.Counter()
    for e in edges:
        s = e.get("source", "")
        t = e.get("target", "")
        out_deg[s] += 1
        in_deg[t] += 1

    total = len(node_ids)
    m = len(edges)
    density = (m / (total * (total - 1))) if total > 1 else 0.0
    hubs = sorted(((nid, out_deg[nid] + in_deg[nid]) for nid in node_ids), key=lambda x: x[1], reverse=True)[:12]

    return {
        "nodes": total,
        "edges": m,
        "density": density,
        "avg_out": (sum(out_deg.values()) / total) if total else 0.0,
        "avg_in": (sum(in_deg.values()) / total) if total else 0.0,
        "hubs": hubs,
    }


def icon_audit(graph: Dict[str, Any], view: Dict[str, Any]) -> Dict[str, Any]:
    nodes = {n.get("id", ""): n for n in graph.get("nodes", [])}
    styles = view.get("node_styles", {})
    bad: List[str] = []
    per_icon = collections.Counter()

    for nid, st in styles.items():
        icon = st.get("icon", "default")
        per_icon[icon] += 1
        kind = nodes.get(nid, {}).get("kind", "")
        if kind == "entrypoint" and icon not in {"cli", "rpc", "http", "entry"}:
            bad.append(f"entrypoint icon mismatch {nid} -> {icon}")
        if kind == "resource" and icon not in {"database", "queue", "resource"}:
            bad.append(f"resource icon mismatch {nid} -> {icon}")

    return {"per_icon": dict(per_icon), "mismatches": bad[:60], "mismatch_count": len(bad)}


def repo_specific_expectations(graph: Dict[str, Any], journeys: List[Dict[str, Any]]) -> Dict[str, Any]:
    edges = graph.get("edges", [])
    checks = []

    def has_edge(predicate) -> bool:
        return any(predicate(e) for e in edges)

    checks.append({
        "name": "collector CLI has entry edge to collector root",
        "ok": has_edge(lambda e: e.get("source") == "entry:cli:collector" and e.get("target", "").startswith("module:collector::root")),
    })

    checks.append({
        "name": "collector flow reaches collector setup/collector logic",
        "ok": has_edge(lambda e: e.get("source", "").startswith("module:collector::root") and e.get("target", "") in {"module:collector::setup", "module:collector::collector"}),
    })

    checks.append({
        "name": "RPC Query entry exists",
        "ok": any(j.get("entrypoint") == "entry:rpc:Query" for j in journeys),
    })

    checks.append({
        "name": "RPC Query reaches server query module",
        "ok": any(
            e.get("source", "").startswith("module:server::") and e.get("target", "").startswith("module:server::query")
            for e in edges
        ),
    })

    return {
        "checks": checks,
        "pass_count": sum(1 for c in checks if c["ok"]),
        "total": len(checks),
    }


def make_report(
    repo: Path,
    graph: Dict[str, Any],
    journeys: List[Dict[str, Any]],
    view: Dict[str, Any],
    audit: Dict[str, Any],
    focus_journeys: List[str],
) -> str:
    lines: List[str] = []
    lines.append("RepoAtlas Text Audit")
    lines.append("=" * 80)

    lines.append("\n[1] Correctness Model For This Project")
    lines.append("- Journey-first: entrypoints must lead to concrete module flows with evidence.")
    lines.append("- Evidence-backed: edge evidence file:line must exist in repo.")
    lines.append("- Icon semantics: entry/resource icon families must match node kinds.")
    lines.append("- Connectivity: selected journey should show explicit ordered steps.")
    lines.append("- Readability: full graph is allowed to be cluttered; journey view must be concise.")

    lines.append("\n[2] Artifact Integrity")
    s = audit.get("summary", {})
    c = audit.get("checks", {})
    e = audit.get("error_counts", {})
    lines.append(f"status={audit.get('status')} nodes={s.get('nodes')} edges={s.get('edges')} journeys={s.get('journeys')} evidence_total={s.get('evidence_total')}")
    lines.append(f"checks edge_endpoints={c.get('edge_endpoints')} evidence={c.get('evidence_integrity')} journey={c.get('journey_integrity')} drift={c.get('drift_integrity')}")
    lines.append(f"errors edge={e.get('edge_endpoints')} evidence={e.get('evidence')} journey={e.get('journey')} drift={e.get('drift')}")

    lines.append("\n[3] Icon Audit")
    ia = icon_audit(graph, view)
    lines.append(f"icon_counts={ia['per_icon']}")
    lines.append(f"icon_mismatches={ia['mismatch_count']}")
    for m in ia["mismatches"][:20]:
        lines.append(f"  - {m}")

    lines.append("\n[4] Clutter Metrics")
    cl = compute_clutter(graph)
    lines.append(f"nodes={cl['nodes']} edges={cl['edges']} density={cl['density']:.4f} avg_out={cl['avg_out']:.2f}")
    lines.append("top_hubs:")
    for nid, d in cl["hubs"]:
        lines.append(f"  - deg={d:3d} {nid}")

    lines.append("\n[5] Repo-Specific Expectations")
    ex = repo_specific_expectations(graph, journeys)
    lines.append(f"pass={ex['pass_count']}/{ex['total']}")
    for check in ex["checks"]:
        lines.append(f"  - {'OK' if check['ok'] else 'FAIL'} {check['name']}")

    journey_by_name = {j.get("name"): j for j in journeys}
    primary_fidelity_pass = 0
    primary_fidelity_total = 0

    for name in focus_journeys:
        lines.append(f"\n[Journey] {name}")
        j = journey_by_name.get(name)
        if not j:
            lines.append("  - missing journey")
            continue
        lines.append(
            f"  entry={j.get('entrypoint')} nodes={len(j.get('nodes') or [])} edges={len(j.get('edges') or [])}"
        )
        primary, supporting = split_primary_secondary(graph, j, max_primary_steps=40)
        if not primary and not supporting:
            lines.append("  - no explicit edges in journey")
            continue

        lines.append("  Primary Path:")
        for idx, (depth, e) in enumerate(primary, 1):
            ev = evidence_str(e)
            ev_file = ev.split(':', 1)[0]
            ok = file_exists(repo, ev_file)
            lines.append(
                f"    {idx:02d} [d{depth}] {e.get('kind')} {e.get('source')} -> {e.get('target')} @ {ev} {'OK' if ok else 'MISSING'}"
            )

        lines.append("  Supporting Branches:")
        for idx, (depth, e) in enumerate(supporting[:80], 1):
            ev = evidence_str(e)
            ev_file = ev.split(':', 1)[0]
            ok = file_exists(repo, ev_file)
            lines.append(
                f"    {idx:02d} [d{depth}] {e.get('kind')} {e.get('source')} -> {e.get('target')} @ {ev} {'OK' if ok else 'MISSING'}"
            )

        primary_fidelity_total += 1
        good_start = bool(primary) and primary[0][1].get("source") == j.get("entrypoint")
        flow_edges = sum(1 for _, e in primary if e.get("kind") in {"entry_invokes", "assoc_call", "method_call", "call"})
        if good_start and flow_edges >= max(1, len(primary) // 3):
            primary_fidelity_pass += 1

        lines.append(
            f"  primary_quality: start_ok={good_start} flow_edges={flow_edges}/{len(primary)} supporting={len(supporting)}"
        )

    lines.append("\n[6] Overall")
    correctness = 0
    total = 5
    correctness += 1 if audit.get("status") == "pass" else 0
    correctness += 1 if ia["mismatch_count"] == 0 else 0
    correctness += 1 if ex["pass_count"] >= max(1, ex["total"] - 1) else 0
    # clutter criterion: full graph can be dense, but if >150 nodes, must rely on journey focus
    has_journey_focus = len(journeys) > 0
    correctness += 1 if has_journey_focus else 0
    correctness += 1 if primary_fidelity_total and primary_fidelity_pass == primary_fidelity_total else 0
    lines.append(f"alignment_score={correctness}/{total}")
    lines.append(
        f"primary_path_fidelity={primary_fidelity_pass}/{primary_fidelity_total} focus_journeys passed control-flow-first checks"
    )
    lines.append("note: score favors control-flow-first journey readability over raw edge coverage.")

    return "\n".join(lines) + "\n"


def main() -> None:
    p = argparse.ArgumentParser(description="Text-mode RepoAtlas visualization audit")
    p.add_argument("--repo", default=".")
    p.add_argument("--graph", default="docs/repoatlas/graph.json")
    p.add_argument("--journeys", default="docs/repoatlas/journeys.json")
    p.add_argument("--view", default="docs/repoatlas/view_config.json")
    p.add_argument("--audit", default="docs/repoatlas/audit.json")
    p.add_argument("--focus-journey", action="append", default=[])
    p.add_argument("--out", default="docs/repoatlas/text_audit.txt")
    args = p.parse_args()

    repo = Path(args.repo).resolve()

    def rp(s: str) -> Path:
        pth = Path(s)
        return pth if pth.is_absolute() else (repo / pth)

    graph = load_json(rp(args.graph))
    journeys = load_json(rp(args.journeys))
    view = load_json(rp(args.view))
    audit = load_json(rp(args.audit))

    focus = args.focus_journey or ["entry:cli:collector", "entry:rpc:Query"]
    report = make_report(repo, graph, journeys, view, audit, focus)
    out = rp(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(report)
    print(report)
    print(f"written: {out}")


if __name__ == "__main__":
    main()
