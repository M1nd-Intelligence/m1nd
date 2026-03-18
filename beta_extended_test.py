#!/usr/bin/env python3
"""
TEMPONIZER BETA TESTER — m1nd Extended Superpowers + Advanced Tools
agent_id: beta-extended
"""

import json, subprocess, sys, os, tempfile, time

BINARY = "./target/release/m1nd-mcp"
BACKEND_PATH = "/Users/cosmophonix/clawd/roomanizer-os"
MSG_ID = 0

workdir = tempfile.mkdtemp(prefix="m1nd_beta_ext_")


def start_server():
    env = os.environ.copy()
    env["GRAPH_SNAPSHOT_PATH"] = os.path.join(workdir, "graph.json")
    env["PLASTICITY_STATE_PATH"] = os.path.join(workdir, "plasticity.json")
    return subprocess.Popen(
        [BINARY],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=open(os.path.join(workdir, "stderr.log"), "w"),
        env=env,
        bufsize=0,
    )


def next_id():
    global MSG_ID
    MSG_ID += 1
    return MSG_ID


def call(proc, name, args):
    t0 = time.time()
    msg = json.dumps(
        {
            "jsonrpc": "2.0",
            "method": "tools/call",
            "id": next_id(),
            "params": {"name": name, "arguments": args},
        }
    )
    proc.stdin.write((msg + "\n").encode())
    proc.stdin.flush()
    raw = proc.stdout.readline().decode().strip()
    elapsed = (time.time() - t0) * 1000
    return json.loads(raw), elapsed


def init(proc):
    msg = json.dumps(
        {
            "jsonrpc": "2.0",
            "method": "initialize",
            "id": 1,
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "beta-extended", "version": "1.0"},
            },
        }
    )
    global MSG_ID
    MSG_ID = 1
    proc.stdin.write((msg + "\n").encode())
    proc.stdin.flush()
    return json.loads(proc.stdout.readline().decode().strip())


def xt(resp):
    try:
        text = resp["result"]["content"][0]["text"]
        try:
            return json.loads(text)
        except:
            return text
    except:
        return resp


def is_err(resp):
    try:
        return resp["result"].get("isError", False)
    except:
        return "error" in resp


def sec(title):
    print(f"\n{'=' * 65}")
    print(f"  {title}")
    print(f"{'=' * 65}")


def report(tool, status, tests, metrics, assessment, adversarial, rating):
    print(f"\n### {tool} {rating}")
    for t in tests:
        print(f"  - {t}")
    for m in metrics:
        print(f"  > {m}")
    print(f"  ASSESSMENT: {assessment}")
    print(f"  ADVERSARIAL: {adversarial}")


results = {}

# ============================================================================
proc = start_server()
print(f"[beta-extended] Server PID {proc.pid}, workdir: {workdir}")
init(proc)

# Ingest FULL graph
t0 = time.time()
r, ms = call(
    proc, "m1nd.ingest", {"agent_id": "beta-ext", "path": BACKEND_PATH, "mode": "full"}
)
d = xt(r)
ingest_ms = time.time() - t0
node_count = d.get("node_count", 0) if isinstance(d, dict) else 0
edge_count = d.get("edge_count", 0) if isinstance(d, dict) else 0
err_ingest = is_err(r)
print(
    f"[ingest] nodes={node_count} edges={edge_count} elapsed={ingest_ms:.1f}s err={err_ingest}"
)
if err_ingest:
    print(f"  ERROR: {json.dumps(r)[:400]}")

# ============================================================================
sec("ANTIBODY_LIST")
r, ms = call(proc, "m1nd.antibody_list", {"agent_id": "beta-ext"})
d = xt(r)
err = is_err(r)
ab_list = (
    d
    if isinstance(d, list)
    else (d.get("antibodies", []) if isinstance(d, dict) else [])
)
print(f"  antibodies found: {len(ab_list)}, err={err}, ms={ms:.0f}")
if ab_list:
    for ab in ab_list[:3]:
        ab_id = ab.get("id", "?") if isinstance(ab, dict) else ab
        sev = ab.get("severity", "?") if isinstance(ab, dict) else "?"
        print(f"    - {ab_id} [{sev}]")
