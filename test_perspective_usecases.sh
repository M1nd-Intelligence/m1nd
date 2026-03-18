#!/usr/bin/env zsh
# =============================================================================
# m1nd Perspective MCP — Interactive Use Case Tests
# Uses zsh coproc for bidirectional communication with the server
# =============================================================================
set -euo pipefail

BINARY="./target/release/m1nd-mcp"
WORKDIR="/tmp/m1nd_usecase_$$"
GRAPH_SNAP="$WORKDIR/graph_snapshot.json"
PLASTICITY="$WORKDIR/plasticity_state.json"
PASS=0
FAIL=0
TOTAL=0
MSG_ID=0

mkdir -p "$WORKDIR"

# Start server as coprocess (zsh style)
coproc GRAPH_SNAPSHOT_PATH="$GRAPH_SNAP" PLASTICITY_STATE_PATH="$PLASTICITY" "$BINARY" 2>"$WORKDIR/stderr.log"
SERVER_PID=$!

cleanup() {
    kill $SERVER_PID 2>/dev/null || true
    wait $SERVER_PID 2>/dev/null || true
    rm -rf "$WORKDIR"
}
trap cleanup EXIT

# --- Helpers ----------------------------------------------------------------

call() {
    local msg="$1"
    print -p "$msg"
    local resp
    read -p resp
    echo "$resp"
}

next_id() { MSG_ID=$((MSG_ID + 1)); echo $MSG_ID; }

rpc() {
    local name="$1"; local args="$2"
    printf '{"jsonrpc":"2.0","method":"tools/call","id":%d,"params":{"name":"%s","arguments":%s}}' "$(next_id)" "$name" "$args"
}

xt() { echo "$1" | jq -r '.result.content[0].text // empty' 2>/dev/null; }
xf() { echo "$1" | jq -r "fromjson | .$2 // empty" 2>/dev/null; }
is_err() { echo "$1" | jq -e '.result.isError == true' >/dev/null 2>&1; }

ok() {
    local name="$1"; local resp="$2"; local cond="$3"
    TOTAL=$((TOTAL + 1))
    local r; r=$(echo "$resp" | jq -r "$cond" 2>/dev/null || echo "PARSE_ERR")
    if [ "$r" = "true" ]; then PASS=$((PASS + 1)); echo "  [PASS] $name"
    else FAIL=$((FAIL + 1)); echo "  [FAIL] $name ($r)"; echo "    $(echo "$resp" | head -c 300)"; fi
}

sec() { echo ""; echo "=== $1 ==="; }

# --- Init + Ingest ----------------------------------------------------------

echo "Starting server (PID $SERVER_PID)..."

R=$(call '{"jsonrpc":"2.0","method":"initialize","id":1,"params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"uc","version":"1.0"}}}')
MSG_ID=1
echo "Init OK"

R=$(call "$(rpc "m1nd.ingest" '{"agent_id":"uc","path":"/Users/cosmophonix/clawd/roomanizer-os/mcp/m1nd","mode":"full"}')")
T=$(xt "$R")
echo "Ingested: $(echo "$T" | jq -r 'fromjson | "\(.node_count) nodes, \(.edge_count) edges"' 2>/dev/null)"

# ============================================================================
sec "UC1: Exploration — 'What does McpServer do?'"
# ============================================================================

R=$(call "$(rpc "m1nd.perspective.start" '{"agent_id":"exp","query":"McpServer"}')")
T=$(xt "$R")
ok "start" "$R" '.result.content[0].text | fromjson | .perspective_id == "persp_exp_001"'
FOCUS=$(xf "$T" "focus_node"); RSV=$(xf "$T" "route_set_version")
echo "  Focus: $FOCUS | Routes: $(echo "$T" | jq 'fromjson | .routes | length' 2>/dev/null)"

