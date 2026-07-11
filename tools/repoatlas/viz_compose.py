#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
import shlex
import subprocess
from pathlib import Path
from typing import Any, Dict, List, Optional


def load_json(path: Path) -> Any:
    return json.loads(path.read_text())


def classify_icon(node: Dict[str, Any]) -> str:
    nid = node.get("id", "")
    kind = node.get("kind", "")
    file = (node.get("file") or "").lower()
    boundary = (node.get("boundary") or "").lower()

    if kind == "resource":
        if ":table:" in nid:
            return "database"
        if ":queue:" in nid:
            return "queue"
        return "resource"

    if kind == "entrypoint":
        if nid.startswith("entry:rpc:"):
            return "rpc"
        if nid.startswith("entry:http:"):
            return "http"
        if nid.startswith("entry:cli:"):
            return "cli"
        return "entry"

    if kind == "module":
        # Prefer explicit boundary over substring heuristics to avoid misclassification
        # (e.g. module:common::server_config should remain library/common).
        boundary_map = {
            "collector": "collector",
            "interface": "ui",
            "server": "server",
            "common": "library",
            "proto": "network",
            "tools": "library",
        }
        if boundary in boundary_map:
            return boundary_map[boundary]

        if "grpc" in nid or "grpc" in file:
            return "network"
        if "query" in nid or "planner" in nid or "executor" in nid:
            return "query"
        if "db" in nid or "schema" in nid or "postgres" in nid or "surreal" in nid:
            return "storage"
        if "collector" in nid:
            return "collector"
        if "interface" in nid:
            return "ui"
        if "server" in nid:
            return "server"
        if "common" in nid:
            return "library"

    return "default"


def icon_style(icon: str, kind: str) -> Dict[str, Any]:
    palette = {
        "cli": {"shape": "box", "color": "#57d3f3"},
        "rpc": {"shape": "diamond", "color": "#5cd7c8"},
        "http": {"shape": "diamond", "color": "#4bbcf0"},
        "entry": {"shape": "diamond", "color": "#57d3f3"},
        "database": {"shape": "database", "color": "#ffbf69"},
        "queue": {"shape": "triangle", "color": "#f9a66c"},
        "resource": {"shape": "box", "color": "#ffc861"},
        "network": {"shape": "dot", "color": "#6fd3ff"},
        "query": {"shape": "dot", "color": "#c1e36f"},
        "storage": {"shape": "dot", "color": "#ffda7a"},
        "collector": {"shape": "dot", "color": "#71d59f"},
        "ui": {"shape": "dot", "color": "#b28cff"},
        "server": {"shape": "dot", "color": "#69c0ff"},
        "library": {"shape": "dot", "color": "#9ac0d8"},
        "default": {"shape": "dot", "color": "#97a4bb"},
    }
    base = palette.get(icon, palette["default"]).copy()
    if kind == "entrypoint":
        base["borderWidth"] = 2
    elif kind == "resource":
        base["borderWidth"] = 1
    else:
        base["borderWidth"] = 1
    return base


def deterministic_view_config(graph: Dict[str, Any], journeys: List[Dict[str, Any]], drift: List[Dict[str, Any]]) -> Dict[str, Any]:
    node_styles: Dict[str, Dict[str, Any]] = {}
    groups: Dict[str, List[str]] = {}

    for n in graph.get("nodes", []):
        nid = n["id"]
        icon = classify_icon(n)
        node_styles[nid] = {"icon": icon, **icon_style(icon, n.get("kind", ""))}

        boundary = n.get("boundary") or "unscoped"
        groups.setdefault(boundary, []).append(nid)

    hotspots = []
    by_level = {"hard": [], "soft": [], "coverage-gap": []}
    for d in drift:
        by_level.setdefault(d.get("level", "soft"), []).append(d)

    for lvl in ["hard", "soft", "coverage-gap"]:
        for d in by_level.get(lvl, [])[:30]:
            hotspots.append({
                "level": lvl,
                "source": d.get("source", ""),
                "target": d.get("target", ""),
                "message": d.get("message", ""),
            })

    journey_focus = [j.get("name") for j in journeys[:20]]

    return {
        "version": 1,
        "strategy": "static-graph + rule-based-icons + llm-layout-hints",
        "node_styles": node_styles,
        "boundary_groups": groups,
        "hotspots": hotspots,
        "suggested_journey_order": journey_focus,
        "llm": {
            "enabled": False,
            "status": "pending",
            "notes": "No LLM overlay applied yet",
        },
    }


def run_agent(agent: str, model: str, prompt: str, timeout_seconds: int) -> str:
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

    proc = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout_seconds)
    if proc.returncode != 0:
        raise RuntimeError(
            f"Visualization agent failed ({proc.returncode}) with command: {' '.join(shlex.quote(x) for x in cmd)}\n"
            f"stderr: {proc.stderr[:1200]}"
        )
    return proc.stdout.strip() or proc.stderr.strip()


