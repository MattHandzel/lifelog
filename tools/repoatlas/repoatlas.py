#!/usr/bin/env python3
from __future__ import annotations

import argparse
import dataclasses
import json
import os
import re
from collections import defaultdict, deque
from pathlib import Path
from typing import Dict, Iterable, List, Optional, Set, Tuple

IGNORE_DIRS = {".git", "target", "node_modules", "worktrees", ".direnv", "dist", "build", ".next"}
SOURCE_EXTS = {".rs", ".py", ".ts", ".tsx", ".js", ".jsx"}


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
class Journey:
    name: str
    entrypoint: str
    narrative: str
    nodes: List[str]
    edges: List[str]


@dataclasses.dataclass
class Decision:
    claim: str
    evidence: List[Evidence]
    impact_surface: List[str]
    status: str = "TODO"


@dataclasses.dataclass
class Drift:
    level: str
    message: str
    source: str
    target: str
    evidence: List[Evidence]


def load_expected(path: Path) -> dict:
    if not path.exists():
        return {}
    if path.suffix.lower() == ".json":
        return json.loads(path.read_text())
    try:
        import yaml  # type: ignore

        return yaml.safe_load(path.read_text()) or {}
    except ModuleNotFoundError as exc:
        raise SystemExit(
            f"Expected architecture file '{path}' is YAML and PyYAML is not installed. "
            f"Use JSON or install pyyaml. ({exc})"
        )


def should_skip(path: Path) -> bool:
    return any(part in IGNORE_DIRS for part in path.parts)


def iter_files(root: Path, exts: Set[str], include_tests: bool) -> Iterable[Path]:
    for base, dirs, files in os.walk(root):
        base_path = Path(base)
        dirs[:] = [d for d in dirs if d not in IGNORE_DIRS]
        if should_skip(base_path):
            continue
        for file in files:
            path = base_path / file
            if path.suffix in exts:
                if not include_tests and is_test_path(rel(root, path)):
                    continue
                yield path


def is_test_path(relative_path: str) -> bool:
    rp = relative_path.lower()
    return (
        "/tests/" in f"/{rp}"
        or rp.endswith("_test.rs")
        or rp.endswith("_test.py")
        or rp.endswith(".spec.ts")
        or rp.endswith(".test.ts")
        or rp.endswith(".spec.tsx")
        or rp.endswith(".test.tsx")
    )


def rel(root: Path, path: Path) -> str:
    return str(path.relative_to(root)).replace("\\", "/")


def module_id(root: Path, path: Path) -> str:
    rp = rel(root, path)
    parts = rp.split("/")
    crate = parts[0] if len(parts) > 1 else "root"

    if "/src/" in rp:
        after_src = rp.split("/src/", 1)[1]
        mod_path = after_src
        if mod_path.endswith("main.rs") or mod_path.endswith("lib.rs"):
            mod = "root"
        elif mod_path.endswith("/mod.rs"):
            mod = mod_path[: -len("/mod.rs")].replace("/", "::")
            if not mod:
                mod = "root"
        elif mod_path == "mod.rs":
            mod = "root"
        else:
            mod = mod_path.rsplit(".", 1)[0].replace("/", "::")
    else:
        mod = rp.rsplit(".", 1)[0].replace("/", "::")

    return f"module:{crate}::{mod}"


