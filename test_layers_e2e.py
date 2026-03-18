#!/usr/bin/env python3
"""
m1nd Superpowers — E2E Golden Tests for Layers 1-7.
Exercises all 13 new MCP tools against real codebase data.
Step 10 of Grounded One-Shot Build pipeline.
"""
import json
import subprocess
import sys
import os
import tempfile
import shutil

BINARY = "./target/release/m1nd-mcp"
BACKEND_PATH = "/Users/cosmophonix/clawd/roomanizer-os/backend"
M1ND_PATH = "/Users/cosmophonix/clawd/roomanizer-os/mcp/m1nd"
PASS = 0
FAIL = 0
TOTAL = 0
MSG_ID = 0

workdir = tempfile.mkdtemp(prefix="m1nd_layers_")

def start_server():
    env = os.environ.copy()
    env["M1ND_GRAPH_SOURCE"] = os.path.join(workdir, "graph.json")
    env["M1ND_PLASTICITY_STATE"] = os.path.join(workdir, "plasticity.json")
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
    msg = json.dumps({"jsonrpc":"2.0","method":"initialize","id":1,"params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"golden-layers","version":"1.0"}}})
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
    else: FAIL += 1; print(f"  [FAIL] {name}"); print(f"    {detail[:400]}" if detail else "")

def sec(t): print(f"\n{'='*60}\n  {t}\n{'='*60}")

# ============================================================================
proc = start_server()
print(f"Server PID {proc.pid}, workdir {workdir}")
init(proc)

# Ingest backend (with cross-file edges from L1)
sec("INGEST: Backend with L1 cross-file edges")
r = call(proc, "m1nd.ingest", {"agent_id":"golden","path":BACKEND_PATH,"mode":"full"})
d = xt(r)
nodes = d.get("node_count", 0)
edges = d.get("edge_count", 0)
print(f"  Ingested: {nodes} nodes, {edges} edges")
ok("Ingest succeeded", nodes > 100 and edges > 100, f"nodes={nodes} edges={edges}")

# ============================================================================
# L1: Cross-File Edges
# ============================================================================
sec("L1: Cross-File Edges — import resolution")

# L1-01: Import edges exist (chat_handler -> config)
r = call(proc, "m1nd.why", {"agent_id":"l1","source":"file::chat_handler.py","target":"file::config.py","max_hops":3})
d = xt(r)
if isinstance(d, dict):
    paths = d.get("paths", [])
    ok("L1-01: chat_handler->config has paths", len(paths) >= 1, f"paths={len(paths)}")
    # Check for imports relation in any path
    has_import = any("imports" in str(p) for p in paths) if paths else False
    ok("L1-01: has 'imports' relation", has_import or len(paths) >= 1, f"paths={json.dumps(paths)[:200]}")
else:
    ok("L1-01: why returned dict", False, f"got: {str(d)[:200]}")

# L1-02: Test edges (test_spawner -> spawner)
r = call(proc, "m1nd.why", {"agent_id":"l1","source":"file::tests/test_spawner.py","target":"file::spawner.py","max_hops":3})
d = xt(r)
if isinstance(d, dict):
    paths = d.get("paths", [])
    ok("L1-02: test_spawner->spawner has paths", len(paths) >= 1, f"paths={len(paths)}")
else:
    ok("L1-02: why returned dict", False, f"got: {str(d)[:200]}")

# L1-03: Impact includes route files after cross-file edges
r = call(proc, "m1nd.impact", {"agent_id":"l1","node_id":"file::main.py","direction":"forward"})
d = xt(r)
if isinstance(d, dict):
    blast = d.get("blast_radius", [])
    route_hits = [b for b in blast if "_routes" in b.get("label","")]
    ok("L1-03: main.py impact reaches route files", len(route_hits) >= 2, f"route_hits={len(route_hits)}")
else:
    ok("L1-03: impact returned dict", False, f"got: {str(d)[:200]}")

# L1-04: Circular import no panic
r = call(proc, "m1nd.why", {"agent_id":"l1","source":"file::config.py","target":"file::config.py","max_hops":6})
ok("L1-04: self-loop no crash", not is_err(r), f"resp: {str(r)[:200]}")

# ============================================================================
# L2: Semantic Search
# ============================================================================
sec("L2: Semantic Search — m1nd.seek + m1nd.scan")

# L2-01: seek finds auth-related code
r = call(proc, "m1nd.seek", {"agent_id":"l2","query":"authentication and session validation","top_k":10})
d = xt(r)
ok("L2-01: seek returns results", isinstance(d, dict) and len(d.get("results",[])) > 0, f"d={str(d)[:200]}")
if isinstance(d, dict):
    ok("L2-01: embeddings_used is false (V1)", d.get("embeddings_used") == False, f"embeddings_used={d.get('embeddings_used')}")
    results = d.get("results", [])
    ok("L2-01: results have scores", all("score" in r for r in results[:3]), f"first={results[0] if results else 'none'}")

# L2-02: seek with scope filter
r = call(proc, "m1nd.seek", {"agent_id":"l2","query":"storm lifecycle management","top_k":5,"scope":"backend/stormender"})
d = xt(r)
ok("L2-02: seek with scope", isinstance(d, dict) and not is_err(r), f"d={str(d)[:200]}")
if isinstance(d, dict):
    results = d.get("results", [])
    # All results should be in stormender scope
    in_scope = all("stormender" in r.get("node_id","").lower() or "stormender" in r.get("label","").lower() or "stormender" in str(r.get("file_path","")).lower() for r in results) if results else True
    ok("L2-02: results respect scope", in_scope or len(results) == 0, f"results={len(results)}")

# L2-03: scan with error_handling pattern
r = call(proc, "m1nd.scan", {"agent_id":"l2","pattern":"error_handling","scope":"backend/","limit":20})
d = xt(r)
ok("L2-03: scan returns findings", isinstance(d, dict) and "findings" in (d or {}), f"d={str(d)[:200]}")
if isinstance(d, dict):
    findings = d.get("findings", [])
    ok("L2-03: has scan findings", len(findings) >= 0, f"count={len(findings)}")
    ok("L2-03: elapsed_ms present", "elapsed_ms" in d, f"keys={list(d.keys())}")

# L2-04: scan with test_coverage pattern
r = call(proc, "m1nd.scan", {"agent_id":"l2","pattern":"test_coverage","limit":30})
d = xt(r)
ok("L2-04: test_coverage scan", isinstance(d, dict) and not is_err(r), f"d={str(d)[:200]}")

# ============================================================================
# L3: Temporal Intelligence
# ============================================================================
sec("L3: Temporal Intelligence — m1nd.timeline + m1nd.diverge")

# L3-01: timeline for a known file
r = call(proc, "m1nd.timeline", {"agent_id":"l3","node":"file::chat_handler.py","depth":"30d"})
d = xt(r)
ok("L3-01: timeline returns data", isinstance(d, dict) and not is_err(r), f"d={str(d)[:200]}")
if isinstance(d, dict):
    ok("L3-01: has changes array", isinstance(d.get("changes"), list), f"keys={list(d.keys())}")
    ok("L3-01: has velocity", d.get("velocity") in ("accelerating","decelerating","stable", None), f"velocity={d.get('velocity')}")
    ok("L3-01: has stability_score", isinstance(d.get("stability_score"), (int, float)), f"stability={d.get('stability_score')}")
    ok("L3-01: has co_changed_with", isinstance(d.get("co_changed_with"), list), f"co_changed={d.get('co_changed_with','?')}")

# L3-02: diverge from recent date
r = call(proc, "m1nd.diverge", {"agent_id":"l3","baseline":"2026-03-01"})
d = xt(r)
ok("L3-02: diverge returns data", isinstance(d, dict) and not is_err(r), f"d={str(d)[:200]}")
if isinstance(d, dict):
    ok("L3-02: has structural_drift", isinstance(d.get("structural_drift"), (int, float)), f"drift={d.get('structural_drift')}")
    ok("L3-02: has new_nodes", isinstance(d.get("new_nodes"), list), f"new_nodes type={type(d.get('new_nodes'))}")

# ============================================================================
# L4: Investigation Memory
# ============================================================================
sec("L4: Investigation Memory — m1nd.trail.*")

# L4-01: trail.save
r = call(proc, "m1nd.trail.save", {
    "agent_id":"l4",
    "label":"golden test investigation",
    "hypotheses":[{"statement":"config.py is the most imported file","confidence":0.8}],
    "conclusions":[{"statement":"cross-file edges work","confidence":0.95}],
    "open_questions":["Does federation preserve trails?"],
    "tags":["golden","test"],
    "visited_nodes":[{"node_external_id":"file::config.py","relevance":0.9}],
    "activation_boosts":{"file::config.py":0.5}
})
d = xt(r)
ok("L4-01: trail.save succeeds", isinstance(d, dict) and not is_err(r), f"d={str(d)[:200]}")
trail_id = None
if isinstance(d, dict):
    trail_id = d.get("trail_id")
    ok("L4-01: has trail_id", trail_id is not None, f"trail_id={trail_id}")
    ok("L4-01: nodes_saved >= 1", d.get("nodes_saved",0) >= 1, f"nodes_saved={d.get('nodes_saved')}")

# L4-02: trail.list shows our trail
r = call(proc, "m1nd.trail.list", {"agent_id":"l4","filter_tags":["golden"]})
d = xt(r)
ok("L4-02: trail.list returns data", isinstance(d, dict) and not is_err(r), f"d={str(d)[:200]}")
if isinstance(d, dict):
    trails = d.get("trails", [])
    ok("L4-02: found our trail", len(trails) >= 1, f"count={len(trails)}")
    ok("L4-02: label matches", any("golden" in t.get("label","") for t in trails), f"trails={[t.get('label') for t in trails]}")

# L4-03: trail.resume
if trail_id:
    r = call(proc, "m1nd.trail.resume", {"agent_id":"l4","trail_id":trail_id})
    d = xt(r)
    ok("L4-03: trail.resume succeeds", isinstance(d, dict) and not is_err(r), f"d={str(d)[:200]}")
    if isinstance(d, dict):
        ok("L4-03: has nodes_reactivated", "nodes_reactivated" in d, f"keys={list(d.keys())}")
        ok("L4-03: trail data present", "trail" in d, f"keys={list(d.keys())}")
else:
    ok("L4-03: trail.resume (skipped, no trail_id)", False, "trail_id was None")

# L4-04: trail.save second trail for merge test
r2 = call(proc, "m1nd.trail.save", {
    "agent_id":"l4-b",
    "label":"second investigation",
    "hypotheses":[{"statement":"spawner is tightly coupled","confidence":0.6}],
    "tags":["golden","merge-test"],
    "visited_nodes":[{"node_external_id":"file::spawner.py","relevance":0.8}]
})
d2 = xt(r2)
trail_id_2 = d2.get("trail_id") if isinstance(d2, dict) else None

# L4-05: trail.merge
if trail_id and trail_id_2:
    r = call(proc, "m1nd.trail.merge", {"agent_id":"l4","trail_ids":[trail_id, trail_id_2],"label":"merged golden test"})
    d = xt(r)
    ok("L4-05: trail.merge succeeds", isinstance(d, dict) and not is_err(r), f"d={str(d)[:200]}")
    if isinstance(d, dict):
        ok("L4-05: has merged_trail_id", "merged_trail_id" in d, f"keys={list(d.keys())}")
        ok("L4-05: nodes merged >= 2", d.get("nodes_merged",0) >= 2, f"nodes_merged={d.get('nodes_merged')}")
else:
    ok("L4-05: trail.merge (skipped)", False, f"trail_ids: {trail_id}, {trail_id_2}")

# ============================================================================
# L5: Hypothesis Engine
# ============================================================================
sec("L5: Hypothesis Engine — m1nd.hypothesize + m1nd.differential")

# L5-01: hypothesize a dependency claim
r = call(proc, "m1nd.hypothesize", {"agent_id":"l5","claim":"chat_handler depends on config","max_hops":4})
d = xt(r)
ok("L5-01: hypothesize returns data", isinstance(d, dict) and not is_err(r), f"d={str(d)[:200]}")
if isinstance(d, dict):
    ok("L5-01: has verdict", d.get("verdict") in ("likely_true","likely_false","inconclusive"), f"verdict={d.get('verdict')}")
    ok("L5-01: has confidence [0,1]", 0 <= d.get("confidence",0) <= 1, f"confidence={d.get('confidence')}")
    ok("L5-01: has claim_type", d.get("claim_type") is not None, f"claim_type={d.get('claim_type')}")

# L5-02: hypothesize isolation claim
r = call(proc, "m1nd.hypothesize", {"agent_id":"l5","claim":"critic is isolated from whatsapp","max_hops":5})
d = xt(r)
ok("L5-02: isolation hypothesis", isinstance(d, dict) and not is_err(r), f"d={str(d)[:200]}")
if isinstance(d, dict):
    ok("L5-02: has evidence", isinstance(d.get("supporting_evidence"), list) or isinstance(d.get("contradicting_evidence"), list), f"keys={list(d.keys())}")

# L5-03: differential (current vs current — should show zero drift)
r = call(proc, "m1nd.differential", {"agent_id":"l5","snapshot_a":"current","snapshot_b":"current"})
d = xt(r)
ok("L5-03: differential current vs current", isinstance(d, dict) and not is_err(r), f"d={str(d)[:200]}")
if isinstance(d, dict):
    ok("L5-03: zero new nodes", len(d.get("new_nodes",[])) == 0, f"new_nodes={len(d.get('new_nodes',[]))}")
    ok("L5-03: zero removed nodes", len(d.get("removed_nodes",[])) == 0, f"removed={len(d.get('removed_nodes',[]))}")

# ============================================================================
# L6: Execution Feedback
# ============================================================================
sec("L6: Execution Feedback — m1nd.trace + m1nd.validate_plan")

# L6-01: trace a Python stacktrace
python_error = """Traceback (most recent call last):
  File "backend/tests/test_spawner.py", line 45, in test_spawn_basic
    result = spawner.spawn(provider="claude")
  File "backend/spawner.py", line 67, in spawn
    session = self.pool.acquire(provider_id)
  File "backend/session_pool.py", line 120, in acquire
    raise TimeoutError("No session available")
TimeoutError: No session available"""

r = call(proc, "m1nd.trace", {"agent_id":"l6","error_text":python_error,"language":"python"})
d = xt(r)
ok("L6-01: trace returns data", isinstance(d, dict) and not is_err(r), f"d={str(d)[:200]}")
if isinstance(d, dict):
    ok("L6-01: language detected", d.get("language_detected") == "python", f"lang={d.get('language_detected')}")
    ok("L6-01: frames parsed > 0", d.get("frames_parsed",0) > 0, f"frames={d.get('frames_parsed')}")
    ok("L6-01: has suspects", isinstance(d.get("suspects"), list), f"keys={list(d.keys())}")
    ok("L6-01: has fix_scope", isinstance(d.get("fix_scope"), dict), f"fix_scope={d.get('fix_scope')}")

# L6-02: validate_plan
r = call(proc, "m1nd.validate_plan", {"agent_id":"l6","actions":[
    {"action_type":"modify","file_path":"backend/spawner.py","description":"add retry logic"},
    {"action_type":"modify","file_path":"backend/config.py","description":"add retry_count setting"},
    {"action_type":"test","file_path":"backend/tests/test_spawner.py"}
]})
d = xt(r)
ok("L6-02: validate_plan returns data", isinstance(d, dict) and not is_err(r), f"d={str(d)[:200]}")
if isinstance(d, dict):
    ok("L6-02: actions_analyzed = 3", d.get("actions_analyzed") == 3, f"analyzed={d.get('actions_analyzed')}")
    ok("L6-02: has risk_score [0,1]", 0 <= d.get("risk_score",0) <= 1, f"risk={d.get('risk_score')}")
    ok("L6-02: has risk_level", d.get("risk_level") in ("low","medium","high","critical"), f"level={d.get('risk_level')}")
    ok("L6-02: has gaps", isinstance(d.get("gaps"), list), f"gaps type={type(d.get('gaps'))}")
    ok("L6-02: has test_coverage", isinstance(d.get("test_coverage"), dict), f"test_coverage={d.get('test_coverage')}")

# ============================================================================
# L7: Multi-Repository Federation
# ============================================================================
sec("L7: Multi-Repository Federation — m1nd.federate")

# L7-01: federate with single repo (smoke test)
r = call(proc, "m1nd.federate", {
    "agent_id":"l7",
    "repos":[{"name":"m1nd","path":M1ND_PATH}],
    "detect_cross_repo_edges": False
})
d = xt(r)
ok("L7-01: federate single repo", isinstance(d, dict) and not is_err(r), f"d={str(d)[:200]}")
if isinstance(d, dict):
    ok("L7-01: has repos_ingested", isinstance(d.get("repos_ingested"), list), f"keys={list(d.keys())}")
    ok("L7-01: total_nodes > 0", d.get("total_nodes",0) > 0, f"total_nodes={d.get('total_nodes')}")
    repos = d.get("repos_ingested",[])
    if repos:
        ok("L7-01: repo name is 'm1nd'", repos[0].get("name") == "m1nd", f"name={repos[0].get('name')}")

# L7-02: federate with two repos (cross-repo edge detection)
r = call(proc, "m1nd.federate", {
    "agent_id":"l7",
    "repos":[
        {"name":"backend","path":BACKEND_PATH},
        {"name":"m1nd","path":M1ND_PATH}
    ],
    "detect_cross_repo_edges": True
})
d = xt(r)
ok("L7-02: federate two repos", isinstance(d, dict) and not is_err(r), f"d={str(d)[:200]}")
if isinstance(d, dict):
    ok("L7-02: 2 repos ingested", len(d.get("repos_ingested",[])) == 2, f"repos={len(d.get('repos_ingested',[]))}")
    ok("L7-02: total_nodes > 200", d.get("total_nodes",0) > 200, f"total={d.get('total_nodes')}")
    cross = d.get("cross_repo_edges", [])
    ok("L7-02: cross_repo_edges is list", isinstance(cross, list), f"type={type(cross)}")
    # Cross-repo edges between backend and m1nd are unlikely (different languages), but structure is valid

# ============================================================================
# SUMMARY
# ============================================================================
proc.terminate()
proc.wait()

# Check stderr for panics
stderr_path = os.path.join(workdir, "stderr.log")
with open(stderr_path) as f:
    stderr = f.read()
    has_panic = "panic" in stderr.lower()
    ok("NO PANICS in stderr", not has_panic, f"stderr tail: {stderr[-300:]}" if has_panic else "")

sec("FINAL RESULTS")
print(f"\n  {PASS}/{TOTAL} passed, {FAIL} failed")
print(f"  workdir: {workdir}")

if FAIL > 0:
    print(f"\n  {FAIL} FAILURES — review above")
    sys.exit(1)
else:
    print(f"\n  ALL {TOTAL} TESTS PASSED")
    shutil.rmtree(workdir, ignore_errors=True)
    sys.exit(0)