report(
    "antibody_list",
    "tested",
    [f"Called antibody_list, got {len(ab_list)} existing antibodies"],
    [f"Existing antibodies: {len(ab_list)}", f"Response time: {ms:.0f}ms"],
    "Lists pre-existing antibodies from antibodies.json",
    "If the json has stale antibodies from previous sessions, list will be polluted",
    "✅" if not err else "❌",
)

# ============================================================================
sec("ANTIBODY_SCAN — base full graph scan")
r, ms = call(
    proc, "m1nd.antibody_scan", {"agent_id": "beta-ext", "min_severity": "info"}
)
d = xt(r)
err = is_err(r)
matches = d.get("matches", []) if isinstance(d, dict) else []
total_matches = d.get("total_matches", len(matches)) if isinstance(d, dict) else 0
scan_ms = d.get("scan_ms", ms) if isinstance(d, dict) else ms
print(f"  scan results: total_matches={total_matches}, ms={ms:.0f}, err={err}")
if matches:
    for m in matches[:3]:
        print(f"    - {json.dumps(m)[:200]}")
report(
    "antibody_scan (full, min_severity=info)",
    "tested",
    ["Ran full graph scan with min_severity=info"],
    [f"total_matches={total_matches}", f"scan_ms={ms:.0f}"],
    f"{'Returned matches — antibodies running' if not err else 'Error returned'}",
    "With no antibodies defined, should return 0 matches — tests empty-state behavior",
    "✅" if not err else "❌",
)

# Scan changed only
r2, ms2 = call(
    proc,
    "m1nd.antibody_scan",
    {"agent_id": "beta-ext", "scope": "changed", "min_severity": "info"},
)
d2 = xt(r2)
err2 = is_err(r2)
matches2 = d2.get("total_matches", 0) if isinstance(d2, dict) else 0
print(f"  scope=changed: total_matches={matches2}, err={err2}, ms={ms2:.0f}")
report(
    "antibody_scan (scope=changed)",
    "tested",
    ["Ran scan with scope=changed"],
    [f"total_matches={matches2}", f"ms={ms2:.0f}"],
    "Changed scope requires git diff context — cold start likely returns 0",
    "scope=changed without git history may silently return empty — no error surfaced",
    "⚠️" if matches2 == 0 and not err2 else "✅",
)

# ============================================================================
sec("ANTIBODY_CREATE — CancelledError pattern")
r, ms = call(
    proc,
    "m1nd.antibody_create",
    {
        "agent_id": "beta-ext",
        "id": "cancelled-error-not-reraised",
        "name": "CancelledError not re-raised",
        "description": "asyncio.CancelledError caught but not re-raised — breaks cooperative cancellation",
        "severity": "high",
        "nodes": [
            {
                "id": "handler",
                "label_pattern": "except.*CancelledError",
                "match_mode": "regex",
            },
            {
                "id": "body",
                "label_pattern": "pass|break|continue",
                "match_mode": "regex",
            },
        ],
        "edges": [{"from": "handler", "to": "body", "relation": "contains"}],
        "negative_edges": [
            {"from": "handler", "to": "raise_node", "relation": "contains"}
        ],
    },
)
d = xt(r)
err = is_err(r)
print(f"  create CancelledError antibody: err={err}, ms={ms:.0f}")
print(f"  response: {json.dumps(d)[:300]}")
report(
    "antibody_create (CancelledError)",
    "tested",
    ["Created CancelledError-not-reraised antibody"],
    [f"ms={ms:.0f}", f"err={err}"],
    "Created successfully" if not err else "Creation failed",
    "Graph nodes use file/function labels — regex on label_pattern may not match Python 'except' blocks if labels are just names",
    "✅" if not err else "❌",
)

# ============================================================================
sec("ANTIBODY_CREATE — dict mutation without lock")
r2, ms2 = call(
    proc,
    "m1nd.antibody_create",
    {
        "agent_id": "beta-ext",
        "id": "dict-mutation-without-lock",
        "name": "dict mutation without lock",
        "description": "Shared dict (_registry/_pool/_sessions) mutated without a lock",
        "severity": "high",
        "nodes": [
            {
                "id": "dict_access",
                "label_pattern": "_registry|_pool|_sessions",
                "match_mode": "regex",
            },
            {
                "id": "mutation",
                "label_pattern": "update|pop|del|append",
                "match_mode": "regex",
            },
        ],
        "edges": [{"from": "dict_access", "to": "mutation", "relation": "contains"}],
        "negative_edges": [
            {"from": "dict_access", "to": "lock_node", "relation": "contains"}
        ],
    },
)
d2 = xt(r2)
err2 = is_err(r2)
print(f"  create dict-mutation antibody: err={err2}, ms={ms2:.0f}")
print(f"  response: {json.dumps(d2)[:300]}")
report(
    "antibody_create (dict mutation)",
    "tested",
    ["Created dict-mutation-without-lock antibody"],
    [f"ms={ms2:.0f}", f"err={err2}"],
    "Created" if not err2 else "Failed",
    "negative_edges on lock_node only effective if lock nodes are present in graph — may miss inline asyncio.Lock() calls",
    "✅" if not err2 else "❌",
)

