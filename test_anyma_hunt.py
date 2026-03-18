#!/usr/bin/env python3
"""
m1nd Perspective MCP — ANYMA Bug Hunt
Ingest the full backend, use perspective tools to explore ANYMA subsystems,
find structural issues, dead connections, orphan modules.
"""
import json
import subprocess
import sys
import os
import tempfile
import time

BINARY = "./target/release/m1nd-mcp"
BACKEND_PATH = "/Users/cosmophonix/clawd/roomanizer-os/backend"
FINDINGS = []

workdir = tempfile.mkdtemp(prefix="m1nd_anyma_")

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

MSG_ID = 0
def next_id():
    global MSG_ID
    MSG_ID += 1
    return MSG_ID

def call(proc, name, args):
    msg = json.dumps({
        "jsonrpc": "2.0",
        "method": "tools/call",
        "id": next_id(),
        "params": {"name": name, "arguments": args}
    })
    proc.stdin.write((msg + "\n").encode())
    proc.stdin.flush()
    line = proc.stdout.readline().decode().strip()
    return json.loads(line)

def init(proc):
    msg = json.dumps({
        "jsonrpc": "2.0",
        "method": "initialize",
        "id": 1,
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "anyma-hunt", "version": "1.0"}
        }
    })
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
        return None

def is_error(resp):
    try:
        return resp["result"].get("isError", False)
    except:
        return True

def finding(category, severity, title, detail):
    f = {"category": category, "severity": severity, "title": title, "detail": detail}
    FINDINGS.append(f)
    icon = {"critical": "!!!", "high": "!!", "medium": "!", "info": "."}[severity]
    print(f"  [{icon}] {title}")
    if detail:
        print(f"      {detail[:200]}")

def explore_node(proc, agent, persp_id, query, depth=3):
    """Follow routes from a node, collecting visited nodes and their connections."""
    visited = []
    dead_ends = []

    r = call(proc, "m1nd.perspective.start", {"agent_id": agent, "query": query})
    d = xt(r)
    if isinstance(d, str) or d is None:
        return {"error": d, "visited": [], "dead_ends": []}

    rsv = d.get("route_set_version", 0)
    focus = d.get("focus_node", "?")
    routes = d.get("routes", [])
    visited.append({"node": focus, "routes": len(routes), "route_targets": [r["target_label"] for r in routes[:6]]})

    if not routes:
        dead_ends.append(focus)

    # Follow up to `depth` levels
    for i in range(depth):
        if not routes:
            break
        route = routes[0]
        r = call(proc, "m1nd.perspective.follow", {
            "agent_id": agent, "perspective_id": persp_id,
            "route_id": route["route_id"], "route_set_version": rsv
        })
        d = xt(r)
        if isinstance(d, str) or d is None or is_error(r):
            break
        rsv = d.get("route_set_version", rsv)
        focus = d.get("new_focus", "?")
        routes = d.get("routes", [])
        visited.append({"node": focus, "routes": len(routes), "route_targets": [r["target_label"] for r in routes[:6]]})
        if not routes:
            dead_ends.append(focus)

    # Close
    call(proc, "m1nd.perspective.close", {"agent_id": agent, "perspective_id": persp_id})

    return {"visited": visited, "dead_ends": dead_ends}

# ============================================================================
# Main
# ============================================================================

proc = start_server()
print(f"Server PID {proc.pid}")
init(proc)
print("Init OK")

# Ingest full backend
print("\n=== INGESTING FULL BACKEND ===")
t0 = time.time()
r = call(proc, "m1nd.ingest", {"agent_id": "hunt", "path": BACKEND_PATH, "mode": "full"})
d = xt(r)
elapsed = time.time() - t0
print(f"Ingested: {d.get('node_count', '?')} nodes, {d.get('edge_count', '?')} edges in {elapsed:.1f}s")

node_count = d.get("node_count", 0)
edge_count = d.get("edge_count", 0)

# ============================================================================
print("\n=== PHASE 1: ANYMA Subsystem Discovery ===")
# Which ANYMA modules does the graph know about?
# ============================================================================

anyma_modules = [
    "anyma_orchestrator", "archivist_daemon", "archivist_runtime",
    "mapper_daemon", "mapper_extract", "mapper_grounding",
    "sentinel", "verifier", "pattern_cortex", "critic",
    "director_daemon", "director_storms", "director_support",
    "observer_daemon", "pulse_daemon", "ego_daemon",
]