def discover_entrypoints(root: Path, files: List[Path], nodes: Dict[str, Node], edges: List[Edge], file_to_module: Dict[str, str]) -> None:
    py_route_re = re.compile(r"@\w+\.(get|post|put|delete|patch)\(\s*[\"']([^\"']+)")
    js_route_re = re.compile(r"\b(app|router)\.(get|post|put|delete|patch)\(\s*[\"']([^\"']+)[\"']")

    for file in files:
        rp = rel(root, file)
        text = file.read_text(errors="ignore")
        mid = file_to_module[rp]

        if file.name in {"main.rs", "main.py", "__main__.py"}:
            entry_id = f"entry:cli:{rp.split('/')[0]}"
            nodes.setdefault(entry_id, Node(entry_id, "entrypoint", entry_id, rp))
            edges.append(Edge(entry_id, mid, "entry_invokes", [Evidence(rp, 1)], 0.95, ["entrypoint"]))

        if file.suffix == ".py":
            for i, line in enumerate(text.splitlines(), start=1):
                m = py_route_re.search(line)
                if not m:
                    continue
                route = m.group(2)
                entry_id = f"entry:http:{route}"
                nodes.setdefault(entry_id, Node(entry_id, "entrypoint", entry_id, rp))
                edges.append(Edge(entry_id, mid, "entry_invokes", [Evidence(rp, i)], 0.9, ["network", "entrypoint"]))

        if file.suffix in {".ts", ".tsx", ".js", ".jsx"}:
            for i, line in enumerate(text.splitlines(), start=1):
                m = js_route_re.search(line)
                if not m:
                    continue
                method = m.group(2).upper()
                route = m.group(3)
                entry_id = f"entry:http:{method} {route}"
                nodes.setdefault(entry_id, Node(entry_id, "entrypoint", entry_id, rp))
                edges.append(Edge(entry_id, mid, "entry_invokes", [Evidence(rp, i)], 0.9, ["network", "entrypoint"]))

    proto = root / "proto"
    if proto.exists():
        rpc_re = re.compile(r"\brpc\s+([A-Za-z0-9_]+)")
        for p in iter_files(proto, {".proto"}, include_tests=True):
            rp = rel(root, p)
            text = p.read_text(errors="ignore")
            for i, line in enumerate(text.splitlines(), start=1):
                m = rpc_re.search(line)
                if not m:
                    continue
                name = m.group(1)
                entry_id = f"entry:rpc:{name}"
                nodes.setdefault(entry_id, Node(entry_id, "entrypoint", entry_id, rp))
                grpc_target = next((m for m in file_to_module.values() if m.endswith("::grpc_service")), "module:server::grpc_service")
                edges.append(Edge(entry_id, grpc_target, "entry_invokes", [Evidence(rp, i)], 0.9, ["network", "entrypoint"]))


def discover_import_edges(root: Path, files: List[Path], edges: List[Edge], file_to_module: Dict[str, str], module_index: Dict[str, str]) -> None:
    py_import_re = re.compile(r"^(?:from\s+([\w\.]+)\s+import|import\s+([\w\.]+))")
    rs_use_re = re.compile(r"^use\s+crate::([a-zA-Z0-9_]+)")
    rs_qualified_re = re.compile(r"\bcrate::([a-zA-Z0-9_]+)::")
    ts_import_re = re.compile(r"from\s+[\"'](\.{1,2}/[^\"']+)[\"']")

    for file in files:
        rp = rel(root, file)
        src = file_to_module[rp]
        text = file.read_text(errors="ignore")
        for i, line in enumerate(text.splitlines(), start=1):
            t = line.strip()
            target = None
            if file.suffix == ".rs":
                m = rs_use_re.match(t)
                if m:
                    candidate = f"module:{rp.split('/')[0]}::{m.group(1)}"
                    target = candidate if candidate in module_index else None
                for qm in rs_qualified_re.finditer(t):
                    candidate = f"module:{rp.split('/')[0]}::{qm.group(1)}"
                    if candidate in module_index and candidate != src:
                        edges.append(Edge(src, candidate, "qualified_ref", [Evidence(rp, i)], 0.7, []))
            elif file.suffix == ".py":
                m = py_import_re.match(t)
                if m:
                    module_name = (m.group(1) or m.group(2) or "").replace(".", "::")
                    if module_name:
                        candidate = f"module:{rp.split('/')[0]}::{module_name}"
                        target = candidate if candidate in module_index else None
            elif file.suffix in {".ts", ".tsx", ".js", ".jsx"}:
                m = ts_import_re.search(t)
                if m:
                    imp = m.group(1)
                    base = Path(rp).parent
                    resolved = (base / imp).as_posix()
                    candidates = [resolved + ext for ext in [".ts", ".tsx", ".js", ".jsx"]] + [f"{resolved}/index.ts", f"{resolved}/index.tsx"]
                    for c in candidates:
                        c = str(Path(c)).replace("\\", "/")
                        if c in file_to_module:
                            target = file_to_module[c]
                            break

            if target and target != src:
                edges.append(Edge(src, target, "import", [Evidence(rp, i)], 0.95, []))