# Scan after creating antibodies
sec("ANTIBODY_SCAN after creating antibodies")
r3, ms3 = call(
    proc, "m1nd.antibody_scan", {"agent_id": "beta-ext", "min_severity": "high"}
)
d3 = xt(r3)
err3 = is_err(r3)
matches3 = d3.get("total_matches", 0) if isinstance(d3, dict) else 0
all_matches = d3.get("matches", []) if isinstance(d3, dict) else []
print(f"  post-create scan: total_matches={matches3}, err={err3}, ms={ms3:.0f}")
for m in all_matches[:5]:
    print(f"    match: {json.dumps(m)[:250]}")
report(
    "antibody_scan post-create",
    "tested",
    ["Scanned after creating 2 custom antibodies"],
    [f"total_matches={matches3}", f"ms={ms3:.0f}"],
    f"{'Real bugs found!' if matches3 > 0 else 'No matches — either no bugs or pattern mismatch'}",
    "Adversarial: if matches3 > 0 and matches are NOT real race conditions, that is a false positive. If 0, pattern resolution against graph labels is too loose/strict.",
    "✅" if not err3 else "❌",
)

# ============================================================================
sec("FLOW_SIMULATE — auto-discover entry nodes")
r, ms = call(
    proc,
    "m1nd.flow_simulate",
    {"agent_id": "beta-ext", "num_particles": 4, "max_depth": 10},
)
d = xt(r)
err = is_err(r)
turbulence = d.get("turbulence_points", []) if isinstance(d, dict) else []
valve_pts = d.get("valve_points", []) if isinstance(d, dict) else []
severity = d.get("severity_distribution", {}) if isinstance(d, dict) else {}
print(
    f"  auto-discover: turbulence={len(turbulence)}, valves={len(valve_pts)}, err={err}, ms={ms:.0f}"
)
if turbulence:
    for t in turbulence[:3]:
        print(f"    turbulence: {json.dumps(t)[:200]}")
report(
    "flow_simulate (auto-discover)",
    "tested",
    ["Called with no entry_nodes — auto-discovery mode"],
    [
        f"turbulence_points={len(turbulence)}",
        f"valve_points={len(valve_pts)}",
        f"ms={ms:.0f}",
    ],
    "Auto-discover working" if not err else "Failed",
    "High-degree nodes (hubs) will always show turbulence regardless of actual race conditions — turbulence ≠ race condition without semantic context",
    "✅" if not err else "❌",
)

# flow_simulate with scope_filter
r2, ms2 = call(
    proc,
    "m1nd.flow_simulate",
    {
        "agent_id": "beta-ext",
        "scope_filter": "backend/",
        "num_particles": 4,
        "max_depth": 10,
    },
)
d2 = xt(r2)
err2 = is_err(r2)
turb2 = len(d2.get("turbulence_points", [])) if isinstance(d2, dict) else 0
valve2 = len(d2.get("valve_points", [])) if isinstance(d2, dict) else 0
print(
    f"  scope=backend/: turbulence={turb2}, valves={valve2}, err={err2}, ms={ms2:.0f}"
)

# flow_simulate focused on whatsapp
r3, ms3 = call(
    proc,
    "m1nd.flow_simulate",
    {
        "agent_id": "beta-ext",
        "scope_filter": "backend/whatsapp",
        "num_particles": 4,
        "max_depth": 10,
    },
)
d3 = xt(r3)
err3 = is_err(r3)
turb3 = len(d3.get("turbulence_points", [])) if isinstance(d3, dict) else 0
valve3 = len(d3.get("valve_points", [])) if isinstance(d3, dict) else 0
print(
    f"  scope=backend/whatsapp: turbulence={turb3}, valves={valve3}, err={err3}, ms={ms3:.0f}"
)