def extract_json(text: str) -> Dict[str, Any]:
    m = re.search(r"```json\s*(\{.*\})\s*```", text, re.DOTALL)
    if m:
        return json.loads(m.group(1))
    start = text.find("{")
    end = text.rfind("}")
    if start >= 0 and end > start:
        return json.loads(text[start : end + 1])
    raise ValueError("No JSON object found in visualization-agent output")


def llm_overlay(
    agent: str,
    model: str,
    graph: Dict[str, Any],
    journeys: List[Dict[str, Any]],
    drift: List[Dict[str, Any]],
    decisions: List[Dict[str, Any]],
    timeout_seconds: int,
) -> Dict[str, Any]:
    compact = {
        "node_count": len(graph.get("nodes", [])),
        "edge_count": len(graph.get("edges", [])),
        "journeys": [
            {"name": j.get("name"), "entrypoint": j.get("entrypoint"), "node_count": len(j.get("nodes", []))}
            for j in journeys[:20]
        ],
        "top_drift": drift[:40],
        "top_decisions": decisions[:60],
    }

    prompt = (
        "You are a visualization composition agent.\n"
        "Do not infer new architecture facts. Only propose view-layer hints from provided static analysis summary.\n"
        "Return JSON only with keys:\n"
        "{\n"
        "  \"layout\": {\"mode\": \"clustered|force\", \"focus_boundaries\": [..], \"focus_nodes\": [..]},\n"
        "  \"legend\": [{\"icon\":\"...\",\"label\":\"...\",\"meaning\":\"...\"}],\n"
        "  \"drift_story\": [{\"title\":\"...\",\"nodes\":[..],\"edges\":[..]}],\n"
        "  \"journey_presets\": [{\"name\":\"...\",\"entrypoint\":\"...\",\"why\":\"...\"}]\n"
        "}\n"
        "Static summary:\n"
        f"{json.dumps(compact, indent=2)}"
    )

    raw = run_agent(agent, model, prompt, timeout_seconds)
    return extract_json(raw)


def merge_overlay(base: Dict[str, Any], overlay: Dict[str, Any]) -> Dict[str, Any]:
    merged = dict(base)
    merged["layout"] = overlay.get("layout", {"mode": "force", "focus_boundaries": [], "focus_nodes": []})
    merged["legend"] = overlay.get("legend", [])
    merged["drift_story"] = overlay.get("drift_story", [])
    merged["journey_presets"] = overlay.get("journey_presets", [])
    merged.setdefault("llm", {})
    merged["llm"]["enabled"] = True
    merged["llm"]["status"] = "ok"
    return merged


def main() -> None:
    p = argparse.ArgumentParser(description="Compose visualization config from static RepoAtlas outputs")
    p.add_argument("--repo", default=".")
    p.add_argument("--graph", default="docs/repoatlas/graph.json")
    p.add_argument("--journeys", default="docs/repoatlas/journeys.json")
    p.add_argument("--decisions", default="docs/repoatlas/decisions.json")
    p.add_argument("--drift", default="docs/repoatlas/drift.json")
    p.add_argument("--out", default="docs/repoatlas/view_config.json")
    p.add_argument("--agent", default="codex", choices=["codex", "gemini", "claude"])
    p.add_argument("--model", default="")
    p.add_argument("--timeout-seconds", type=int, default=180)
    p.add_argument("--require-llm", action="store_true", help="Fail if LLM overlay generation fails")
    args = p.parse_args()

    repo = Path(args.repo).resolve()

    def resolve_path(raw: str) -> Path:
        pth = Path(raw)
        return pth if pth.is_absolute() else (repo / pth)

    graph = load_json(resolve_path(args.graph))
    journeys = load_json(resolve_path(args.journeys))
    decisions = load_json(resolve_path(args.decisions))
    drift = load_json(resolve_path(args.drift))

    base = deterministic_view_config(graph, journeys, drift)

    try:
        overlay = llm_overlay(args.agent, args.model, graph, journeys, drift, decisions, args.timeout_seconds)
        view = merge_overlay(base, overlay)
    except Exception as exc:
        if args.require_llm:
            raise SystemExit(f"Visualization LLM overlay failed: {exc}")
        err = str(exc).replace("\n", " ").strip()
        view = dict(base)
        view["llm"] = {
            "enabled": False,
            "status": "failed",
            "notes": f"Using deterministic rules only; llm_error={err[:220]}",
        }

    out = resolve_path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(view, indent=2))
    print(f"Visualization config written to {out}")


if __name__ == "__main__":
    main()