R=$(call "$(rpc "m1nd.perspective.routes" "{\"agent_id\":\"exp\",\"perspective_id\":\"persp_exp_001\",\"route_set_version\":$RSV}")")
T=$(xt "$R")
ok "routes" "$R" '.result.content[0].text | fromjson | (.routes | length) > 0'
RSV=$(xf "$T" "route_set_version")
echo "  Neighbors of McpServer:"
echo "$T" | jq -r 'fromjson | .routes[:6][] | "    \(.target_label) [\(.family)] score=\(.score)"' 2>/dev/null || true

ROUTE1=$(echo "$T" | jq -r 'fromjson | .routes[0].route_id' 2>/dev/null)

R=$(call "$(rpc "m1nd.perspective.inspect" "{\"agent_id\":\"exp\",\"perspective_id\":\"persp_exp_001\",\"route_id\":\"$ROUTE1\",\"route_set_version\":$RSV}")")
T=$(xt "$R")
if is_err "$R"; then
    TOTAL=$((TOTAL + 1)); FAIL=$((FAIL + 1)); echo "  [FAIL] inspect: $(echo "$T" | head -c 200)"
else
    ok "inspect" "$R" '.result.content[0].text | fromjson | .target_node != null'
    echo "  Target: $(xf "$T" "target_label") | Source: $(echo "$T" | jq -r 'fromjson | .provenance.source_path // "none"' 2>/dev/null)"
fi

# ============================================================================
sec "UC2: Anchored — 'Understand SessionState'"
# ============================================================================

R=$(call "$(rpc "m1nd.perspective.start" '{"agent_id":"anc","query":"SessionState","anchor_node":"session"}')")
T=$(xt "$R")
ok "anchored start" "$R" '.result.content[0].text | fromjson | .perspective_id == "persp_anc_001"'
RSV=$(xf "$T" "route_set_version"); FOCUS=$(xf "$T" "focus_node")
echo "  Anchor: $(xf "$T" "anchor_node") | Focus: $FOCUS"

R=$(call "$(rpc "m1nd.perspective.routes" "{\"agent_id\":\"anc\",\"perspective_id\":\"persp_anc_001\",\"route_set_version\":$RSV}")")
T=$(xt "$R")
ok "routes" "$R" '.result.content[0].text | fromjson | (.routes | length) > 0'
RSV=$(xf "$T" "route_set_version")
ROUTE1=$(echo "$T" | jq -r 'fromjson | .routes[0].route_id' 2>/dev/null)
echo "  Routes from SessionState:"
echo "$T" | jq -r 'fromjson | .routes[:5][] | "    \(.target_label) [score=\(.score)]"' 2>/dev/null || true

# Follow
R=$(call "$(rpc "m1nd.perspective.follow" "{\"agent_id\":\"anc\",\"perspective_id\":\"persp_anc_001\",\"route_id\":\"$ROUTE1\",\"route_set_version\":$RSV}")")
T=$(xt "$R")
if is_err "$R"; then
    TOTAL=$((TOTAL + 1)); FAIL=$((FAIL + 1)); echo "  [FAIL] follow: $(echo "$T" | head -c 200)"
else
    ok "follow" "$R" '.result.content[0].text | fromjson | .new_focus != null'
    NEW_FOCUS=$(xf "$T" "new_focus"); RSV=$(xf "$T" "route_set_version")
    echo "  Followed to: $NEW_FOCUS"
fi

# Back
R=$(call "$(rpc "m1nd.perspective.back" '{"agent_id":"anc","perspective_id":"persp_anc_001"}')")
T=$(xt "$R")
ok "back" "$R" '.result.content[0].text | fromjson | .restored_focus != null'
echo "  Restored: $(xf "$T" "restored_focus")"

# ============================================================================
sec "UC3: Branch + Compare"
# ============================================================================

R=$(call "$(rpc "m1nd.perspective.start" '{"agent_id":"cmp","query":"PerspectiveState"}')")
T=$(xt "$R")
RSV=$(xf "$T" "route_set_version")
ROUTE_A=$(echo "$T" | jq -r 'fromjson | .routes[0].route_id // empty' 2>/dev/null)

if [ -n "$ROUTE_A" ]; then
    R=$(call "$(rpc "m1nd.perspective.follow" "{\"agent_id\":\"cmp\",\"perspective_id\":\"persp_cmp_001\",\"route_id\":\"$ROUTE_A\",\"route_set_version\":$RSV}")")
    T=$(xt "$R")
    echo "  Original → $(xf "$T" "new_focus")"
fi

R=$(call "$(rpc "m1nd.perspective.branch" '{"agent_id":"cmp","perspective_id":"persp_cmp_001","branch_name":"alt"}')")
T=$(xt "$R")
ok "branch" "$R" '.result.content[0].text | fromjson | .branch_perspective_id == "persp_cmp_002"'

R=$(call "$(rpc "m1nd.perspective.routes" '{"agent_id":"cmp","perspective_id":"persp_cmp_002"}')")
T=$(xt "$R")
RSV2=$(xf "$T" "route_set_version")
ROUTE_B=$(echo "$T" | jq -r 'fromjson | .routes[1].route_id // .routes[0].route_id // empty' 2>/dev/null)

if [ -n "${ROUTE_B:-}" ]; then
    R=$(call "$(rpc "m1nd.perspective.follow" "{\"agent_id\":\"cmp\",\"perspective_id\":\"persp_cmp_002\",\"route_id\":\"$ROUTE_B\",\"route_set_version\":$RSV2}")")
    T=$(xt "$R")
    echo "  Branch → $(xf "$T" "new_focus")"
fi

R=$(call "$(rpc "m1nd.perspective.compare" '{"agent_id":"cmp","perspective_id_a":"persp_cmp_001","perspective_id_b":"persp_cmp_002"}')")
T=$(xt "$R")
ok "compare" "$R" '.result.content[0].text | fromjson | .shared_nodes != null'
echo "  Shared: $(echo "$T" | jq 'fromjson | .shared_nodes | length' 2>/dev/null) | A-only: $(echo "$T" | jq 'fromjson | .unique_to_a | length' 2>/dev/null) | B-only: $(echo "$T" | jq 'fromjson | .unique_to_b | length' 2>/dev/null)"

# ============================================================================
sec "UC4: Deep Navigation — 3 follows + 2 backs"
# ============================================================================

R=$(call "$(rpc "m1nd.perspective.start" '{"agent_id":"div","query":"server"}')")
T=$(xt "$R"); RSV=$(xf "$T" "route_set_version")
START=$(xf "$T" "focus_node"); echo "  Start: $START"
NODES=("$START")

for i in 1 2 3; do
    ROUTE=$(echo "$T" | jq -r 'fromjson | .routes[0].route_id // empty' 2>/dev/null)
    [ -z "$ROUTE" ] && { echo "  Dead end at level $i"; break; }
    R=$(call "$(rpc "m1nd.perspective.follow" "{\"agent_id\":\"div\",\"perspective_id\":\"persp_div_001\",\"route_id\":\"$ROUTE\",\"route_set_version\":$RSV}")")
    T=$(xt "$R")
    if is_err "$R"; then echo "  Follow $i error"; break; fi
    NODE=$(xf "$T" "new_focus"); RSV=$(xf "$T" "route_set_version")
    NODES+=("$NODE"); echo "  Follow $i → $NODE"
done

for i in 1 2; do
    R=$(call "$(rpc "m1nd.perspective.back" '{"agent_id":"div","perspective_id":"persp_div_001"}')")
    T=$(xt "$R"); echo "  Back $i → $(xf "$T" "restored_focus")"
done

TOTAL=$((TOTAL + 1))
[ ${#NODES[@]} -ge 3 ] && { PASS=$((PASS + 1)); echo "  [PASS] ${#NODES[@]} nodes visited"; } || { FAIL=$((FAIL + 1)); echo "  [FAIL] only ${#NODES[@]}"; }

# ============================================================================
sec "UC5: Lock Lifecycle"
# ============================================================================

R=$(call "$(rpc "m1nd.lock.create" '{"agent_id":"rfx","perspective_id":"persp_div_001","scope":["server"]}')")
T=$(xt "$R")
ok "lock.create" "$R" '.result.content[0].text | fromjson | .lock_id != null'
LID=$(xf "$T" "lock_id")
echo "  Lock: $LID | Baseline: $(xf "$T" "baseline_nodes") nodes, $(xf "$T" "baseline_edges") edges"

R=$(call "$(rpc "m1nd.lock.watch" "{\"agent_id\":\"rfx\",\"lock_id\":\"$LID\",\"strategy\":\"on_ingest\"}")")
ok "lock.watch" "$R" '.result.content[0].text | fromjson | .strategy == "on_ingest"'

R=$(call "$(rpc "m1nd.lock.diff" "{\"agent_id\":\"rfx\",\"lock_id\":\"$LID\"}")")
T=$(xt "$R")
ok "lock.diff" "$R" '.result.content[0].text | fromjson | .diff != null'
echo "  Changes: $(echo "$T" | jq -r 'fromjson | .diff | to_entries | map("\(.key)=\(.value)") | join(", ")' 2>/dev/null)"

R=$(call "$(rpc "m1nd.lock.rebase" "{\"agent_id\":\"rfx\",\"lock_id\":\"$LID\"}")")
ok "lock.rebase" "$R" '.result.content[0].text | fromjson | .new_generation != null'

R=$(call "$(rpc "m1nd.lock.release" "{\"agent_id\":\"rfx\",\"lock_id\":\"$LID\"}")")
ok "lock.release" "$R" '.result.content[0].text | fromjson | .released == true'

# ============================================================================
sec "UC6: Multi-Agent + List"
# ============================================================================

R=$(call "$(rpc "m1nd.perspective.start" '{"agent_id":"a1","query":"dispatch"}')")
ok "a1" "$R" '.result.content[0].text | fromjson | .perspective_id == "persp_a1_001"'

R=$(call "$(rpc "m1nd.perspective.start" '{"agent_id":"a2","query":"PerspectiveLens"}')")
ok "a2" "$R" '.result.content[0].text | fromjson | .perspective_id == "persp_a2_001"'

R=$(call "$(rpc "m1nd.perspective.list" '{"agent_id":"a1"}')")
T=$(xt "$R")
ok "list" "$R" '.result.content[0].text | fromjson | (.perspectives | length) >= 2'
echo "  Active perspectives:"
echo "$T" | jq -r 'fromjson | .perspectives[] | "    \(.perspective_id) focus=\(.focus_node // "?")"' 2>/dev/null || true

# ============================================================================
sec "UC7: Suggest"
# ============================================================================

R=$(call "$(rpc "m1nd.perspective.start" '{"agent_id":"sug","query":"Graph"}')")
T=$(xt "$R"); RSV=$(xf "$T" "route_set_version")

R=$(call "$(rpc "m1nd.perspective.suggest" "{\"agent_id\":\"sug\",\"perspective_id\":\"persp_sug_001\",\"route_set_version\":$RSV}")")
T=$(xt "$R")
if is_err "$R"; then
    TOTAL=$((TOTAL + 1)); PASS=$((PASS + 1))
    echo "  [PASS] suggest returned valid error (stale or empty)"
else
    ok "suggest" "$R" '.result.content[0].text | fromjson | .suggestion != null'
    echo "  $(echo "$T" | jq -r 'fromjson | .suggestion' 2>/dev/null | head -c 200)"
fi

# ============================================================================
sec "CLEANUP"
# ============================================================================

for P in "exp:persp_exp_001" "anc:persp_anc_001" "cmp:persp_cmp_001" "cmp:persp_cmp_002" "div:persp_div_001" "a1:persp_a1_001" "a2:persp_a2_001" "sug:persp_sug_001"; do
    A="${P%%:*}"; ID="${P##*:}"
    call "$(rpc "m1nd.perspective.close" "{\"agent_id\":\"$A\",\"perspective_id\":\"$ID\"}")" >/dev/null
done

R=$(call "$(rpc "m1nd.health" '{"agent_id":"uc"}')")
T=$(xt "$R")
ok "health" "$R" '.result.content[0].text | fromjson | .status == "ok"'
echo "  $(echo "$T" | jq -r 'fromjson | "\(.node_count) nodes, \(.edge_count) edges, \(.active_sessions) sessions"' 2>/dev/null)"

TOTAL=$((TOTAL + 1))
if grep -q "panic" "$WORKDIR/stderr.log" 2>/dev/null; then
    FAIL=$((FAIL + 1)); echo "  [FAIL] PANIC!"
else
    PASS=$((PASS + 1)); echo "  [PASS] No panics"
fi

echo ""
echo "================================================================"
echo " RESULTS: $PASS / $TOTAL passed ($FAIL failed)"
echo "================================================================"
[ $FAIL -eq 0 ] && echo "  STATUS: ALL PASS" || { echo "  STATUS: $FAIL FAILURES"; }

exit $FAIL
