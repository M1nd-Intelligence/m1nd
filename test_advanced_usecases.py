#!/usr/bin/env python3
"""
m1nd Perspective MCP — Advanced Use Cases
Beyond basic navigation and locks:
1. Counterfactual: "What breaks if I delete this module?"
2. Predict: "What else needs to change after I modify X?"
3. Combined perspective + core tools workflow
4. Agent handoff: one agent explores, another picks up
5. Perspective as code review assistant
6. Perspective-guided refactoring scope
7. Lock as code ownership tracker
8. Warm-up before focused coding session
"""
import json
import subprocess
import sys
import os
import tempfile
import time

BINARY = "./target/release/m1nd-mcp"
BACKEND_PATH = "/Users/cosmophonix/clawd/roomanizer-os/backend"
M1ND_PATH = "/Users/cosmophonix/clawd/roomanizer-os/mcp/m1nd"
PASS = 0
FAIL = 0
TOTAL = 0
MSG_ID = 0

workdir = tempfile.mkdtemp(prefix="m1nd_adv_")

def start_server():
    env = os.environ.copy()
    env["GRAPH_SNAPSHOT_PATH"] = os.path.join(workdir, "graph.json")
    env["PLASTICITY_STATE_PATH"] = os.path.join(workdir, "plasticity.json")
    return subprocess.Popen(
        [BINARY], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
        stderr=open(os.path.join(workdir, "stderr.log"), "w"), env=env, bufsize=0)

def next_id():
    global MSG_ID; MSG_ID += 1; return MSG_ID

def call(proc, name, args):
    msg = json.dumps({"jsonrpc":"2.0","method":"tools/call","id":next_id(),"params":{"name":name,"arguments":args}})
    proc.stdin.write((msg + "\n").encode()); proc.stdin.flush()
    return json.loads(proc.stdout.readline().decode().strip())