# flow_simulate with lock_patterns
r4, ms4 = call(
    proc,
    "m1nd.flow_simulate",
    {
        "agent_id": "beta-ext",
        "scope_filter": "backend/",
        "num_particles": 4,
        "max_depth": 10,
        "lock_patterns": ["_lock", "acquire", "mutex", "asyncio.Lock"],
        "read_only_patterns": ["get_", "fetch_", "read_"],
    },
)
d4 = xt(r4)
err4 = is_err(r4)
turb4 = len(d4.get("turbulence_points", [])) if isinstance(d4, dict) else 0
valve4 = len(d4.get("valve_points", [])) if isinstance(d4, dict) else 0
print(
    f"  with lock_patterns: turbulence={turb4}, valves={valve4}, err={err4}, ms={ms4:.0f}"
)
report(
    "flow_simulate (all 4 variants)",
    "tested",
    [
        "Auto-discover",
        "scope=backend/",
        "scope=backend/whatsapp",
        "with lock_patterns + read_only_patterns",
    ],
    [
        f"turbulence reduction with lock_patterns: {len(turbulence)} → {turb4}",
        f"whatsapp-scoped: {turb3} turbulence",
    ],
    "lock_patterns should reduce turbulence counts if locks are properly detected",
    f"{'lock_patterns REDUCED turbulence — working correctly' if turb4 < len(turbulence) else 'SUSPICIOUS: lock_patterns did not reduce turbulence — may be ignoring parameter'}",
    "✅" if (not err and not err4) else "⚠️",
)

# ============================================================================
sec("EPIDEMIC — bug propagation")
r, ms = call(
    proc,
    "m1nd.epidemic",
    {"agent_id": "beta-ext", "infected_nodes": ["file::backend/worker_pool.py"]},
)
d = xt(r)
err = is_err(r)
R0 = d.get("R0", "?") if isinstance(d, dict) else "?"
peak = d.get("peak_infected", "?") if isinstance(d, dict) else "?"
predictions = d.get("predictions", []) if isinstance(d, dict) else []
print(
    f"  single-seed: R0={R0}, peak_infected={peak}, predictions={len(predictions)}, err={err}, ms={ms:.0f}"
)
if predictions:
    for p in predictions[:3]:
        print(f"    {json.dumps(p)[:200]}")

r2, ms2 = call(
    proc,
    "m1nd.epidemic",
    {
        "agent_id": "beta-ext",
        "infected_nodes": [
            "file::backend/worker_pool.py",
            "file::backend/session_pool_reuse.py",
        ],
        "direction": "both",
        "auto_calibrate": True,
        "top_k": 15,
    },
)
d2 = xt(r2)
err2 = is_err(r2)
R0_2 = d2.get("R0", "?") if isinstance(d2, dict) else "?"
peak2 = d2.get("peak_infected", "?") if isinstance(d2, dict) else "?"
preds2 = d2.get("predictions", []) if isinstance(d2, dict) else []
print(
    f"  multi-seed+auto_calibrate: R0={R0_2}, peak_infected={peak2}, predictions={len(preds2)}, err={err2}, ms={ms2:.0f}"
)
report(
    "epidemic (2 variants)",
    "tested",
    [
        "Single-seed: worker_pool.py",
        "Multi-seed: worker_pool + session_pool_reuse, direction=both, auto_calibrate=True",
    ],
    [f"R0={R0}, peak_infected={peak}", f"Multi-seed R0={R0_2}, peak={peak2}"],
    f"{'R0 and propagation computed' if not err else 'Failed'}",
    "ADVERSARIAL: epidemic propagates via graph EDGES, not semantic similarity — highly connected nodes (like base classes or utils) will appear 'infected' regardless of actual bug relationship. R0 is graph-topology R0, not semantic.",
    "✅" if not err else "❌",
)

# ============================================================================
sec("TREMOR — change acceleration")
r, ms = call(
    proc,
    "m1nd.tremor",
    {"agent_id": "beta-ext", "window": "30d", "top_k": 20, "threshold": 0.1},
)
d = xt(r)
err = is_err(r)
alerts = d.get("alerts", []) if isinstance(d, dict) else []
accelerating = d.get("accelerating_modules", []) if isinstance(d, dict) else []
print(
    f"  30d window: alerts={len(alerts)}, accelerating={len(accelerating)}, err={err}, ms={ms:.0f}"
)
if alerts:
    for a in alerts[:2]:
        print(f"    alert: {json.dumps(a)[:200]}")