def discover_call_edges(root: Path, files: List[Path], edges: List[Edge], file_to_module: Dict[str, str]) -> None:
    def_patterns = {
        ".rs": re.compile(r"(?:pub\s+)?(?:async\s+)?fn\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\("),
        ".py": re.compile(r"def\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\("),
        ".ts": re.compile(r"(?:export\s+)?(?:async\s+)?function\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\("),
        ".tsx": re.compile(r"(?:export\s+)?(?:async\s+)?function\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\("),
        ".js": re.compile(r"function\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\("),
        ".jsx": re.compile(r"function\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\("),
    }
    call_re = re.compile(r"\b([a-zA-Z_][a-zA-Z0-9_]*)\s*\(")
    ignore = {"if", "for", "while", "match", "Some", "Ok", "Err", "new", "from", "into", "clone", "default"}

    owners: Dict[str, Set[str]] = defaultdict(set)
    for file in files:
        rp = rel(root, file)
        mod = file_to_module[rp]
        text = file.read_text(errors="ignore")
        pat = def_patterns.get(file.suffix)
        if not pat:
            continue
        for line in text.splitlines():
            m = pat.search(line)
            if m:
                owners[m.group(1)].add(mod)

    unique_owner = {name: list(mods)[0] for name, mods in owners.items() if len(mods) == 1}

    for file in files:
        rp = rel(root, file)
        src = file_to_module[rp]
        text = file.read_text(errors="ignore")
        crate_name = rp.split("/")[0]
        rust_use_aliases: Dict[str, str] = {}
        rust_var_modules: Dict[str, str] = {}

        if file.suffix == ".rs":
            rust_use_aliases = rust_alias_map(rp, text, file_to_module)
            rust_var_modules = rust_var_module_map(text, rust_use_aliases, crate_name, file_to_module)

            assoc_call_re = re.compile(r"\b([A-Za-z_][A-Za-z0-9_:]*)::([A-Za-z_][A-Za-z0-9_]*)\s*\(")
            method_call_re = re.compile(r"\b([a-z_][a-zA-Z0-9_]*)\.([A-Za-z_][A-Za-z0-9_]*)\s*\(")

        for i, line in enumerate(text.splitlines(), start=1):
            for m in call_re.finditer(line):
                name = m.group(1)
                if name in ignore:
                    continue
                idx = m.start(1)
                if idx > 0 and line[idx - 1] in {".", ":"}:
                    continue
                target = unique_owner.get(name)
                if target and target != src:
                    edges.append(Edge(src, target, "call", [Evidence(rp, i)], 0.45, ["best-effort"]))

            if file.suffix == ".rs":
                for m in assoc_call_re.finditer(line):
                    path = m.group(1)
                    if path in {"if", "for", "while", "match", "loop"}:
                        continue
                    target = resolve_rust_path_to_module(path, crate_name, rust_use_aliases, file_to_module)
                    if target and target != src:
                        edges.append(Edge(src, target, "assoc_call", [Evidence(rp, i)], 0.8, ["runtime-flow"]))

                for m in method_call_re.finditer(line):
                    var = m.group(1)
                    target = rust_var_modules.get(var)
                    if target and target != src:
                        edges.append(Edge(src, target, "method_call", [Evidence(rp, i)], 0.75, ["runtime-flow"]))


def resolve_rust_path_to_module(
    path: str,
    crate_name: str,
    alias_to_module: Dict[str, str],
    file_to_module: Dict[str, str],
) -> Optional[str]:
    if path in alias_to_module:
        return alias_to_module[path]

    parts = path.split("::")
    if not parts:
        return None

    base_parts: List[str]
    if parts[0] == "crate":
        base_parts = parts[1:]
    elif parts[0] == crate_name:
        base_parts = parts[1:]
    elif parts[0].startswith("lifelog_") and parts[0].endswith(crate_name):
        base_parts = parts[1:]
    else:
        return None

    known = set(file_to_module.values())
    for k in range(len(base_parts), 0, -1):
        candidate = f"module:{crate_name}::{'::'.join(base_parts[:k])}"
        if candidate in known:
            return candidate

    root_candidate = f"module:{crate_name}::root"
    if root_candidate in known:
        return root_candidate
    return None


def rust_alias_map(rp: str, text: str, file_to_module: Dict[str, str]) -> Dict[str, str]:
    crate_name = rp.split("/")[0]
    known = set(file_to_module.values())
    alias_map: Dict[str, str] = {}

    use_re = re.compile(r"^\s*use\s+([^;]+);")
    for line in text.splitlines():
        m = use_re.match(line)
        if not m:
            continue
        raw = m.group(1).strip()
        if "{" in raw or "}" in raw:
            continue
        if " as " in raw:
            path, alias = [x.strip() for x in raw.split(" as ", 1)]
        else:
            path = raw
            alias = path.split("::")[-1]

        target = resolve_rust_path_to_module(path, crate_name, {}, file_to_module)
        if target and target in known:
            alias_map[alias] = target

    return alias_map


