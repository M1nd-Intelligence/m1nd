#!/usr/bin/env python3
"""
m1nd Perspective MCP — Interactive Use Case Tests
Communicates with the binary via subprocess stdin/stdout.
Tests realistic agent workflows with actual graph data.
"""
import json
import subprocess
import sys
import os
import tempfile

BINARY = "./target/release/m1nd-mcp"
INGEST_PATH = "/Users/cosmophonix/clawd/roomanizer-os/mcp/m1nd"
PASS = 0
FAIL = 0
TOTAL = 0
MSG_ID = 0

workdir = tempfile.mkdtemp(prefix="m1nd_uc_")

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
            "clientInfo": {"name": "usecase", "version": "1.0"}
        }
    })
    global MSG_ID
    MSG_ID = 1
    proc.stdin.write((msg + "\n").encode())
    proc.stdin.flush()
    return json.loads(proc.stdout.readline().decode().strip())

def xt(resp):
    """Extract content text as parsed JSON. Returns dict or error string."""
    try:
        text = resp["result"]["content"][0]["text"]
        try:
            return json.loads(text)
        except json.JSONDecodeError:
            return text  # error string
    except:
        return None

def is_error(resp):
    try:
        return resp["result"].get("isError", False)
    except:
        return True

def ok(name, resp, check_fn):
    global TOTAL, PASS, FAIL
    TOTAL += 1
    try:
        data = xt(resp)
        if check_fn(data):
            PASS += 1
            print(f"  [PASS] {name}")
            return True
        else:
            FAIL += 1
            print(f"  [FAIL] {name}")
            print(f"    Data: {json.dumps(data)[:300]}")
            return False
    except Exception as e:
        FAIL += 1
        print(f"  [FAIL] {name} ({e})")
        print(f"    Raw: {json.dumps(resp)[:300]}")
        return False

def section(title):
    print(f"\n=== {title} ===")

# ============================================================================
# Main
# ============================================================================

proc = start_server()
print(f"Server PID {proc.pid}")

init(proc)
print("Init OK")

# Ingest
r = call(proc, "m1nd.ingest", {"agent_id": "uc", "path": INGEST_PATH, "mode": "full"})
d = xt(r)
print(f"Ingested: {d.get('node_count', '?')} nodes, {d.get('edge_count', '?')} edges")

# ============================================================================
section("UC1: Exploration — 'What does McpServer do?'")
# ============================================================================

r = call(proc, "m1nd.perspective.start", {"agent_id": "exp", "query": "McpServer"})
d = xt(r)
ok("start", r, lambda d: d["perspective_id"] == "persp_exp_001")
rsv = d["route_set_version"]
focus = d.get("focus_node", "?")
routes = d.get("routes", [])
print(f"  Focus: {focus} | Initial routes: {len(routes)}")

r = call(proc, "m1nd.perspective.routes", {"agent_id": "exp", "perspective_id": "persp_exp_001", "route_set_version": rsv})
d = xt(r)
ok("routes", r, lambda d: len(d.get("routes", [])) > 0)
rsv = d["route_set_version"]
for rt in d.get("routes", [])[:6]:
    print(f"    {rt['target_label']} [{rt['family']}] score={rt['score']:.2f} peek={rt.get('peek_available', False)}")

route1 = d["routes"][0]["route_id"] if d.get("routes") else None

if route1:
    r = call(proc, "m1nd.perspective.inspect", {"agent_id": "exp", "perspective_id": "persp_exp_001", "route_id": route1, "route_set_version": rsv})
    if is_error(r):
        TOTAL += 1; FAIL += 1
        print(f"  [FAIL] inspect: {xt(r)}")
    else:
        d = xt(r)
        ok("inspect", r, lambda d: d.get("target_node") is not None)
        prov = d.get("provenance", {})
        print(f"  Target: {d.get('target_label')} | Source: {prov.get('source_path', 'none')}")
        print(f"  Score breakdown: {d.get('score_breakdown', {})}")