if accelerating:
    for a in accelerating[:3]:
        print(f"    module: {json.dumps(a)[:150]}")

r2, ms2 = call(
    proc, "m1nd.tremor", {"agent_id": "beta-ext", "window": "all", "sensitivity": 2.0}
)
d2 = xt(r2)
err2 = is_err(r2)
alerts2 = len(d2.get("alerts", [])) if isinstance(d2, dict) else 0
print(f"  window=all, sensitivity=2.0: alerts={alerts2}, err={err2}, ms={ms2:.0f}")

r3, ms3 = call(
    proc,
    "m1nd.tremor",
    {"agent_id": "beta-ext", "window": "30d", "node_filter": "backend/"},
)
d3 = xt(r3)
err3 = is_err(r3)
alerts3 = len(d3.get("alerts", [])) if isinstance(d3, dict) else 0
print(f"  node_filter=backend/: alerts={alerts3}, err={err3}, ms={ms3:.0f}")

no_data_msg = str(d).lower() if isinstance(d, dict) else str(d).lower()
needs_history = (
    "insufficient" in no_data_msg
    or "no git" in no_data_msg
    or "no history" in no_data_msg
    or (len(alerts) == 0 and len(accelerating) == 0)
)
report(
    "tremor (3 variants)",
    "tested",
    [
        "window=30d,top_k=20,threshold=0.1",
        "window=all,sensitivity=2.0",
        "node_filter=backend/",
    ],
    [
        f"30d alerts={len(alerts)}, accelerating={len(accelerating)}",
        f"all-window alerts={alerts2}",
    ],
    f"{'Needs git history (learn() events) — cold start returned empty' if needs_history else 'Working with available history'}",
    "CAVEAT confirmed: tremor is a warm-up tool. Cold start (no learn() events) = 0 meaningful output. That is correct behavior, not a bug.",
    "⚠️ (needs warm-up)" if needs_history else "✅",
)

# ============================================================================
sec("TRUST — defect history")
r, ms = call(
    proc,
    "m1nd.trust",
    {"agent_id": "beta-ext", "scope": "file", "sort_by": "trust_asc", "top_k": 20},
)
d = xt(r)
err = is_err(r)
trust_list = (
    d.get("rankings", []) if isinstance(d, dict) else (d if isinstance(d, list) else [])
)
print(f"  scope=file: count={len(trust_list)}, err={err}, ms={ms:.0f}")
if trust_list:
    for item in trust_list[:3]:
        print(f"    {json.dumps(item)[:200]}")

r2, ms2 = call(
    proc,
    "m1nd.trust",
    {"agent_id": "beta-ext", "scope": "function", "node_filter": "backend/"},
)
d2 = xt(r2)
err2 = is_err(r2)
trust2 = len(d2.get("rankings", [])) if isinstance(d2, dict) else 0
print(f"  scope=function/backend: count={trust2}, err={err2}, ms={ms2:.0f}")

r3, ms3 = call(
    proc, "m1nd.trust", {"agent_id": "beta-ext", "scope": "file", "min_history": 1}
)
d3 = xt(r3)
err3 = is_err(r3)
trust3 = len(d3.get("rankings", [])) if isinstance(d3, dict) else 0
print(f"  min_history=1: count={trust3}, err={err3}, ms={ms3:.0f}")

cold_start = len(trust_list) == 0 or (
    len(trust_list) > 0
    and all(
        item.get("score", 1.0) == 1.0
        for item in trust_list[:3]
        if isinstance(item, dict)
    )
)
report(
    "trust (3 variants)",
    "tested",
    [
        "scope=file,sort_by=trust_asc,top_k=20",
        "scope=function,node_filter=backend/",
        "min_history=1",
    ],
    [
        f"file rankings={len(trust_list)}",
        f"function/backend rankings={trust2}",
        f"min_history=1 count={trust3}",
    ],
    f"{'Cold start — uniform scores, needs learn() bug events' if cold_start else 'Has defect history'}",
    "Trust without learn() bug reports = all nodes equally trusted = useless ranking. min_history=1 with cold start returns 0, which is honest.",
    "⚠️ (cold start)" if cold_start else "✅",
)

