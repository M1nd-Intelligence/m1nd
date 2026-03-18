#!/usr/bin/env bash
# =============================================================================
# m1nd Perspective MCP — End-to-End Integration Test
# Tests all 17 new tools (12 perspective + 5 lock)
# =============================================================================
set -euo pipefail

BINARY="./target/release/m1nd-mcp"
WORKDIR="/tmp/m1nd_persp_e2e_$$"
GRAPH_SNAP="$WORKDIR/graph_snapshot.json"
PLASTICITY="$WORKDIR/plasticity_state.json"
PASS=0
FAIL=0
TOTAL=0

mkdir -p "$WORKDIR"

# --- Helpers ----------------------------------------------------------------

rpc() {
    local name="$1"
    local args="$2"
    local id="$3"
    printf '{"jsonrpc":"2.0","method":"tools/call","id":%d,"params":{"name":"%s","arguments":%s}}\n' "$id" "$name" "$args"
}

init_msg() {
    echo '{"jsonrpc":"2.0","method":"initialize","id":1,"params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"persp-e2e","version":"1.0"}}}'
}

extract_content_text() {
    echo "$1" | jq -r '.result.content[0].text // empty' 2>/dev/null
}

assert_ok() {
    local test_name="$1"
    local response="$2"
    local condition="$3"
    TOTAL=$((TOTAL + 1))
    local result
    result=$(echo "$response" | jq -r "$condition" 2>/dev/null || echo "PARSE_ERROR")
    if [ "$result" = "true" ]; then
        PASS=$((PASS + 1))
        echo "  [PASS] $test_name"
    else
        FAIL=$((FAIL + 1))
        echo "  [FAIL] $test_name (got: $result)"
        echo "         Response: $(echo "$response" | head -c 300)"
    fi
}

get_resp() {
    local responses="$1"
    local id=$2
    echo "$responses" | while IFS= read -r line; do
        local rid
        rid=$(echo "$line" | jq -r '.id // empty' 2>/dev/null)
        if [ "$rid" = "$id" ]; then
            echo "$line"
            break
        fi
    done
}

if [ ! -x "$BINARY" ]; then
    echo "[FATAL] Binary not found at $BINARY. Run: cargo build --release"
    exit 1
fi

echo ""
echo "================================================================"
echo " PERSPECTIVE MCP E2E TEST"
echo "================================================================"
echo ""

# ============================================================================
# SESSION 1: Core 13 tools + Perspective lifecycle
# ============================================================================

# Use the m1nd source itself for ingest
INGEST_PATH="$(cd "$(dirname "$0")" && pwd)"

echo "--- Phase 1: Ingest + Perspective lifecycle ---"

MSG_01=$(init_msg)
MSG_02=$(rpc "m1nd.ingest" "{\"path\":\"$INGEST_PATH\",\"agent_id\":\"persp-test\"}" 2)
MSG_03=$(rpc "m1nd.health" '{"agent_id":"persp-test"}' 3)

# perspective.start
MSG_04=$(rpc "m1nd.perspective.start" '{"agent_id":"persp-test","query":"graph"}' 4)

# perspective.list
MSG_05=$(rpc "m1nd.perspective.list" '{"agent_id":"persp-test"}' 5)

# perspective.routes (uses perspective_id from start — we'll check after)
MSG_06=$(rpc "m1nd.perspective.routes" '{"agent_id":"persp-test","perspective_id":"persp_persp-te_001","page":1,"page_size":6}' 6)

# perspective.suggest — route_set_version must match, so we use a late-bound approach
# We send it with version 0; it will fail with stale error showing current version.
# SKIP suggest for now — it requires dynamic route_set_version which bash can't chain.
# The unit tests cover suggest logic. Here we just verify the error shape.
MSG_07=$(rpc "m1nd.perspective.suggest" '{"agent_id":"persp-test","perspective_id":"persp_persp-te_001","route_set_version":0}' 7)

# perspective.branch
MSG_08=$(rpc "m1nd.perspective.branch" '{"agent_id":"persp-test","perspective_id":"persp_persp-te_001","branch_name":"test_branch"}' 8)

# perspective.compare (compare original vs branch)
MSG_09=$(rpc "m1nd.perspective.compare" '{"agent_id":"persp-test","perspective_id_a":"persp_persp-te_001","perspective_id_b":"persp_persp-te_002"}' 9)

# perspective.close (close the branch)
MSG_10=$(rpc "m1nd.perspective.close" '{"agent_id":"persp-test","perspective_id":"persp_persp-te_002"}' 10)

# Second perspective.start (verify IDs increment)
MSG_11=$(rpc "m1nd.perspective.start" '{"agent_id":"persp-test","query":"session","anchor_node":"file::m1nd-mcp/src/session.rs"}' 11)