def rust_var_module_map(
    text: str,
    alias_to_module: Dict[str, str],
    crate_name: str,
    file_to_module: Dict[str, str],
) -> Dict[str, str]:
    var_map: Dict[str, str] = {}

    let_new_re = re.compile(r"\blet\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*=.*?\b([A-Za-z_][A-Za-z0-9_:]*)::new\s*\(")
    let_struct_re = re.compile(r"\blet\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*([A-Za-z_][A-Za-z0-9_:]*)\s*\{")
    let_clone_re = re.compile(r"\blet\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*([a-zA-Z_][a-zA-Z0-9_]*)\.clone\s*\(")

    for line in text.splitlines():
        m_clone = let_clone_re.search(line)
        if m_clone:
            var_map[m_clone.group(1)] = var_map.get(m_clone.group(2), "")
            if not var_map[m_clone.group(1)]:
                var_map.pop(m_clone.group(1), None)

        m_new = let_new_re.search(line)
        if m_new:
            var, ctor = m_new.group(1), m_new.group(2)
            target = resolve_rust_path_to_module(ctor, crate_name, alias_to_module, file_to_module)
            if target:
                var_map[var] = target

        m_struct = let_struct_re.search(line)
        if m_struct:
            var, typ = m_struct.group(1), m_struct.group(2)
            target = resolve_rust_path_to_module(typ, crate_name, alias_to_module, file_to_module)
            if target:
                var_map[var] = target

    return var_map


def discover_resources(
    root: Path,
    files: List[Path],
    nodes: Dict[str, Node],
    edges: List[Edge],
    file_to_module: Dict[str, str],
    include_tests: bool,
) -> None:
    table_re = re.compile(r"(?i)(?:from|join|into|update|insert\s+into|delete\s+from|define\s+table|create\s+table)\s+([a-zA-Z_][a-zA-Z0-9_]*)")
    surreal_re = re.compile(r"table\(\s*[\"']([a-zA-Z_][a-zA-Z0-9_]*)[\"']\s*\)")
    queue_re = re.compile(r"(?i)(?:topic|queue)[:=\s\"']+([a-zA-Z0-9_.-]+)")
    sqlish_re = re.compile(r"(?i)\b(select|insert|update|delete|from|join|create\s+table|define\s+table)\b")
    stopwords = {"if", "the", "a", "an", "and", "or", "on", "to", "from", "table", "error", "server"}

    scan = files + list(iter_files(root, {".sql"}, include_tests))
    for file in scan:
        rp = rel(root, file)
        text = file.read_text(errors="ignore")
        src = file_to_module.get(rp)
        for i, line in enumerate(text.splitlines(), start=1):
            for m in surreal_re.finditer(line):
                table = m.group(1).lower()
                if len(table) < 3 or table in stopwords:
                    continue
                nid = f"resource:table:{table}"
                nodes.setdefault(nid, Node(nid, "resource", nid, rp))
                if src:
                    edges.append(Edge(src, nid, "resource_access", [Evidence(rp, i)], 0.85, ["persistence"]))

            if file.suffix != ".sql" and not sqlish_re.search(line):
                continue

            for m in table_re.finditer(line):
                table = m.group(1).lower()
                if len(table) < 3 or table in stopwords:
                    continue
                nid = f"resource:table:{table}"
                nodes.setdefault(nid, Node(nid, "resource", nid, rp))
                if src:
                    edges.append(Edge(src, nid, "resource_access", [Evidence(rp, i)], 0.75, ["persistence"]))
            for m in queue_re.finditer(line):
                name = m.group(1).lower()
                nid = f"resource:queue:{name}"
                nodes.setdefault(nid, Node(nid, "resource", nid, rp))
                if src:
                    edges.append(Edge(src, nid, "resource_access", [Evidence(rp, i)], 0.6, ["async"]))


def dedup_edges(edges: List[Edge]) -> List[Edge]:
    seen: Set[Tuple[str, str, str, str, int]] = set()
    out: List[Edge] = []
    for edge in edges:
        ev = edge.evidence[0] if edge.evidence else Evidence("", 0)
        key = (edge.kind, edge.source, edge.target, ev.file, ev.line)
        if key not in seen:
            seen.add(key)
            out.append(edge)
    return out


def assign_boundaries(expected: dict, root: Path, nodes: Dict[str, Node]) -> None:
    boundaries = expected.get("boundaries", [])
    for node in nodes.values():
        if not node.file:
            continue
        p = rel(root, root / node.file)
        for b in boundaries:
            name = b.get("name")
            patterns = b.get("includes", [])
            if any(path_match(p, pat) for pat in patterns):
                node.boundary = name
                break