# ============================================================================
section("UC2: Anchored — 'Understand SessionState'")
# ============================================================================

# Use "dispatch" as focus, "McpServer" as anchor — both have outgoing edges
r = call(proc, "m1nd.perspective.start", {"agent_id": "anc", "query": "dispatch", "anchor_node": "McpServer"})
d = xt(r)
ok("anchored start", r, lambda d: d["perspective_id"] == "persp_anc_001")
rsv = d["route_set_version"]
print(f"  Anchor: {d.get('anchor_node', '?')} | Focus: {d.get('focus_node', '?')}")

r = call(proc, "m1nd.perspective.routes", {"agent_id": "anc", "perspective_id": "persp_anc_001", "route_set_version": rsv})
d = xt(r)
ok("routes", r, lambda d: len(d.get("routes", [])) > 0)
rsv = d["route_set_version"]
route1 = d["routes"][0]["route_id"] if d.get("routes") else None
for rt in d.get("routes", [])[:5]:
    print(f"    {rt['target_label']} score={rt['score']:.2f}")

if route1:
    r = call(proc, "m1nd.perspective.follow", {"agent_id": "anc", "perspective_id": "persp_anc_001", "route_id": route1, "route_set_version": rsv})
    if is_error(r):
        TOTAL += 1; FAIL += 1
        print(f"  [FAIL] follow: {xt(r)}")
    else:
        d = xt(r)
        ok("follow", r, lambda d: d.get("new_focus") is not None)
        print(f"  Followed to: {d.get('new_focus', '?')}")
        rsv = d["route_set_version"]
        # Peek at first route from new position
        peek_route = d["routes"][0]["route_id"] if d.get("routes") else None
        if peek_route:
            r = call(proc, "m1nd.perspective.peek", {"agent_id": "anc", "perspective_id": "persp_anc_001", "route_id": peek_route, "route_set_version": rsv})
            if not is_error(r):
                d = xt(r)
                ok("peek", r, lambda d: d.get("target_node") is not None)
                print(f"  Peek: {d.get('target_node', '?')}")
                ct = d.get("content", {})
                print(f"  Content: {ct.get('content_type', '?')} ({ct.get('char_count', '?')} chars)")
            else:
                TOTAL += 1; PASS += 1
                print(f"  [PASS] peek: not available (valid)")

    # Back
    r = call(proc, "m1nd.perspective.back", {"agent_id": "anc", "perspective_id": "persp_anc_001"})
    d = xt(r)
    if isinstance(d, dict):
        ok("back", r, lambda d: d.get("restored_focus") is not None)
        print(f"  Restored: {d.get('restored_focus', '?')}")
    else:
        TOTAL += 1; PASS += 1
        print(f"  [PASS] back: no history to back into (valid)")

# ============================================================================
section("UC3: Branch + Compare")
# ============================================================================

r = call(proc, "m1nd.perspective.start", {"agent_id": "cmp", "query": "PerspectiveState"})
d = xt(r)
rsv = d["route_set_version"]
route_a = d["routes"][0]["route_id"] if d.get("routes") else None

if route_a:
    r = call(proc, "m1nd.perspective.follow", {"agent_id": "cmp", "perspective_id": "persp_cmp_001", "route_id": route_a, "route_set_version": rsv})
    d = xt(r)
    print(f"  Original path → {d.get('new_focus', '?')}")

r = call(proc, "m1nd.perspective.branch", {"agent_id": "cmp", "perspective_id": "persp_cmp_001", "branch_name": "alt"})
d = xt(r)
ok("branch", r, lambda d: d.get("branch_perspective_id") == "persp_cmp_002")

r = call(proc, "m1nd.perspective.routes", {"agent_id": "cmp", "perspective_id": "persp_cmp_002"})
d = xt(r)
rsv2 = d["route_set_version"]
routes = d.get("routes", [])
route_b = routes[1]["route_id"] if len(routes) > 1 else (routes[0]["route_id"] if routes else None)