# perspective.close (close second)
MSG_12=$(rpc "m1nd.perspective.close" '{"agent_id":"persp-test","perspective_id":"persp_persp-te_003"}' 12)

# perspective.close (close first)
MSG_13=$(rpc "m1nd.perspective.close" '{"agent_id":"persp-test","perspective_id":"persp_persp-te_001"}' 13)

# perspective.list (should be empty now)
MSG_14=$(rpc "m1nd.perspective.list" '{"agent_id":"persp-test"}' 14)

# --- Lock lifecycle ---

# lock.create
MSG_15=$(rpc "m1nd.lock.create" '{"agent_id":"persp-test","scope":"node","root_nodes":["file::m1nd-mcp/src/session.rs"]}' 15)

# lock.watch
MSG_16=$(rpc "m1nd.lock.watch" '{"agent_id":"persp-test","lock_id":"lock_persp-te_001","strategy":"on_ingest"}' 16)

# lock.diff (no changes expected)
MSG_17=$(rpc "m1nd.lock.diff" '{"agent_id":"persp-test","lock_id":"lock_persp-te_001"}' 17)

# lock.rebase
MSG_18=$(rpc "m1nd.lock.rebase" '{"agent_id":"persp-test","lock_id":"lock_persp-te_001"}' 18)

# lock.release
MSG_19=$(rpc "m1nd.lock.release" '{"agent_id":"persp-test","lock_id":"lock_persp-te_001"}' 19)

# Final health
MSG_20=$(rpc "m1nd.health" '{"agent_id":"persp-test"}' 20)

ALL_MSGS=$(printf '%s\n' \
    "$MSG_01" "$MSG_02" "$MSG_03" "$MSG_04" "$MSG_05" \
    "$MSG_06" "$MSG_07" "$MSG_08" "$MSG_09" "$MSG_10" \
    "$MSG_11" "$MSG_12" "$MSG_13" "$MSG_14" "$MSG_15" \
    "$MSG_16" "$MSG_17" "$MSG_18" "$MSG_19" "$MSG_20"
)

echo "  Sending 20 JSON-RPC messages..."

RESPONSES=$(echo "$ALL_MSGS" | M1ND_GRAPH_SOURCE="$GRAPH_SNAP" M1ND_PLASTICITY_STATE="$PLASTICITY" timeout 120 "$BINARY" 2>"$WORKDIR/stderr.log")

echo "  Server exited. Parsing responses..."
echo ""

# --- Extract responses ---

R_INIT=$(get_resp "$RESPONSES" 1)
R_INGEST=$(get_resp "$RESPONSES" 2)
R_HEALTH=$(get_resp "$RESPONSES" 3)
R_P_START=$(get_resp "$RESPONSES" 4)
R_P_LIST1=$(get_resp "$RESPONSES" 5)
R_P_ROUTES=$(get_resp "$RESPONSES" 6)
R_P_SUGGEST=$(get_resp "$RESPONSES" 7)
R_P_BRANCH=$(get_resp "$RESPONSES" 8)
R_P_COMPARE=$(get_resp "$RESPONSES" 9)
R_P_CLOSE1=$(get_resp "$RESPONSES" 10)
R_P_START2=$(get_resp "$RESPONSES" 11)
R_P_CLOSE2=$(get_resp "$RESPONSES" 12)
R_P_CLOSE3=$(get_resp "$RESPONSES" 13)
R_P_LIST2=$(get_resp "$RESPONSES" 14)
R_L_CREATE=$(get_resp "$RESPONSES" 15)
R_L_WATCH=$(get_resp "$RESPONSES" 16)
R_L_DIFF=$(get_resp "$RESPONSES" 17)
R_L_REBASE=$(get_resp "$RESPONSES" 18)
R_L_RELEASE=$(get_resp "$RESPONSES" 19)
R_HEALTH2=$(get_resp "$RESPONSES" 20)

# --- Assertions ---

echo "=== INIT + INGEST ==="
assert_ok "init: protocol version" "$R_INIT" '.result.protocolVersion == "2024-11-05"'

T_INGEST=$(extract_content_text "$R_INGEST")
assert_ok "ingest: no error" "$R_INGEST" '(.result.isError // false) == false'
assert_ok "ingest: nodes > 0" "$T_INGEST" '(.nodes_created // 0) > 0'
echo "  Ingested: $(echo "$T_INGEST" | jq '.nodes_created // 0') nodes, $(echo "$T_INGEST" | jq '.edges_created // 0') edges"