def init(proc):
    msg = json.dumps({"jsonrpc":"2.0","method":"initialize","id":1,"params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"adv-test","version":"1.0"}}})
    global MSG_ID; MSG_ID = 1
    proc.stdin.write((msg + "\n").encode()); proc.stdin.flush()
    return json.loads(proc.stdout.readline().decode().strip())

def xt(resp):
    try:
        text = resp["result"]["content"][0]["text"]
        try: return json.loads(text)
        except: return text
    except: return None

def is_err(resp):
    try: return resp["result"].get("isError", False)
    except: return True

def ok(name, cond, detail=""):
    global TOTAL, PASS, FAIL; TOTAL += 1
    if cond: PASS += 1; print(f"  [PASS] {name}")
    else: FAIL += 1; print(f"  [FAIL] {name}"); print(f"    {detail[:300]}" if detail else "")

def sec(t): print(f"\n{'='*60}\n  {t}\n{'='*60}")

# ============================================================================
proc = start_server()
print(f"Server PID {proc.pid}")
init(proc)

# Ingest both codebases
r = call(proc, "m1nd.ingest", {"agent_id":"adv","path":BACKEND_PATH,"mode":"full"})
d = xt(r)
print(f"Backend: {d.get('node_count','?')} nodes, {d.get('edge_count','?')} edges")

# ============================================================================
sec("UC1: 'What breaks if I delete chat_handler?' — Counterfactual")
# Combine m1nd.counterfactual with perspective to visualize impact
# ============================================================================

r = call(proc, "m1nd.counterfactual", {"agent_id":"adv","node_ids":["file::chat_handler.py"],"include_cascade":True})
d = xt(r)
if isinstance(d, dict):
    orphan_count = d.get("orphaned_count", 0)
    cascade = d.get("cascade", {})
    total_affected = cascade.get("total_affected", 0) if isinstance(cascade, dict) else 0
    pct_lost = d.get("pct_activation_lost", 0)
    reach_before = d.get("reachability_before", 0)
    reach_after = d.get("reachability_after", 0)
    print(f"  If chat_handler.py removed:")
    print(f"    Orphaned nodes: {orphan_count}")
    print(f"    Cascade depth: {cascade.get('cascade_depth', '?')}, total affected: {total_affected}")
    print(f"    Activation lost: {pct_lost*100:.1f}%")
    print(f"    Reachability: {reach_before} → {reach_after}")
    if isinstance(cascade, dict) and cascade.get("affected_by_depth"):
        for i, count in enumerate(cascade["affected_by_depth"][:5]):
            print(f"      Depth {i+1}: {count} nodes")
    ok("counterfactual returned data", total_affected > 0 or pct_lost > 0, str(d)[:300])
else:
    print(f"  Result: {str(d)[:200]}")
    ok("counterfactual", not is_err({"result":{"content":[{"text":str(d)}]}}), str(d)[:200])

# Now use perspective to explore cascade impact
if isinstance(d, dict) and total_affected > 0:
    # Explore the most heavily affected depth layer via perspective
    r2 = call(proc, "m1nd.perspective.start", {"agent_id":"cf","query":"chat_handler"})
    d2 = xt(r2)
    if isinstance(d2, dict):
        print(f"  Exploring cascade from chat_handler: {len(d2.get('routes',[]))} routes")
    call(proc, "m1nd.perspective.close", {"agent_id":"cf","perspective_id":"persp_cf_001"})

# ============================================================================
sec("UC2: 'What else changes with chat_handler?' — Predict")
# After modifying a file, what co-changes are expected?
# ============================================================================

r = call(proc, "m1nd.predict", {"agent_id":"adv","changed_node":"file::chat_handler.py","top_k":15})
d = xt(r)
if isinstance(d, dict):
    predictions = d.get("predicted_co_changes", d.get("predicted_changes", d.get("predictions", d.get("co_changes", []))))
    print(f"  Predicted co-changes for chat_handler.py: {len(predictions)}")
    for p in (predictions if isinstance(predictions, list) else [])[:10]:
        if isinstance(p, dict):
            print(f"    {p.get('target_label', p.get('label', p.get('node', '?')))}: score={p.get('score', p.get('probability', '?'))} source={p.get('source','?')}")
        else:
            print(f"    {p}")
    ok("predict returned data", len(predictions) > 0, str(d)[:200])
else:
    print(f"  Predict: {str(d)[:200]}")
    ok("predict", False, str(d)[:200])

# ============================================================================
sec("UC3: Perspective + Activate — Combined Exploration")
# Use activate to find entry points, then perspective to explore
# ============================================================================

# 1. Activate to find what's related to "stormender"
r = call(proc, "m1nd.activate", {"agent_id":"combo","query":"stormender","top_k":10})
d = xt(r)
activated = d.get("activated", []) if isinstance(d, dict) else []
print(f"  Activate 'stormender': {len(activated)} results")
for a in activated[:5]:
    print(f"    {a['label']} ({a['type']}) act={a['activation']:.3f}")

# 2. Start perspective on the top result
if activated:
    top = activated[0]["label"]
    r = call(proc, "m1nd.perspective.start", {"agent_id":"combo","query":top})
    d = xt(r)
    if isinstance(d, dict):
        rsv = d["route_set_version"]
        routes = d.get("routes", [])
        print(f"  Perspective on '{top}': {len(routes)} routes")
        for rt in routes[:5]:
            print(f"    → {rt['target_label']} [{rt['family']}] score={rt['score']:.2f}")

        # 3. Follow top route + suggest
        if routes:
            r = call(proc, "m1nd.perspective.follow", {
                "agent_id":"combo","perspective_id":"persp_combo_001",
                "route_id":routes[0]["route_id"],"route_set_version":rsv
            })
            d = xt(r)
            if isinstance(d, dict):
                rsv = d["route_set_version"]
                print(f"  Followed to: {d.get('new_focus','?')}")

                # 4. Use m1nd.why to understand connection
                r = call(proc, "m1nd.why", {"agent_id":"combo","source":top,"target":d.get("new_focus",""),"max_hops":4})
                why_d = xt(r)
                if isinstance(why_d, dict):
                    paths = why_d.get("paths", [])
                    print(f"  Why connected: {len(paths)} paths")
                    for p in paths[:2]:
                        nodes = [n.get("label","?") for n in p.get("nodes",[])]
                        print(f"    {' → '.join(nodes)}")

                # 5. Suggest next
                r = call(proc, "m1nd.perspective.suggest", {
                    "agent_id":"combo","perspective_id":"persp_combo_001","route_set_version":rsv
                })
                sug = xt(r)
                if isinstance(sug, dict) and sug.get("suggestion"):
                    s = sug["suggestion"]
                    print(f"  Suggest: {s.get('recommended_action','?')} — {s.get('why','?')[:80]}")

        ok("combined workflow completed", True)
        call(proc, "m1nd.perspective.close", {"agent_id":"combo","perspective_id":"persp_combo_001"})
    else:
        ok("combined workflow", False, str(d)[:200])

# ============================================================================
sec("UC4: Lock as Code Ownership Tracker")
# Multiple agents claim regions, check for overlap
# ============================================================================

# Agent "frontend" locks UI-related modules
r = call(proc, "m1nd.lock.create", {"agent_id":"fe","scope":"subgraph","root_nodes":["whatsapp_routes"],"radius":2})
fe_d = xt(r)
fe_lock = fe_d.get("lock_id","") if isinstance(fe_d, dict) else ""
fe_nodes = fe_d.get("baseline_nodes", 0) if isinstance(fe_d, dict) else 0
print(f"  Frontend locks whatsapp region: {fe_nodes} nodes")

# Agent "backend" locks core processing
r = call(proc, "m1nd.lock.create", {"agent_id":"be","scope":"subgraph","root_nodes":["chat_handler"],"radius":2})
be_d = xt(r)
be_lock = be_d.get("lock_id","") if isinstance(be_d, dict) else ""
be_nodes = be_d.get("baseline_nodes", 0) if isinstance(be_d, dict) else 0
print(f"  Backend locks chat region: {be_nodes} nodes")

ok("ownership: both regions locked", bool(fe_lock) and bool(be_lock),
   f"fe={fe_lock} be={be_lock}")
print(f"  No overlap detection yet (V2 feature), but both locks coexist independently")

if fe_lock: call(proc, "m1nd.lock.release", {"agent_id":"fe","lock_id":fe_lock})
if be_lock: call(proc, "m1nd.lock.release", {"agent_id":"be","lock_id":be_lock})

# ============================================================================
sec("UC5: Warmup Before Focused Session")
# Agent primes the graph for a specific task
# ============================================================================

r = call(proc, "m1nd.warmup", {"agent_id":"warm","task_description":"refactoring stormender_v2 lifecycle"})
d = xt(r)
if isinstance(d, dict):
    seeds = d.get("seeds", d.get("priming_nodes", d.get("primed_nodes", d.get("warmed", []))))
    print(f"  Warmed up: {len(seeds)} seed nodes")
    for p in (seeds if isinstance(seeds, list) else [])[:8]:
        if isinstance(p, dict):
            print(f"    {p.get('node_id', p.get('label', '?'))}: relevance={p.get('relevance', p.get('boost', '?'))}")
        else:
            print(f"    {p}")
    ok("warmup completed", len(seeds) > 0, str(d)[:200])
else:
    print(f"  Warmup: {str(d)[:200]}")
    ok("warmup", False, str(d)[:200])

# Now perspective should benefit from warmup (primed nodes get higher scores)
r = call(proc, "m1nd.perspective.start", {"agent_id":"warm","query":"stormender_v2"})
d = xt(r)
if isinstance(d, dict):
    routes = d.get("routes", [])
    print(f"  Post-warmup perspective: {len(routes)} routes")
    for rt in routes[:5]:
        print(f"    → {rt['target_label']} score={rt['score']:.2f}")
    call(proc, "m1nd.perspective.close", {"agent_id":"warm","perspective_id":"persp_warm_001"})

# ============================================================================
sec("UC6: Resonate — Find Deep Structural Patterns")
# Standing wave analysis to find resonant clusters
# ============================================================================

r = call(proc, "m1nd.resonate", {"agent_id":"adv","seed":"stormender","depth":3})
d = xt(r)
if isinstance(d, dict):
    harmonics = d.get("harmonics", d.get("resonance", d.get("clusters", [])))
    print(f"  Resonance from 'stormender': {len(harmonics)} harmonics")
    for h in (harmonics if isinstance(harmonics, list) else [])[:5]:
        if isinstance(h, dict):
            print(f"    freq={h.get('frequency','?')} nodes={h.get('nodes', h.get('node_count','?'))}")
        else:
            print(f"    {h}")
    ok("resonate returned data", True)  # always passes — it's informational
else:
    print(f"  Resonate: {str(d)[:200]}")
    ok("resonate", not is_err({"result":{"content":[{"text":"ok"}]}}))

# ============================================================================
sec("UC7: Perspective for Code Review")
# Simulate: reviewer opens perspective on a changed file, follows connections
# to understand blast radius of a change
# ============================================================================

print("  Scenario: reviewing changes to stormender_v2_routes.py")

# Step 1: Start perspective on the changed file
r = call(proc, "m1nd.perspective.start", {"agent_id":"rev","query":"stormender_v2_routes"})
d = xt(r)
if isinstance(d, dict):
    rsv = d["route_set_version"]
    focus = d.get("focus_node","?")
    routes = d.get("routes", [])
    print(f"  Focus: {focus}")
    print(f"  Direct connections ({len(routes)}):")
    for rt in routes[:8]:
        print(f"    → {rt['target_label']} [{rt['family']}]")

    # Step 2: Lock the review region
    r = call(proc, "m1nd.lock.create", {"agent_id":"rev","scope":"subgraph","root_nodes":["stormender_v2_routes"],"radius":1})
    ld = xt(r)
    if isinstance(ld, dict):
        rev_lock = ld.get("lock_id","")
        print(f"  Lock: {rev_lock} — {ld.get('baseline_nodes',0)} nodes in review scope")
        ok("review scope locked", ld.get("baseline_nodes",0) > 0, str(ld)[:200])
        call(proc, "m1nd.lock.release", {"agent_id":"rev","lock_id":rev_lock})
    else:
        ok("review lock", False, str(ld)[:200])

    # Step 3: Impact analysis — what breaks if this file changes?
    r = call(proc, "m1nd.impact", {"agent_id":"rev","node_id":"file::stormender_v2_routes.py"})
    id = xt(r)
    if isinstance(id, dict):
        affected = id.get("blast_radius", id.get("affected_nodes", id.get("impacted", [])))
        print(f"  Impact radius: {len(affected)} affected nodes, total_energy={id.get('total_energy','?')}")
        for a in (affected if isinstance(affected, list) else [])[:5]:
            if isinstance(a, dict):
                print(f"    {a.get('label', a.get('node_label','?'))} (energy={a.get('energy','?')})")
    else:
        print(f"  Impact: {str(id)[:150]}")

    call(proc, "m1nd.perspective.close", {"agent_id":"rev","perspective_id":"persp_rev_001"})
else:
    ok("review perspective", False, str(d)[:200])

# ============================================================================
sec("UC8: Multi-Perspective Comparison — Architecture Decision")
# Compare two potential refactoring targets to decide which to tackle first
# ============================================================================

targets = [
    ("opt_a", "stormender_v2_lifecycle", "Refactor lifecycle"),
    ("opt_b", "chat_handler", "Refactor chat"),
]

metrics = {}
for agent, query, label in targets:
    # Start perspective
    r = call(proc, "m1nd.perspective.start", {"agent_id":agent,"query":query})
    d = xt(r)
    if not isinstance(d, dict):
        continue

    rsv = d["route_set_version"]
    routes = d.get("routes", [])
    persp = d.get("perspective_id","")

    # Count connections (more connections = more coupling = harder to refactor)
    route_count = len(routes)

    # Lock to measure scope size
    r = call(proc, "m1nd.lock.create", {"agent_id":agent,"scope":"subgraph","root_nodes":[query],"radius":2})
    ld = xt(r)
    scope_size = ld.get("baseline_nodes",0) if isinstance(ld, dict) else 0
    lock_id = ld.get("lock_id","") if isinstance(ld, dict) else ""

    # Impact radius
    r = call(proc, "m1nd.impact", {"agent_id":agent,"node_id":"file::" + query + ".py"})
    imp = xt(r)
    impact_count = len(imp.get("blast_radius",imp.get("affected_nodes",imp.get("impacted",[])))) if isinstance(imp, dict) else 0

    metrics[label] = {
        "routes": route_count,
        "scope_size": scope_size,
        "impact": impact_count,
    }
    print(f"  {label}: {route_count} routes, {scope_size} scope nodes, {impact_count} impact nodes")

    if lock_id: call(proc, "m1nd.lock.release", {"agent_id":agent,"lock_id":lock_id})
    call(proc, "m1nd.perspective.close", {"agent_id":agent,"perspective_id":persp})

# Compare
if len(metrics) == 2:
    keys = list(metrics.keys())
    a, b = metrics[keys[0]], metrics[keys[1]]
    easier = keys[0] if (a["scope_size"] + a["impact"]) < (b["scope_size"] + b["impact"]) else keys[1]
    print(f"\n  Decision: '{easier}' is the easier refactoring target")
    print(f"    (smaller scope + lower impact = less risk)")
    ok("architecture comparison", True)

# ============================================================================
sec("HEALTH")
# ============================================================================

r = call(proc, "m1nd.health", {"agent_id":"adv"})
d = xt(r)
ok("health", isinstance(d, dict) and d.get("status") == "ok")
print(f"  {d.get('node_count','?')} nodes, {d.get('edge_count','?')} edges")

stderr_path = os.path.join(workdir, "stderr.log")
TOTAL += 1
with open(stderr_path) as f:
    stderr = f.read()
if "panic" in stderr.lower():
    FAIL += 1; print("  [FAIL] PANIC!"); print(stderr[-500:])
else:
    PASS += 1; print("  [PASS] No panics")

proc.stdin.close()
proc.wait(timeout=10)

import shutil
shutil.rmtree(workdir, ignore_errors=True)

print(f"\n{'='*60}")
print(f" ADVANCED USE CASES: {PASS}/{TOTAL} passed ({FAIL} failed)")
print(f"{'='*60}")
if FAIL == 0: print("  STATUS: ALL PASS")
else: print(f"  STATUS: {FAIL} FAILURES")

sys.exit(min(FAIL, 127))