def path_match(path: str, pattern: str) -> bool:
    regex = "^" + re.escape(pattern).replace(r"\*\*", ".*").replace(r"\*", "[^/]*") + "$"
    return re.match(regex, path) is not None


def tag_boundary_crossings(nodes: Dict[str, Node], edges: List[Edge]) -> None:
    for edge in edges:
        s = nodes.get(edge.source)
        t = nodes.get(edge.target)
        if not s or not t:
            continue
        if s.boundary and t.boundary and s.boundary != t.boundary and "boundary-crossing" not in edge.tags:
            edge.tags.append("boundary-crossing")


def build_journeys(nodes: Dict[str, Node], edges: List[Edge], max_hops: int) -> List[Journey]:
    by_src: Dict[str, List[Edge]] = defaultdict(list)
    for edge in edges:
        by_src[edge.source].append(edge)

    journeys: List[Journey] = []
    for node in nodes.values():
        if node.kind != "entrypoint":
            continue
        seen = {node.id}
        eq = deque([(node.id, 0)])
        j_edges: Set[str] = set()
        while eq:
            cur, depth = eq.popleft()
            if depth >= max_hops:
                continue
            for e in by_src.get(cur, []):
                j_edges.add(f"{e.kind}:{e.source}->{e.target}")
                if e.target not in seen:
                    seen.add(e.target)
                    eq.append((e.target, depth + 1))
        journeys.append(
            Journey(
                name=node.id,
                entrypoint=node.id,
                narrative=f"{node.id} expands to {len(seen)} nodes within {max_hops} hops",
                nodes=sorted(seen),
                edges=sorted(j_edges),
            )
        )
    return journeys


def build_decisions(edges: List[Edge], journeys: List[Journey]) -> List[Decision]:
    out: Dict[str, Decision] = {}

    for e in edges:
        if "boundary-crossing" in e.tags:
            claim = f"Cross-boundary dependency: {e.source} -> {e.target}"
            out.setdefault(claim, Decision(claim, [], [e.source, e.target]))
            out[claim].evidence.extend(e.evidence)

        if e.kind == "resource_access":
            claim = f"{e.source} accesses {e.target}"
            out.setdefault(claim, Decision(claim, [], [e.source, e.target]))
            out[claim].evidence.extend(e.evidence)

    for j in journeys:
        resources = [n for n in j.nodes if n.startswith("resource:")]
        if resources:
            claim = f"{j.entrypoint} touches {len(resources)} resources"
            out.setdefault(claim, Decision(claim, [], resources))

    return sorted(out.values(), key=lambda d: d.claim)


def detect_drift(expected: dict, nodes: Dict[str, Node], edges: List[Edge], journeys: List[Journey]) -> List[Drift]:
    allow: Set[Tuple[str, str]] = set()
    forbid: Set[Tuple[str, str]] = set()

    for rule in expected.get("rules", []):
        for item in rule.get("allow", []):
            pair = parse_rule_pair(item)
            if pair:
                allow.add(pair)
        for item in rule.get("forbid", []):
            pair = parse_rule_pair(item)
            if pair:
                forbid.add(pair)

    out: Dict[Tuple[str, str, str], Drift] = {}
    for e in edges:
        s = nodes.get(e.source)
        t = nodes.get(e.target)
        if not s or not t:
            continue
        if not s.boundary or not t.boundary or s.boundary == t.boundary:
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

    by_entry = {j.entrypoint: j for j in journeys}
    for req in expected.get("required_journeys", []):
        entry = req.get("entrypoint", "")
        found = next((j for eid, j in by_entry.items() if entry in eid), None)
        if not found:
            key = ("coverage-gap", entry, "")
            out[key] = Drift("coverage-gap", f"Required journey '{req.get('name', entry)}' not found", entry, "", [])
            continue
        for must in req.get("must_include", []):
            if must not in found.nodes:
                key = ("coverage-gap", found.entrypoint, must)
                out[key] = Drift(
                    "coverage-gap",
                    f"Required journey '{req.get('name', found.entrypoint)}' missing '{must}'",
                    found.entrypoint,
                    must,
                    [],
                )

    return sorted(out.values(), key=lambda d: (d.level, d.message, d.source, d.target))