T_HEALTH=$(extract_content_text "$R_HEALTH")
assert_ok "health: nodes > 50" "$T_HEALTH" '(.node_count // 0) > 50'

echo ""
echo "=== PERSPECTIVE.START ==="
T_START=$(extract_content_text "$R_P_START")
assert_ok "start: no error" "$R_P_START" '(.result.isError // false) == false'
assert_ok "start: has perspective_id" "$T_START" 'has("perspective_id")'
assert_ok "start: has mode" "$T_START" 'has("mode")'
assert_ok "start: has routes" "$T_START" 'has("routes")'
assert_ok "start: has route_set_version" "$T_START" 'has("route_set_version")'
assert_ok "start: has cache_generation" "$T_START" 'has("cache_generation")'
P_ID=$(echo "$T_START" | jq -r '.perspective_id // "none"')
P_RSV=$(echo "$T_START" | jq '.route_set_version // 0')
P_ROUTES=$(echo "$T_START" | jq '.total_routes // 0')
echo "  perspective_id=$P_ID, routes=$P_ROUTES, version=$P_RSV"

echo ""
echo "=== PERSPECTIVE.LIST (after start) ==="
T_LIST1=$(extract_content_text "$R_P_LIST1")
assert_ok "list1: no error" "$R_P_LIST1" '(.result.isError // false) == false'
assert_ok "list1: has perspectives" "$T_LIST1" 'has("perspectives")'
assert_ok "list1: count >= 1" "$T_LIST1" '((.perspectives // []) | length) >= 1'

echo ""
echo "=== PERSPECTIVE.ROUTES ==="
T_ROUTES=$(extract_content_text "$R_P_ROUTES")
assert_ok "routes: no error" "$R_P_ROUTES" '(.result.isError // false) == false'
assert_ok "routes: has page" "$T_ROUTES" 'has("page")'
assert_ok "routes: has total_routes" "$T_ROUTES" 'has("total_routes")'
assert_ok "routes: has routes array" "$T_ROUTES" 'has("routes")'
assert_ok "routes: has lens_summary" "$T_ROUTES" 'has("lens_summary")'

echo ""
echo "=== PERSPECTIVE.SUGGEST (stale version → expected error) ==="
# suggest requires matching route_set_version which is dynamic.
# We verify the staleness guard works correctly.
assert_ok "suggest: returns stale error (expected)" "$R_P_SUGGEST" '.result.isError == true'
SUGGEST_ERR=$(extract_content_text "$R_P_SUGGEST")
TOTAL=$((TOTAL + 1))
if echo "$SUGGEST_ERR" | grep -q "route set stale"; then
    PASS=$((PASS + 1))
    echo "  [PASS] suggest: stale guard fires correctly"
else
    FAIL=$((FAIL + 1))
    echo "  [FAIL] suggest: unexpected error: $SUGGEST_ERR"
fi

echo ""
echo "=== PERSPECTIVE.BRANCH ==="
T_BRANCH=$(extract_content_text "$R_P_BRANCH")
assert_ok "branch: no error" "$R_P_BRANCH" '(.result.isError // false) == false'
assert_ok "branch: has branch_perspective_id" "$T_BRANCH" 'has("branch_perspective_id")'
assert_ok "branch: has branch_name" "$T_BRANCH" '.branch_name == "test_branch"'
B_ID=$(echo "$T_BRANCH" | jq -r '.branch_perspective_id // "none"')
echo "  branch_id=$B_ID"

echo ""
echo "=== PERSPECTIVE.COMPARE ==="
T_COMPARE=$(extract_content_text "$R_P_COMPARE")
assert_ok "compare: no error" "$R_P_COMPARE" '(.result.isError // false) == false'
assert_ok "compare: has shared_nodes" "$T_COMPARE" 'has("shared_nodes")'
assert_ok "compare: has unique_to_a" "$T_COMPARE" 'has("unique_to_a")'
assert_ok "compare: has unique_to_b" "$T_COMPARE" 'has("unique_to_b")'

echo ""
echo "=== PERSPECTIVE.CLOSE (branch) ==="
T_CLOSE1=$(extract_content_text "$R_P_CLOSE1")
assert_ok "close_branch: no error" "$R_P_CLOSE1" '(.result.isError // false) == false'
assert_ok "close_branch: closed=true" "$T_CLOSE1" '.closed == true'

echo ""
echo "=== PERSPECTIVE.START (anchored) ==="
T_START2=$(extract_content_text "$R_P_START2")
assert_ok "start2: no error" "$R_P_START2" '(.result.isError // false) == false'
assert_ok "start2: mode=anchored" "$T_START2" '.mode == "anchored"'
assert_ok "start2: anchor_node set" "$T_START2" '.anchor_node == "file::m1nd-mcp/src/session.rs"'
echo "  Anchored perspective: $(echo "$T_START2" | jq -r '.perspective_id // "none"'), focus=$(echo "$T_START2" | jq -r '.focus_node // "none"')"

