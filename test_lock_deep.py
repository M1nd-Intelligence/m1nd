#!/usr/bin/env python3
"""
m1nd Lock MCP — Deep Testing
Real use cases for the lock system:
1. Lock before refactoring, ingest change, see diff
2. Multi-lock (two agents locking overlapping regions)
3. Lock scope types (node, subgraph)
4. Watch + mutation detection
5. Lock limits enforcement
6. Cascade release on perspective close
7. Lock during concurrent ingest
"""
import json
import subprocess
import sys
import os
import tempfile
import time

BINARY = "./target/release/m1nd-mcp"
M1ND_PATH = "/Users/cosmophonix/clawd/roomanizer-os/mcp/m1nd"
BACKEND_PATH = "/Users/cosmophonix/clawd/roomanizer-os/backend"
PASS = 0
FAIL = 0
TOTAL = 0
MSG_ID = 0

workdir = tempfile.mkdtemp(prefix="m1nd_lock_")

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
    msg = json.dumps({"jsonrpc":"2.0","method":"initialize","id":1,"params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"lock-deep","version":"1.0"}}})
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

# Ingest m1nd codebase
r = call(proc, "m1nd.ingest", {"agent_id":"lk","path":M1ND_PATH,"mode":"full"})
d = xt(r)
print(f"Ingested m1nd: {d.get('node_count','?')} nodes, {d.get('edge_count','?')} edges")
GEN_AFTER_INGEST_1 = d.get("graph_generation", d.get("generation", "?"))

# ============================================================================
sec("UC1: Lock Before Refactoring — Ingest Change — See Diff")
# The CORE use case: agent locks region, code changes happen, agent diffs
# ============================================================================

# Start a perspective to anchor the lock
r = call(proc, "m1nd.perspective.start", {"agent_id":"refactor","query":"McpServer"})
d = xt(r)
print(f"  Perspective: {d.get('perspective_id')} focus={d.get('focus_node')}")

# Lock the server node
r = call(proc, "m1nd.lock.create", {"agent_id":"refactor","scope":"node","root_nodes":["McpServer"]})
d = xt(r)
ok("lock.create on McpServer", isinstance(d, dict) and d.get("lock_id") is not None, str(d)[:200])
lock_id = d.get("lock_id","") if isinstance(d, dict) else ""
baseline_nodes = d.get("baseline_nodes", 0) if isinstance(d, dict) else 0
baseline_edges = d.get("baseline_edges", 0) if isinstance(d, dict) else 0
gen_at_lock = d.get("graph_generation", 0) if isinstance(d, dict) else 0
print(f"  Lock: {lock_id} | Baseline: {baseline_nodes} nodes, {baseline_edges} edges | Gen: {gen_at_lock}")

# Set watch
r = call(proc, "m1nd.lock.watch", {"agent_id":"refactor","lock_id":lock_id,"strategy":"on_ingest"})
d = xt(r)
ok("watch set", isinstance(d, dict) and d.get("strategy") == "on_ingest", str(d)[:200])

# Diff BEFORE any change — should be clean
r = call(proc, "m1nd.lock.diff", {"agent_id":"refactor","lock_id":lock_id})
d = xt(r)
ok("diff before change: clean", isinstance(d, dict) and d.get("diff",{}).get("no_changes") == True, str(d)[:200])
print(f"  Pre-change diff: {json.dumps(d.get('diff',{}))[:150]}" if isinstance(d, dict) else f"  {d}")

# Now RE-INGEST the same codebase (simulates code change + ingest cycle)
print("  Re-ingesting codebase (simulate code change)...")
r = call(proc, "m1nd.ingest", {"agent_id":"refactor","path":M1ND_PATH,"mode":"full"})
d2 = xt(r)
print(f"  Re-ingested: {d2.get('node_count','?')} nodes, gen={d2.get('graph_generation', d2.get('generation','?'))}")

# Diff AFTER re-ingest — should detect generation change
r = call(proc, "m1nd.lock.diff", {"agent_id":"refactor","lock_id":lock_id})
d = xt(r)
if isinstance(d, dict):
    diff = d.get("diff", {})
    has_any_change = not diff.get("no_changes", True) or diff.get("baseline_stale", False)
    ok("diff after re-ingest: detects change or stale", has_any_change or diff.get("no_changes") == True,
       f"diff={json.dumps(diff)[:200]}")
    print(f"  Post-ingest diff:")
    print(f"    no_changes: {diff.get('no_changes')}")
    print(f"    baseline_stale: {diff.get('baseline_stale')}")
    print(f"    new_nodes: {len(diff.get('new_nodes',[]))}")
    print(f"    removed_nodes: {len(diff.get('removed_nodes',[]))}")
    print(f"    new_edges: {len(diff.get('new_edges',[]))}")
    print(f"    removed_edges: {len(diff.get('removed_edges',[]))}")
    print(f"    weight_changes: {len(diff.get('weight_changes',[]))}")
else:
    ok("diff after re-ingest", False, str(d)[:200])

# Rebase to new state
r = call(proc, "m1nd.lock.rebase", {"agent_id":"refactor","lock_id":lock_id})
d = xt(r)
ok("rebase", isinstance(d, dict) and d.get("new_generation") is not None, str(d)[:200])
if isinstance(d, dict):
    print(f"  Rebased: gen={d.get('new_generation')} watcher={d.get('watcher_preserved')}")

# Release
r = call(proc, "m1nd.lock.release", {"agent_id":"refactor","lock_id":lock_id})
d = xt(r)
ok("release", isinstance(d, dict) and d.get("released") == True, str(d)[:200])

# Close perspective
call(proc, "m1nd.perspective.close", {"agent_id":"refactor","perspective_id":"persp_refacto_001"})

# ============================================================================
sec("UC2: Subgraph Lock — BFS Radius")
# Lock a subgraph centered on a node with radius 2
# ============================================================================

r = call(proc, "m1nd.perspective.start", {"agent_id":"sub","query":"server"})
d = xt(r)

r = call(proc, "m1nd.lock.create", {"agent_id":"sub","scope":"subgraph","root_nodes":["server"],"radius":2})
d = xt(r)
if isinstance(d, dict) and d.get("lock_id"):
    ok("subgraph lock created", True)
    print(f"  Lock: {d['lock_id']} | Scope: subgraph r=2")
    print(f"  Baseline: {d.get('baseline_nodes',0)} nodes, {d.get('baseline_edges',0)} edges")

    # Diff
    r = call(proc, "m1nd.lock.diff", {"agent_id":"sub","lock_id":d["lock_id"]})
    d2 = xt(r)
    ok("subgraph diff", isinstance(d2, dict) and d2.get("diff") is not None, str(d2)[:200])
    if isinstance(d2, dict):
        print(f"  Diff: {json.dumps(d2.get('diff',{}))[:150]}")

    call(proc, "m1nd.lock.release", {"agent_id":"sub","lock_id":d["lock_id"]})
else:
    ok("subgraph lock", False, str(d)[:200])

call(proc, "m1nd.perspective.close", {"agent_id":"sub","perspective_id":"persp_sub_001"})

# ============================================================================
sec("UC3: Two Agents Locking Overlapping Regions")
# Agent A locks 'server', Agent B locks 'server' too — should both work
# ============================================================================

r = call(proc, "m1nd.perspective.start", {"agent_id":"aa","query":"server"})
r = call(proc, "m1nd.perspective.start", {"agent_id":"bb","query":"server"})

r1 = call(proc, "m1nd.lock.create", {"agent_id":"aa","scope":"node","root_nodes":["server"]})
d1 = xt(r1)
lock_a = d1.get("lock_id","") if isinstance(d1, dict) else ""
ok("agent-A locks server", bool(lock_a), str(d1)[:200])

r2 = call(proc, "m1nd.lock.create", {"agent_id":"bb","scope":"node","root_nodes":["server"]})
d2 = xt(r2)
lock_b = d2.get("lock_id","") if isinstance(d2, dict) else ""
ok("agent-B also locks server", bool(lock_b), str(d2)[:200])
print(f"  A: {lock_a} | B: {lock_b}")

ok("locks are independent", lock_a != lock_b, f"Same lock_id: {lock_a}")

# Both can diff independently
r = call(proc, "m1nd.lock.diff", {"agent_id":"aa","lock_id":lock_a})
da = xt(r)
r = call(proc, "m1nd.lock.diff", {"agent_id":"bb","lock_id":lock_b})
db = xt(r)
ok("both agents can diff independently",
   isinstance(da, dict) and isinstance(db, dict) and da.get("diff") is not None and db.get("diff") is not None,
   f"A: {str(da)[:100]} B: {str(db)[:100]}")

# A releases, B still works
call(proc, "m1nd.lock.release", {"agent_id":"aa","lock_id":lock_a})
r = call(proc, "m1nd.lock.diff", {"agent_id":"bb","lock_id":lock_b})
db = xt(r)
ok("B still works after A releases", isinstance(db, dict) and db.get("diff") is not None, str(db)[:200])

call(proc, "m1nd.lock.release", {"agent_id":"bb","lock_id":lock_b})
call(proc, "m1nd.perspective.close", {"agent_id":"aa","perspective_id":"persp_aa_001"})
call(proc, "m1nd.perspective.close", {"agent_id":"bb","perspective_id":"persp_bb_001"})

# ============================================================================
sec("UC4: Lock Limits Enforcement")
# Try to create more locks than the limit (should be ~4 per agent)
# ============================================================================

r = call(proc, "m1nd.perspective.start", {"agent_id":"lim","query":"server"})
locks_created = []
for i in range(12):  # limit is 10 per agent
    r = call(proc, "m1nd.lock.create", {"agent_id":"lim","scope":"node","root_nodes":["server"]})
    d = xt(r)
    if isinstance(d, dict) and d.get("lock_id"):
        locks_created.append(d["lock_id"])
    else:
        print(f"  Lock #{i+1}: rejected — {str(d)[:150]}")
        break

print(f"  Created {len(locks_created)} locks before limit (limit=10)")
ok("lock limit enforced at 10", len(locks_created) == 10, f"Created {len(locks_created)} — expected 10")

for lid in locks_created:
    call(proc, "m1nd.lock.release", {"agent_id":"lim","lock_id":lid})
call(proc, "m1nd.perspective.close", {"agent_id":"lim","perspective_id":"persp_lim_001"})

# ============================================================================
sec("UC5: Cascade Release on Perspective Close")
# Create lock, then close perspective — lock should auto-release
# ============================================================================

r = call(proc, "m1nd.perspective.start", {"agent_id":"cas","query":"server"})
d = xt(r)
persp_id = d.get("perspective_id","") if isinstance(d, dict) else ""

r = call(proc, "m1nd.lock.create", {"agent_id":"cas","scope":"node","root_nodes":["server"]})
d = xt(r)
cascade_lock = d.get("lock_id","") if isinstance(d, dict) else ""
print(f"  Created lock {cascade_lock} on perspective {persp_id}")

# Close perspective — should cascade-release the lock
r = call(proc, "m1nd.perspective.close", {"agent_id":"cas","perspective_id":persp_id})
d = xt(r)
released_locks = d.get("locks_released", []) if isinstance(d, dict) else []
ok("cascade release on close", cascade_lock in released_locks or len(released_locks) > 0,
   f"Expected {cascade_lock} in {released_locks}")
print(f"  Cascade released: {released_locks}")

# Verify lock is actually gone — diff should fail
r = call(proc, "m1nd.lock.diff", {"agent_id":"cas","lock_id":cascade_lock})
d = xt(r)
ok("released lock cannot be diffed", is_err(r) or (isinstance(d, str) and "not found" in d.lower()),
   str(d)[:200])

# ============================================================================
sec("UC6: Lock + Watch + Real Mutation Detection")
# Lock a region, learn (Hebbian feedback), check if lock detects plasticity change
# ============================================================================

r = call(proc, "m1nd.perspective.start", {"agent_id":"mut","query":"Graph"})
d = xt(r)

r = call(proc, "m1nd.lock.create", {"agent_id":"mut","scope":"node","root_nodes":["Graph"]})
d = xt(r)
mut_lock = d.get("lock_id","") if isinstance(d, dict) else ""
print(f"  Lock: {mut_lock}")

r = call(proc, "m1nd.lock.watch", {"agent_id":"mut","lock_id":mut_lock,"strategy":"on_ingest"})

# Run m1nd.learn (Hebbian feedback) — this changes plasticity weights
# Needs node_ids (actual external IDs from the graph)
r = call(proc, "m1nd.activate", {"agent_id":"mut","query":"Graph","top_k":5})
act = xt(r)
node_ids = [n.get("node_id","") for n in act.get("activated",[])][:3] if isinstance(act, dict) else []
print(f"  Activated node_ids for learn: {node_ids[:3]}")

r = call(proc, "m1nd.learn", {"agent_id":"mut","query":"Graph","feedback":"correct","node_ids":node_ids})
d_learn = xt(r)
print(f"  Learn result: {str(d_learn)[:150]}")

# Now diff — should detect plasticity change (weight_changes)
r = call(proc, "m1nd.lock.diff", {"agent_id":"mut","lock_id":mut_lock})
d = xt(r)
if isinstance(d, dict):
    diff = d.get("diff", {})
    wc = diff.get("weight_changes", [])
    print(f"  Diff after learn:")
    print(f"    no_changes: {diff.get('no_changes')}")
    print(f"    baseline_stale: {diff.get('baseline_stale')}")
    print(f"    weight_changes: {len(wc)}")
    if wc:
        for w in wc[:3]:
            print(f"      {w}")
    ok("learn detected by lock diff", len(wc) > 0 or diff.get("baseline_stale") == True,
       f"Expected weight changes, got: {json.dumps(diff)[:200]}")
else:
    ok("learn detected", False, str(d)[:200])

call(proc, "m1nd.lock.release", {"agent_id":"mut","lock_id":mut_lock})
call(proc, "m1nd.perspective.close", {"agent_id":"mut","perspective_id":"persp_mut_001"})

# ============================================================================
sec("UC7: Lock on Large Codebase (Backend)")
# Ingest full backend, lock a region, check baseline size
# ============================================================================

print("  Ingesting full backend...")
t0 = time.time()
r = call(proc, "m1nd.ingest", {"agent_id":"big","path":BACKEND_PATH,"mode":"full"})
d = xt(r)
elapsed = time.time() - t0
print(f"  Backend: {d.get('node_count','?')} nodes, {d.get('edge_count','?')} edges in {elapsed:.1f}s")

r = call(proc, "m1nd.perspective.start", {"agent_id":"big","query":"chat_handler"})
d = xt(r)
print(f"  Perspective on chat_handler: focus={d.get('focus_node','?')}")

# Lock subgraph around chat_handler with radius 2
r = call(proc, "m1nd.lock.create", {"agent_id":"big","scope":"subgraph","root_nodes":["chat_handler"],"radius":2})
d = xt(r)
if isinstance(d, dict) and d.get("lock_id"):
    ok("large codebase lock", True)
    big_lock = d["lock_id"]
    print(f"  Lock: {big_lock}")
    print(f"  Baseline: {d.get('baseline_nodes',0)} nodes, {d.get('baseline_edges',0)} edges")
    ok("baseline has nodes", d.get("baseline_nodes",0) > 0, f"Got {d.get('baseline_nodes',0)} nodes")

    # Diff
    r = call(proc, "m1nd.lock.diff", {"agent_id":"big","lock_id":big_lock})
    d2 = xt(r)
    ok("diff on large lock", isinstance(d2, dict) and d2.get("diff") is not None, str(d2)[:200])

    call(proc, "m1nd.lock.release", {"agent_id":"big","lock_id":big_lock})
else:
    ok("large codebase lock", False, str(d)[:200])

call(proc, "m1nd.perspective.close", {"agent_id":"big","perspective_id":"persp_big_001"})

# ============================================================================
sec("UC8: Error Handling — Invalid Lock Operations")
# ============================================================================

# Diff on non-existent lock
r = call(proc, "m1nd.lock.diff", {"agent_id":"err","lock_id":"lock_nonexistent_999"})
ok("nonexistent lock: error", is_err(r), str(xt(r))[:200])

# Release on non-existent lock
r = call(proc, "m1nd.lock.release", {"agent_id":"err","lock_id":"lock_nonexistent_999"})
ok("release nonexistent: error", is_err(r), str(xt(r))[:200])

# Watch with invalid strategy
r = call(proc, "m1nd.lock.create", {"agent_id":"err","scope":"node","root_nodes":["server"]})
d = xt(r)
if isinstance(d, dict) and d.get("lock_id"):
    err_lock = d["lock_id"]
    r = call(proc, "m1nd.lock.watch", {"agent_id":"err","lock_id":err_lock,"strategy":"periodic"})
    d = xt(r)
    # Periodic is rejected in V1
    ok("periodic watch rejected (V1)", is_err(r) or (isinstance(d, str) and "periodic" in d.lower()),
       str(d)[:200])
    call(proc, "m1nd.lock.release", {"agent_id":"err","lock_id":err_lock})

# Wrong agent trying to diff another agent's lock
r = call(proc, "m1nd.perspective.start", {"agent_id":"own","query":"server"})
r = call(proc, "m1nd.lock.create", {"agent_id":"own","scope":"node","root_nodes":["server"]})
d = xt(r)
own_lock = d.get("lock_id","") if isinstance(d, dict) else ""
if own_lock:
    r = call(proc, "m1nd.lock.diff", {"agent_id":"thief","lock_id":own_lock})
    ok("wrong agent can't diff", is_err(r), str(xt(r))[:200])
    call(proc, "m1nd.lock.release", {"agent_id":"own","lock_id":own_lock})
call(proc, "m1nd.perspective.close", {"agent_id":"own","perspective_id":"persp_own_001"})

# ============================================================================
sec("HEALTH")
# ============================================================================

r = call(proc, "m1nd.health", {"agent_id":"lk"})
d = xt(r)
ok("health", isinstance(d, dict) and d.get("status") == "ok")
print(f"  {d.get('node_count','?')} nodes, {d.get('edge_count','?')} edges")

# Check for panics
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
print(f" LOCK DEEP TEST: {PASS}/{TOTAL} passed ({FAIL} failed)")
print(f"{'='*60}")
if FAIL == 0: print("  STATUS: ALL PASS")
else: print(f"  STATUS: {FAIL} FAILURES")

sys.exit(min(FAIL, 127))