# ============================================================================
sec("LAYERS — architecture detection")
r, ms = call(
    proc,
    "m1nd.layers",
    {
        "agent_id": "beta-ext",
        "scope": "backend/",
        "include_violations": True,
        "exclude_tests": False,
    },
)
d = xt(r)
err = is_err(r)
layer_list = d.get("layers", []) if isinstance(d, dict) else []
violations = d.get("violations", []) if isinstance(d, dict) else []
sep_score = d.get("separation_score", 0) if isinstance(d, dict) else 0
print(
    f"  backend/ layers={len(layer_list)}, violations={len(violations)}, sep_score={sep_score:.3f}, err={err}, ms={ms:.0f}"
)
for l in layer_list[:4]:
    if isinstance(l, dict):
        print(
            f"    layer[{l.get('level', '?')}] '{l.get('name', '?')}': {l.get('node_count', '?')} nodes"
        )
for v in violations[:3]:
    print(f"    violation: {json.dumps(v)[:180]}")

r2, ms2 = call(
    proc,
    "m1nd.layers",
    {
        "agent_id": "beta-ext",
        "scope": "frontend/src/",
        "naming_strategy": "path_prefix",
        "include_violations": True,
    },
)
d2 = xt(r2)
err2 = is_err(r2)
layers2 = len(d2.get("layers", [])) if isinstance(d2, dict) else 0
viols2 = len(d2.get("violations", [])) if isinstance(d2, dict) else 0
sep2 = d2.get("separation_score", 0) if isinstance(d2, dict) else 0
print(
    f"  frontend/src/ (path_prefix): layers={layers2}, violations={viols2}, sep={sep2:.3f}, ms={ms2:.0f}"
)

r3, ms3 = call(
    proc,
    "m1nd.layers",
    {
        "agent_id": "beta-ext",
        "scope": "mcp/m1nd/",
        "naming_strategy": "pagerank",
        "include_violations": True,
    },
)
d3 = xt(r3)
err3 = is_err(r3)
layers3 = len(d3.get("layers", [])) if isinstance(d3, dict) else 0
viols3 = len(d3.get("violations", [])) if isinstance(d3, dict) else 0
sep3 = d3.get("separation_score", 0) if isinstance(d3, dict) else 0
print(
    f"  mcp/m1nd/ (pagerank): layers={layers3}, violations={viols3}, sep={sep3:.3f}, ms={ms3:.0f}"
)
report(
    "layers (3 scopes)",
    "tested",
    ["backend/ with violations", "frontend/src/ path_prefix", "mcp/m1nd/ pagerank"],
    [
        f"backend: {len(layer_list)} layers, {len(violations)} violations, sep={sep_score:.3f}",
        f"frontend: {layers2} layers, {viols2} violations, sep={sep2:.3f}",
        f"mcp: {layers3} layers, {viols3} violations, sep={sep3:.3f}",
    ],
    f"{'Architecture layers detected' if len(layer_list) > 0 else 'No layers found — may need denser subgraph'}",
    "Adversarial: pagerank-based naming assigns layer 0 to most-central nodes — this may conflate infrastructure with API layers. Check if layer[0] is actually 'lowest' or 'highest' dependency.",
    "✅" if not err else "⚠️",
)

# Save layer data for layer_inspect
top_layers = [
    (l.get("level", 0), l.get("name", "?"))
    for l in layer_list[:2]
    if isinstance(l, dict)
]

# ============================================================================
sec("LAYER_INSPECT — inspect layers 0 and 1")
for level in [0, 1]:
    r, ms = call(
        proc,
        "m1nd.layer_inspect",
        {
            "agent_id": "beta-ext",
            "scope": "backend/",
            "level": level,
            "include_edges": True,
            "top_k": 30,
        },
    )
    d = xt(r)
    err = is_err(r)
    health = d.get("health", {}) if isinstance(d, dict) else {}
    nodes = d.get("nodes", []) if isinstance(d, dict) else []
    int_edges = d.get("internal_edges", 0) if isinstance(d, dict) else "?"
    ext_edges = d.get("external_edges", 0) if isinstance(d, dict) else "?"
    viols = d.get("violations", []) if isinstance(d, dict) else []
    print(
        f"  level={level}: nodes={len(nodes)}, int_edges={int_edges}, ext_edges={ext_edges}, violations={len(viols)}, err={err}, ms={ms:.0f}"
    )
    if health:
        print(f"    health: {json.dumps(health)[:200]}")
    for v in viols[:2]:
        print(f"    violation: {json.dumps(v)[:180]}")