def parse_rule_pair(value: str) -> Optional[Tuple[str, str]]:
    if "->" not in value:
        return None
    parts = [p.strip() for p in value.split("->", 1)]
    if len(parts) != 2 or not parts[0] or not parts[1]:
        return None
    return (parts[0], parts[1])


def render_markdown(nodes: Dict[str, Node], edges: List[Edge], journeys: List[Journey], decisions: List[Decision], drift: List[Drift]) -> str:
    lines: List[str] = []
    lines.append("# RepoAtlas Report")
    lines.append("")
    lines.append(f"- Nodes: {len(nodes)}")
    lines.append(f"- Edges: {len(edges)}")
    lines.append(f"- Journeys: {len(journeys)}")
    lines.append(f"- Decisions: {len(decisions)}")
    lines.append(f"- Drift findings: {len(drift)}")
    lines.append("")
    lines.append("## Journeys")
    lines.append("")
    for j in journeys:
        lines.append(f"### {j.name}")
        lines.append(f"- Entrypoint: `{j.entrypoint}`")
        lines.append(f"- Narrative: {j.narrative}")
        lines.append(f"- Nodes in view: {len(j.nodes)}")
        lines.append("")
    lines.append("## Drift")
    lines.append("")
    if not drift:
        lines.append("No drift findings.")
    for d in drift:
        lines.append(f"- [{d.level}] {d.message} (`{d.source}` -> `{d.target}`)")
    lines.append("")
    lines.append("## Decisions")
    lines.append("")
    for d in decisions[:200]:
        lines.append(f"- [{d.status}] {d.claim}")
    lines.append("")
    return "\n".join(lines)


def dump_json(path: Path, obj) -> None:
    def encoder(o):
        if dataclasses.is_dataclass(o):
            return dataclasses.asdict(o)
        raise TypeError(f"Cannot serialize {type(o)}")

    path.write_text(json.dumps(obj, indent=2, default=encoder))


def run(repo: Path, expected_file: Path, out: Path, max_hops: int, include_tests: bool) -> None:
    expected = load_expected(expected_file)

    src_files = sorted(iter_files(repo, SOURCE_EXTS, include_tests))
    nodes: Dict[str, Node] = {}
    edges: List[Edge] = []
    file_to_module: Dict[str, str] = {}

    for file in src_files:
        rp = rel(repo, file)
        mid = module_id(repo, file)
        file_to_module[rp] = mid
        nodes.setdefault(mid, Node(mid, "module", mid, rp))

    module_index = {n.id: n.file or "" for n in nodes.values()}

    discover_entrypoints(repo, src_files, nodes, edges, file_to_module)
    discover_import_edges(repo, src_files, edges, file_to_module, module_index)
    discover_call_edges(repo, src_files, edges, file_to_module)
    discover_resources(repo, src_files, nodes, edges, file_to_module, include_tests)

    edges = dedup_edges(edges)
    assign_boundaries(expected, repo, nodes)
    tag_boundary_crossings(nodes, edges)

    journeys = build_journeys(nodes, edges, max_hops)
    decisions = build_decisions(edges, journeys)
    drift = detect_drift(expected, nodes, edges, journeys)

    out.mkdir(parents=True, exist_ok=True)
    dump_json(out / "graph.json", {"nodes": list(nodes.values()), "edges": edges})
    dump_json(out / "journeys.json", journeys)
    dump_json(out / "decisions.json", decisions)
    dump_json(out / "drift.json", drift)
    (out / "report.md").write_text(render_markdown(nodes, edges, journeys, decisions, drift))

    print(f"RepoAtlas report generated in {out}")


def main() -> None:
    parser = argparse.ArgumentParser(description="RepoAtlas: code-first journey explorer + decision auditor + drift detector")
    parser.add_argument("--repo", default=".", help="Path to repository root")
    parser.add_argument("--expected", default="repoatlas/expected_arch.json", help="Path to expected architecture JSON/YAML")
    parser.add_argument("--out", default="docs/repoatlas", help="Output directory")
    parser.add_argument("--max-hops", type=int, default=4, help="Journey expansion depth")
    parser.add_argument("--include-tests", action="store_true", help="Include test files in analysis")
    args = parser.parse_args()

    repo = Path(args.repo).resolve()
    expected = Path(args.expected)
    if not expected.is_absolute():
        expected = (repo / expected).resolve()
    out = Path(args.out)
    if not out.is_absolute():
        out = (repo / out).resolve()

    run(repo, expected, out, args.max_hops, args.include_tests)


if __name__ == "__main__":
    main()