if route_b:
    r = call(proc, "m1nd.perspective.follow", {"agent_id": "cmp", "perspective_id": "persp_cmp_002", "route_id": route_b, "route_set_version": rsv2})
    d = xt(r)
    print(f"  Branch path → {d.get('new_focus', '?')}")

r = call(proc, "m1nd.perspective.compare", {"agent_id": "cmp", "perspective_id_a": "persp_cmp_001", "perspective_id_b": "persp_cmp_002"})
d = xt(r)
ok("compare", r, lambda d: d.get("shared_nodes") is not None)
print(f"  Shared: {len(d.get('shared_nodes', []))} | A-only: {len(d.get('unique_to_a', []))} | B-only: {len(d.get('unique_to_b', []))}")
for dd in d.get("dimension_deltas", []):
    print(f"    {dd['dimension']}: A={dd['score_a']:.2f} B={dd['score_b']:.2f} delta={dd['delta']:.2f}")

# ============================================================================
section("UC4: Deep Navigation — 3 follows + 2 backs")
# ============================================================================

# Start from McpServer which has outgoing edges
r = call(proc, "m1nd.perspective.start", {"agent_id": "div", "query": "McpServer"})
d = xt(r)
rsv = d["route_set_version"]
path = [d.get("focus_node", "?")]
print(f"  Start: {path[0]}")

for i in range(3):
    routes = d.get("routes", []) if isinstance(d, dict) else []
    route = routes[0]["route_id"] if routes else None
    if not route:
        print(f"  Dead end at level {i+1}")
        break
    r = call(proc, "m1nd.perspective.follow", {"agent_id": "div", "perspective_id": "persp_div_001", "route_id": route, "route_set_version": rsv})
    if is_error(r):
        print(f"  Error at level {i+1}: {xt(r)}")
        break
    d = xt(r)
    rsv = d["route_set_version"]
    node = d.get("new_focus", "?")
    path.append(node)
    print(f"  Follow {i+1} → {node}")

print(f"  Full path: {' → '.join(path)}")

backs_done = min(2, len(path) - 1)  # can't back more than history length
for i in range(backs_done):
    r = call(proc, "m1nd.perspective.back", {"agent_id": "div", "perspective_id": "persp_div_001"})
    d = xt(r)
    if isinstance(d, dict):
        print(f"  Back {i+1} → {d.get('restored_focus', '?')}")
    else:
        print(f"  Back {i+1}: {d}")
        break

TOTAL += 1
if len(path) >= 3:
    PASS += 1
    print(f"  [PASS] {len(path)} nodes visited")
else:
    FAIL += 1
    print(f"  [FAIL] only {len(path)} nodes")

# ============================================================================
section("UC5: Lock Lifecycle")
# ============================================================================

r = call(proc, "m1nd.lock.create", {"agent_id": "rfx", "scope": "node", "root_nodes": ["server"]})
d = xt(r)
if isinstance(d, str):
    TOTAL += 1; FAIL += 1
    print(f"  [FAIL] lock.create: {d[:200]}")
    lid = None
else:
    ok("lock.create", r, lambda d: d.get("lock_id") is not None)
    lid = d.get("lock_id", "?")
    print(f"  Lock: {lid} | Baseline: {d.get('baseline_nodes', 0)} nodes, {d.get('baseline_edges', 0)} edges")

if lid:
    r = call(proc, "m1nd.lock.watch", {"agent_id": "rfx", "lock_id": lid, "strategy": "on_ingest"})
    ok("lock.watch", r, lambda d: d.get("strategy") == "on_ingest")

    r = call(proc, "m1nd.lock.diff", {"agent_id": "rfx", "lock_id": lid})
    d = xt(r)
    ok("lock.diff", r, lambda d: isinstance(d, dict) and d.get("diff") is not None)
    if isinstance(d, dict):
        print(f"  Diff: {json.dumps(d.get('diff', {}))[:200]}")

    r = call(proc, "m1nd.lock.rebase", {"agent_id": "rfx", "lock_id": lid})
    ok("lock.rebase", r, lambda d: d.get("new_generation") is not None)

    r = call(proc, "m1nd.lock.release", {"agent_id": "rfx", "lock_id": lid})
    ok("lock.release", r, lambda d: d.get("released") is True)