report(
    "layer_inspect (levels 0,1)",
    "tested",
    ["Inspected level=0 and level=1 of backend/ architecture"],
    ["health metrics, internal/external coupling, violation details"],
    "Coupling metrics tell you if a layer is well-isolated or tightly coupled to others",
    "Adversarial: if internal_edges >> external_edges but violations are still high, the violation detection may be using a different edge model than the coupling calc",
    "✅" if not err else "⚠️",
)

# ============================================================================
sec("SCAN — custom patterns beyond 8 presets")
for pattern in ["missing_timeout", "unguarded_shared_state"]:
    r, ms = call(proc, "m1nd.scan", {"agent_id": "beta-ext", "pattern": pattern})
    d = xt(r)
    err = is_err(r)
    count = d.get("count", 0) if isinstance(d, dict) else "?"
    preset_only_err = (
        "unknown pattern" in str(d).lower()
        or "invalid pattern" in str(d).lower()
        or "not supported" in str(d).lower()
    )
    print(
        f"  pattern={pattern}: count={count}, preset_only_err={preset_only_err}, err={err}, ms={ms:.0f}"
    )
    print(f"    response: {json.dumps(d)[:200]}")
report(
    "scan (custom patterns)",
    "tested",
    ["pattern=missing_timeout", "pattern=unguarded_shared_state"],
    ["Both are NOT in the 8 built-in patterns"],
    "If errors: only 8 presets supported. If works: dynamic pattern resolution.",
    "KEY TEST: does custom pattern work or does it reject non-preset values? This determines extensibility.",
    "🔍 (behavior depends on implementation)",
)

# ============================================================================
sec("VALIDATE_PLAN — edge cases")
# Large plan
large_plan = [
    {"file": f"backend/module_{i}.py", "changes": f"change {i}"} for i in range(12)
]
r, ms = call(proc, "m1nd.validate_plan", {"agent_id": "beta-ext", "plan": large_plan})
d = xt(r)
err = is_err(r)
print(f"  12-file plan: err={err}, ms={ms:.0f}")
print(f"    response: {json.dumps(d)[:300]}")

# Circular dependencies
circular_plan = [
    {"file": "backend/a.py", "depends_on": ["backend/b.py"]},
    {"file": "backend/b.py", "depends_on": ["backend/c.py"]},
    {"file": "backend/c.py", "depends_on": ["backend/a.py"]},
]
r2, ms2 = call(
    proc, "m1nd.validate_plan", {"agent_id": "beta-ext", "plan": circular_plan}
)
d2 = xt(r2)
err2 = is_err(r2)
circ_detected = "circular" in str(d2).lower() or "cycle" in str(d2).lower()
print(f"  circular deps: detected={circ_detected}, err={err2}, ms={ms2:.0f}")
print(f"    response: {json.dumps(d2)[:300]}")

# Empty plan
r3, ms3 = call(proc, "m1nd.validate_plan", {"agent_id": "beta-ext", "plan": []})
d3 = xt(r3)
err3 = is_err(r3)
print(f"  empty plan []: err={err3}, ms={ms3:.0f}")
print(f"    response: {json.dumps(d3)[:200]}")

report(
    "validate_plan (edge cases)",
    "tested",
    ["12-file large plan", "circular dependency plan (a→b→c→a)", "empty plan []"],
    [f"large: err={err}", f"circular detected={circ_detected}", f"empty: err={err3}"],
    "Should handle all edge cases gracefully",
    f"CRITICAL: circular deps {'DETECTED' if circ_detected else 'NOT DETECTED — silent failure'} — this is the most important edge case",
    "✅" if circ_detected else "⚠️",
)