print("Activating each ANYMA module...")
found_modules = []
missing_modules = []

for mod_name in anyma_modules:
    r = call(proc, "m1nd.activate", {"agent_id": "hunt", "query": mod_name, "top_k": 5})
    d = xt(r)
    if isinstance(d, dict):
        activated = d.get("activated", [])
        if activated:
            best = activated[0]
            found_modules.append({"name": mod_name, "label": best["label"], "type": best["type"], "score": best["activation"]})
        else:
            missing_modules.append(mod_name)
    else:
        missing_modules.append(mod_name)

print(f"  Found: {len(found_modules)}/{len(anyma_modules)}")
for m in found_modules:
    print(f"    {m['name']}: {m['label']} ({m['type']}) activation={m['score']:.3f}")
if missing_modules:
    finding("coverage", "medium", f"{len(missing_modules)} ANYMA modules not in graph", ", ".join(missing_modules))

# ============================================================================
print("\n=== PHASE 2: Connectivity Analysis ===")
# For each found module, check how connected it is
# ============================================================================

for mod_info in found_modules[:8]:  # top 8
    agent = f"conn_{mod_info['name'][:4]}"
    persp = f"persp_{agent}_001"

    r = call(proc, "m1nd.perspective.start", {"agent_id": agent, "query": mod_info["label"]})
    d = xt(r)
    if isinstance(d, str) or d is None:
        finding("connectivity", "medium", f"{mod_info['name']}: can't start perspective", str(d)[:100])
        continue

    rsv = d.get("route_set_version", 0)
    routes = d.get("routes", [])
    focus = d.get("focus_node", "?")

    # Get full routes
    r = call(proc, "m1nd.perspective.routes", {
        "agent_id": agent, "perspective_id": persp, "route_set_version": rsv
    })
    d = xt(r)
    if isinstance(d, dict):
        routes = d.get("routes", [])
        total = d.get("total_routes", 0)
    else:
        routes = []
        total = 0

    print(f"  {mod_info['name']}: {total} routes from {focus}")
    for rt in routes[:5]:
        print(f"    → {rt['target_label']} [{rt['family']}] score={rt['score']:.2f} reason={rt.get('reason', '')[:50]}")

    if total == 0:
        finding("connectivity", "high", f"{mod_info['name']}: ISOLATED — 0 routes", f"Node {focus} has no connections in graph")
    elif total == 1:
        finding("connectivity", "medium", f"{mod_info['name']}: weakly connected (1 route)", routes[0]["target_label"])

    call(proc, "m1nd.perspective.close", {"agent_id": agent, "perspective_id": persp})

# ============================================================================
print("\n=== PHASE 3: Cross-Module Affinity ===")
# Check how ANYMA subsystems relate to each other
# ============================================================================

affinity_pairs = [
    ("archivist_daemon", "director_daemon"),  # Director sends work to Archivist
    ("archivist_daemon", "sentinel"),          # Archivist uses Sentinel for locks
    ("archivist_daemon", "verifier"),          # Archivist sends edits to Verifier
    ("critic", "pattern_cortex"),              # Critic feeds Pattern Cortex
    ("director_daemon", "mapper_daemon"),      # Director uses Mapper for context
    ("pulse_daemon", "ego_daemon"),            # Pulse + Ego = awareness layer
    ("anyma_orchestrator", "archivist_daemon"), # Orchestrator manages all
    ("chat_handler", "anyma_orchestrator"),    # Chat triggers ANYMA?
]

print("Checking cross-module affinity (via m1nd.why)...")
for src, tgt in affinity_pairs:
    r = call(proc, "m1nd.why", {"agent_id": "hunt", "source": src, "target": tgt, "max_hops": 6})
    d = xt(r)
    if isinstance(d, dict):
        paths = d.get("paths", [])
        if paths:
            shortest = paths[0]
            hops = len(shortest.get("nodes", [])) - 1
            print(f"  {src} → {tgt}: {len(paths)} paths (shortest: {hops} hops)")
            for p in paths[:2]:
                nodes = [n.get("label", "?") for n in p.get("nodes", [])]
                print(f"    path: {' → '.join(nodes)}")
        else:
            finding("coupling", "high", f"NO PATH: {src} → {tgt}", "Expected connection but graph shows no path")
            print(f"  {src} → {tgt}: NO PATH FOUND")
    else:
        print(f"  {src} → {tgt}: error ({str(d)[:80]})")