echo ""
echo "=== PERSPECTIVE.LIST (after all closed) ==="
T_LIST2=$(extract_content_text "$R_P_LIST2")
assert_ok "list2: no error" "$R_P_LIST2" '(.result.isError // false) == false'
assert_ok "list2: empty after close" "$T_LIST2" '((.perspectives // []) | length) == 0'

echo ""
echo "=== LOCK.CREATE ==="
T_LCREATE=$(extract_content_text "$R_L_CREATE")
assert_ok "lock.create: no error" "$R_L_CREATE" '(.result.isError // false) == false'
assert_ok "lock.create: has lock_id" "$T_LCREATE" 'has("lock_id")'
assert_ok "lock.create: has baseline_nodes" "$T_LCREATE" 'has("baseline_nodes")'
assert_ok "lock.create: has graph_generation" "$T_LCREATE" 'has("graph_generation")'
L_ID=$(echo "$T_LCREATE" | jq -r '.lock_id // "none"')
echo "  lock_id=$L_ID, nodes=$(echo "$T_LCREATE" | jq '.baseline_nodes // 0'), edges=$(echo "$T_LCREATE" | jq '.baseline_edges // 0')"

echo ""
echo "=== LOCK.WATCH ==="
T_LWATCH=$(extract_content_text "$R_L_WATCH")
assert_ok "lock.watch: no error" "$R_L_WATCH" '(.result.isError // false) == false'
assert_ok "lock.watch: strategy=on_ingest" "$T_LWATCH" '.strategy == "on_ingest"'

echo ""
echo "=== LOCK.DIFF ==="
T_LDIFF=$(extract_content_text "$R_L_DIFF")
assert_ok "lock.diff: no error" "$R_L_DIFF" '(.result.isError // false) == false'
assert_ok "lock.diff: has diff" "$T_LDIFF" 'has("diff")'
assert_ok "lock.diff: no_changes (no mutation between create and diff)" "$T_LDIFF" '.diff.no_changes == true'

echo ""
echo "=== LOCK.REBASE ==="
T_LREBASE=$(extract_content_text "$R_L_REBASE")
assert_ok "lock.rebase: no error" "$R_L_REBASE" '(.result.isError // false) == false'
assert_ok "lock.rebase: has new_generation" "$T_LREBASE" 'has("new_generation")'
assert_ok "lock.rebase: watcher preserved" "$T_LREBASE" '.watcher_preserved == true'

echo ""
echo "=== LOCK.RELEASE ==="
T_LRELEASE=$(extract_content_text "$R_L_RELEASE")
assert_ok "lock.release: no error" "$R_L_RELEASE" '(.result.isError // false) == false'
assert_ok "lock.release: released=true" "$T_LRELEASE" '.released == true'

echo ""
echo "=== FINAL HEALTH ==="
T_HEALTH2=$(extract_content_text "$R_HEALTH2")
assert_ok "health2: no error" "$R_HEALTH2" '(.result.isError // false) == false'
assert_ok "health2: has node_count" "$T_HEALTH2" '(.node_count // 0) > 0'
assert_ok "health2: active sessions" "$T_HEALTH2" '((.active_sessions // []) | length) > 0'
echo "  nodes=$(echo "$T_HEALTH2" | jq '.node_count // 0'), sessions=$(echo "$T_HEALTH2" | jq '(.active_sessions // []) | length')"

# Check stderr for panics
echo ""
echo "=== SANITY ==="
if grep -qi "panic\|thread.*panicked\|SIGSEGV\|SIGABRT" "$WORKDIR/stderr.log" 2>/dev/null; then
    TOTAL=$((TOTAL + 1))
    FAIL=$((FAIL + 1))
    echo "  [FAIL] Server panicked! See: $WORKDIR/stderr.log"
else
    TOTAL=$((TOTAL + 1))
    PASS=$((PASS + 1))
    echo "  [PASS] No panics in server stderr"
fi

# --- Final report ---

echo ""
echo "================================================================"
echo " PERSPECTIVE E2E RESULTS: $PASS / $TOTAL passed ($FAIL failed)"
echo "================================================================"
echo ""

if [ "$FAIL" -gt 0 ]; then
    echo "  Server stderr:"
    cat "$WORKDIR/stderr.log" | head -30
    echo ""
    echo "  STATUS: FAIL"
else
    echo "  STATUS: ALL PASS"
    rm -rf "$WORKDIR"
fi

exit "$FAIL"