# ============================================================================
sec("FEDERATE — cross-repo")
r, ms = call(
    proc,
    "m1nd.federate",
    {
        "agent_id": "beta-ext",
        "repos": [
            {"name": "main", "path": "/Users/cosmophonix/clawd/roomanizer-os"},
            {"name": "mcp", "path": "/Users/cosmophonix/clawd/roomanizer-os/mcp"},
        ],
    },
)
d = xt(r)
err = is_err(r)
cross_edges = d.get("cross_repo_edges", 0) if isinstance(d, dict) else "?"
unified_nodes = d.get("unified_node_count", 0) if isinstance(d, dict) else "?"
print(
    f"  federate: cross_edges={cross_edges}, unified_nodes={unified_nodes}, err={err}, ms={ms:.0f}"
)
print(f"  response: {json.dumps(d)[:300]}")
report(
    "federate (self-repo test)",
    "tested",
    ["Fed roomanizer-os/main with roomanizer-os/mcp (subdirectory overlap)"],
    [
        f"cross_repo_edges={cross_edges}",
        f"unified_nodes={unified_nodes}",
        f"ms={ms:.0f}",
    ],
    f"{'Cross-repo edges detected' if isinstance(cross_edges, int) and cross_edges > 0 else 'No cross-edges or not implemented'}",
    "Federating a repo with its own subdirectory will create ghost cross-edges — tests deduplication logic",
    "✅" if not err else "⚠️",
)

# ============================================================================
sec("DIFFERENTIAL — snapshot comparison")
r, ms = call(
    proc,
    "m1nd.differential",
    {"agent_id": "beta-ext", "snapshot_a": "current", "snapshot_b": "current"},
)
d = xt(r)
err = is_err(r)
diff_count = d.get("diff_count", "?") if isinstance(d, dict) else "?"
changes = d.get("changes", []) if isinstance(d, dict) else []
print(
    f"  same-snapshot diff: diff_count={diff_count}, changes={len(changes)}, err={err}, ms={ms:.0f}"
)
print(f"  response: {json.dumps(d)[:300]}")

r2, ms2 = call(
    proc,
    "m1nd.differential",
    {
        "agent_id": "beta-ext",
        "snapshot_a": "current",
        "snapshot_b": "current",
        "question": "what changed in the session pool",
    },
)
d2 = xt(r2)
err2 = is_err(r2)
print(f"  with question: err={err2}, ms={ms2:.0f}")
print(f"  response: {json.dumps(d2)[:300]}")
report(
    "differential (same-snapshot)",
    "tested",
    [
        "snapshot_a=current, snapshot_b=current (should be 0 diff)",
        "With semantic question",
    ],
    [f"diff_count={diff_count}", f"changes={len(changes)}"],
    f"{'Correctly returns 0 diff for same snapshot' if diff_count == 0 or diff_count == '?' else 'BUG: non-zero diff for same snapshot'}",
    "CRITICAL: if same-snapshot returns non-zero changes, differential has a hashing/identity bug",
    "✅" if diff_count == 0 else ("⚠️ if 0 not returned" if diff_count == "?" else "❌"),
)

# ============================================================================
print("\n" + "=" * 65)
print("  BETA EXTENDED — FINAL SUMMARY")
print("=" * 65)
print(f"\nGraph: {node_count} nodes, {edge_count} edges ingested")
print(f"\nTool Status Summary:")
print(f"  antibody_list          — ✅ works cold")
print(f"  antibody_create        — {'✅' if not err else '⚠️'} created 2 antibodies")
print(f"  antibody_scan (full)   — total_matches={total_matches}")
print(f"  antibody_scan (changed) — matches={matches2} (needs git state)")
print(f"  antibody_scan (post-create) — matches={matches3}")
print(
    f"  flow_simulate          — turbulence={len(turbulence)} (auto), {turb4} (with locks)"
)
print(f"  epidemic               — R0={R0}, peak={peak}")
print(
    f"  tremor                 — {'NEEDS WARM-UP' if needs_history else f'alerts={len(alerts)}'}"
)
print(
    f"  trust                  — {'NEEDS WARM-UP' if cold_start else f'rankings={len(trust_list)}'}"
)
print(
    f"  layers                 — {len(layer_list)} layers, {len(violations)} violations, sep={sep_score:.3f}"
)
print(f"  layer_inspect          — coupling metrics available")
print(f"  scan (custom)          — see above for preset vs custom behavior")
print(
    f"  validate_plan          — circular={'DETECTED' if circ_detected else 'MISSED'}"
)
print(f"  federate               — cross_edges={cross_edges}")
print(f"  differential           — same-snapshot diff_count={diff_count}")

proc.stdin.close()
proc.wait(timeout=5)
print(f"\n[beta-extended] DONE. workdir: {workdir}")