# ============================================================================
print("\n=== PHASE 4: Structural Holes (m1nd.missing) ===")
# What's missing from the ANYMA subgraph?
# ============================================================================

r = call(proc, "m1nd.missing", {"agent_id": "hunt", "query": "anyma", "top_k": 20})
d = xt(r)
if isinstance(d, dict):
    holes = d.get("structural_holes", d.get("missing", []))
    if holes:
        print(f"  Found {len(holes)} structural holes:")
        for h in holes[:10]:
            if isinstance(h, dict):
                print(f"    {h.get('label', h.get('node', '?'))}: {h.get('reason', h.get('gap_type', '?'))}")
            else:
                print(f"    {h}")
    else:
        print("  No structural holes found (or query didn't match)")
else:
    print(f"  missing result: {str(d)[:200]}")

# ============================================================================
print("\n=== PHASE 5: Impact Analysis ===")
# What would break if we changed key ANYMA modules?
# ============================================================================

impact_targets = ["anyma_orchestrator", "archivist_daemon", "chat_handler"]
for target in impact_targets:
    r = call(proc, "m1nd.impact", {"agent_id": "hunt", "node": target, "radius": 3})
    d = xt(r)
    if isinstance(d, dict):
        affected = d.get("affected_nodes", d.get("impacted", []))
        print(f"  Impact of changing {target}: {len(affected)} affected nodes")
        for a in affected[:8]:
            if isinstance(a, dict):
                print(f"    {a.get('label', a.get('node', '?'))} (distance={a.get('distance', '?')})")
            else:
                print(f"    {a}")
    else:
        print(f"  {target}: {str(d)[:150]}")

# ============================================================================
print("\n=== PHASE 6: Deep Perspective Walk — ANYMA Orchestrator ===")
# Start at anyma_orchestrator and follow the dependency chain
# ============================================================================

result = explore_node(proc, "deep", "persp_deep_001", "anyma_orchestrator", depth=5)
if result.get("error"):
    print(f"  Error: {result['error']}")
else:
    print(f"  Visited {len(result['visited'])} nodes:")
    for v in result["visited"]:
        print(f"    {v['node']} ({v['routes']} routes) → {v['route_targets'][:3]}")
    if result["dead_ends"]:
        finding("navigation", "info", f"Dead ends: {len(result['dead_ends'])}", ", ".join(result['dead_ends']))

# ============================================================================
print("\n=== PHASE 7: Duplicate/Pattern Detection ===")
# ============================================================================

r = call(proc, "m1nd.fingerprint", {"agent_id": "hunt", "query": "daemon", "top_k": 20})
d = xt(r)
if isinstance(d, dict):
    clusters = d.get("clusters", d.get("fingerprints", []))
    print(f"  Found {len(clusters)} fingerprint clusters for 'daemon':")
    for c in clusters[:10]:
        if isinstance(c, dict):
            print(f"    {c.get('label', c.get('representative', '?'))}: similarity={c.get('similarity', '?')}")
        else:
            print(f"    {c}")
else:
    print(f"  fingerprint: {str(d)[:200]}")

# ============================================================================
# Health + cleanup
# ============================================================================

r = call(proc, "m1nd.health", {"agent_id": "hunt"})
d = xt(r)
print(f"\n=== HEALTH: {d.get('node_count', '?')} nodes, {d.get('edge_count', '?')} edges, {d.get('status', '?')} ===")

proc.stdin.close()
proc.wait(timeout=10)

# ============================================================================
print("\n" + "="*70)
print(f" FINDINGS: {len(FINDINGS)} total")
print("="*70)

by_sev = {}
for f in FINDINGS:
    by_sev.setdefault(f["severity"], []).append(f)

for sev in ["critical", "high", "medium", "info"]:
    items = by_sev.get(sev, [])
    if items:
        print(f"\n  [{sev.upper()}] ({len(items)})")
        for f in items:
            print(f"    - {f['title']}")
            if f["detail"]:
                print(f"      {f['detail'][:150]}")

import shutil
shutil.rmtree(workdir, ignore_errors=True)