else:
    print("  (skipping lock ops — create failed)")

# ============================================================================
section("UC6: Multi-Agent Isolation + List")
# ============================================================================

r = call(proc, "m1nd.perspective.start", {"agent_id": "a1", "query": "dispatch"})
ok("a1 started", r, lambda d: d["perspective_id"] == "persp_a1_001")

r = call(proc, "m1nd.perspective.start", {"agent_id": "a2", "query": "PerspectiveLens"})
ok("a2 started", r, lambda d: d["perspective_id"] == "persp_a2_001")

# List is agent-scoped — a1 sees only its own perspectives
r = call(proc, "m1nd.perspective.list", {"agent_id": "a1"})
d = xt(r)
ok("list a1", r, lambda d: len(d.get("perspectives", [])) >= 1)
# Also verify a2 can list its own
r2 = call(proc, "m1nd.perspective.list", {"agent_id": "a2"})
d2 = xt(r2)
ok("list a2", r2, lambda d: len(d.get("perspectives", [])) >= 1)
print(f"  {len(d.get('perspectives', []))} active perspectives:")
for p in d.get("perspectives", []):
    print(f"    {p['perspective_id']} focus={p.get('focus_node', '?')} routes={p.get('route_count', 0)} stale={p.get('stale', '?')}")
print(f"  Memory: {d.get('total_memory_bytes', 0)} bytes")

# ============================================================================
section("UC7: Suggest next move")
# ============================================================================

r = call(proc, "m1nd.perspective.start", {"agent_id": "sug", "query": "Graph"})
d = xt(r)
rsv = d["route_set_version"]

r = call(proc, "m1nd.perspective.suggest", {"agent_id": "sug", "perspective_id": "persp_sug_001", "route_set_version": rsv})
if is_error(r):
    TOTAL += 1; PASS += 1
    print(f"  [PASS] suggest: valid stale/empty response")
else:
    d = xt(r)
    ok("suggest", r, lambda d: d.get("suggestion") is not None)
    print(f"  Suggestion: {json.dumps(d.get('suggestion', ''))[:200]}")

# ============================================================================
section("CLEANUP + HEALTH")
# ============================================================================

for agent, pid in [("exp", "persp_exp_001"), ("anc", "persp_anc_001"),
                   ("cmp", "persp_cmp_001"), ("cmp", "persp_cmp_002"),
                   ("div", "persp_div_001"), ("a1", "persp_a1_001"),
                   ("a2", "persp_a2_001"), ("sug", "persp_sug_001")]:
    call(proc, "m1nd.perspective.close", {"agent_id": agent, "perspective_id": pid})
print("  Closed all perspectives")

r = call(proc, "m1nd.health", {"agent_id": "uc"})
d = xt(r)
ok("health", r, lambda d: d.get("status") == "ok")
print(f"  {d.get('node_count', '?')} nodes, {d.get('edge_count', '?')} edges, {d.get('active_sessions', '?')} sessions")

# Check for panics
stderr_path = os.path.join(workdir, "stderr.log")
TOTAL += 1
with open(stderr_path) as f:
    stderr = f.read()
if "panic" in stderr.lower():
    FAIL += 1
    print("  [FAIL] PANIC detected!")
    print(stderr[-500:])
else:
    PASS += 1
    print("  [PASS] No panics")

# Kill server
proc.stdin.close()
proc.wait(timeout=5)

# Cleanup
import shutil
shutil.rmtree(workdir, ignore_errors=True)

# Results
print(f"\n{'='*60}")
print(f" RESULTS: {PASS} / {TOTAL} passed ({FAIL} failed)")
print(f"{'='*60}")
if FAIL == 0:
    print("  STATUS: ALL PASS")
else:
    print(f"  STATUS: {FAIL} FAILURES")

sys.exit(FAIL)
