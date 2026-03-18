#!/usr/bin/env bash
# =============================================================================
# m1nd End-to-End Integration Stress + Calibration Test
# Agent D0 — final quality gate
# =============================================================================
set -euo pipefail

BINARY="./target/release/m1nd-mcp"
WORKDIR="/tmp/m1nd_e2e_test_$$"
GRAPH_SNAP="$WORKDIR/graph_snapshot.json"
PLASTICITY="$WORKDIR/plasticity_state.json"
PASS=0
FAIL=0
TOTAL=0
ISSUES=""

mkdir -p "$WORKDIR"

# --- Helpers ----------------------------------------------------------------

rpc() {
    # Build a JSON-RPC tools/call message
    local name="$1"
    local args="$2"
    local id="$3"
    printf '{"jsonrpc":"2.0","method":"tools/call","id":%d,"params":{"name":"%s","arguments":%s}}\n' "$id" "$name" "$args"
}

init_msg() {
    echo '{"jsonrpc":"2.0","method":"initialize","id":1,"params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"e2e-test","version":"1.0"}}}'
}

assert_ok() {
    local test_name="$1"
    local response="$2"
    local condition="$3"  # jq expression that should return "true"
    TOTAL=$((TOTAL + 1))
    local result
    result=$(echo "$response" | jq -r "$condition" 2>/dev/null || echo "PARSE_ERROR")
    if [ "$result" = "true" ]; then
        PASS=$((PASS + 1))
        echo "  [PASS] $test_name"
    else
        FAIL=$((FAIL + 1))
        echo "  [FAIL] $test_name (got: $result)"
        ISSUES="$ISSUES\n  - $test_name"
        # Print response snippet for debugging
        echo "         Response: $(echo "$response" | head -c 500)"
    fi
}

extract_content_text() {
    # MCP wraps tool results in content[].text — extract and parse the inner JSON
    echo "$1" | jq -r '.result.content[0].text // empty' 2>/dev/null
}

# --- Phase 1: Build check ---------------------------------------------------

echo ""
echo "================================================================"
echo " PHASE 1: Build and binary check"
echo "================================================================"

if [ ! -x "$BINARY" ]; then
    echo "[FATAL] Binary not found at $BINARY. Run: cargo build --release"
    exit 1
fi
echo "  Binary: $BINARY (OK)"

# --- Phase 2: Self-ingest test -----------------------------------------------

echo ""
echo "================================================================"
echo " PHASE 2: Self-ingest test (m1nd ingests itself)"
echo "================================================================"

# Build the full sequence of JSON-RPC messages for session 1.
# We send them all as stdin lines; the server processes them sequentially.
# Collect ALL responses (one per line).

SESSION1_INPUT=$(cat <<'JSONRPC_EOF'
INIT_PLACEHOLDER
INGEST_PLACEHOLDER
HEALTH1_PLACEHOLDER
ACTIVATE_PLACEHOLDER
IMPACT_PLACEHOLDER
MISSING_PLACEHOLDER
WHY_PLACEHOLDER
WARMUP_PLACEHOLDER
COUNTERFACTUAL_PLACEHOLDER
PREDICT_PLACEHOLDER
FINGERPRINT_PLACEHOLDER
DRIFT_PLACEHOLDER
LEARN_PLACEHOLDER
RESONATE_PLACEHOLDER
HEALTH2_PLACEHOLDER
JSONRPC_EOF
)

# Build actual messages
MSG_INIT=$(init_msg)

MSG_INGEST=$(rpc "m1nd.ingest" '{"path":"/Users/cosmophonix/connectome-poc/m1nd","agent_id":"e2e-test"}' 2)

MSG_HEALTH1=$(rpc "m1nd.health" '{"agent_id":"e2e-test"}' 3)

MSG_ACTIVATE=$(rpc "m1nd.activate" '{"query":"spreading activation","agent_id":"e2e-test","top_k":20}' 4)

# External IDs in the graph use "file::" prefix (set by ingestor)
MSG_IMPACT=$(rpc "m1nd.impact" '{"node_id":"file::m1nd-core/src/graph.rs","agent_id":"e2e-test","direction":"forward"}' 5)

MSG_MISSING=$(rpc "m1nd.missing" '{"query":"graph query","agent_id":"e2e-test"}' 6)

MSG_WHY=$(rpc "m1nd.why" '{"source":"file::m1nd-core/src/graph.rs","target":"file::m1nd-core/src/activation.rs","agent_id":"e2e-test"}' 7)

MSG_WARMUP=$(rpc "m1nd.warmup" '{"task_description":"implement temporal scoring","agent_id":"e2e-test"}' 8)

MSG_COUNTERFACTUAL=$(rpc "m1nd.counterfactual" '{"node_ids":["file::m1nd-core/src/graph.rs","file::m1nd-core/src/types.rs"],"agent_id":"e2e-test"}' 9)

MSG_PREDICT=$(rpc "m1nd.predict" '{"changed_node":"file::m1nd-core/src/graph.rs","agent_id":"e2e-test"}' 10)

MSG_FINGERPRINT=$(rpc "m1nd.fingerprint" '{"target_node":"file::m1nd-core/src/graph.rs","agent_id":"e2e-test"}' 11)

MSG_DRIFT=$(rpc "m1nd.drift" '{"agent_id":"e2e-test"}' 12)

MSG_LEARN=$(rpc "m1nd.learn" '{"query":"test","agent_id":"e2e-test","feedback":"correct","node_ids":["file::m1nd-core/src/graph.rs","file::m1nd-core/src/types.rs"]}' 13)

MSG_RESONATE=$(rpc "m1nd.resonate" '{"query":"graph activation","agent_id":"e2e-test"}' 14)

MSG_HEALTH2=$(rpc "m1nd.health" '{"agent_id":"e2e-test"}' 15)

# Combine all messages
ALL_MSGS=$(printf '%s\n' \
    "$MSG_INIT" \
    "$MSG_INGEST" \
    "$MSG_HEALTH1" \
    "$MSG_ACTIVATE" \
    "$MSG_IMPACT" \
    "$MSG_MISSING" \
    "$MSG_WHY" \
    "$MSG_WARMUP" \
    "$MSG_COUNTERFACTUAL" \
    "$MSG_PREDICT" \
    "$MSG_FINGERPRINT" \
    "$MSG_DRIFT" \
    "$MSG_LEARN" \
    "$MSG_RESONATE" \
    "$MSG_HEALTH2"
)

echo "  Sending 15 JSON-RPC messages to server..."

# Run the server with env vars pointing to our workdir
RESPONSES=$(echo "$ALL_MSGS" | M1ND_GRAPH_SOURCE="$GRAPH_SNAP" M1ND_PLASTICITY_STATE="$PLASTICITY" timeout 120 "$BINARY" 2>"$WORKDIR/stderr.log")

# Parse responses (one per line, corresponding to ids 1-15)
echo "  Server exited. Parsing responses..."

# Extract each response by id
get_resp() {
    local id=$1
    echo "$RESPONSES" | while IFS= read -r line; do
        local rid
        rid=$(echo "$line" | jq -r '.id // empty' 2>/dev/null)
        if [ "$rid" = "$id" ]; then
            echo "$line"
            break
        fi
    done
}

R_INIT=$(get_resp 1)
R_INGEST=$(get_resp 2)
R_HEALTH1=$(get_resp 3)
R_ACTIVATE=$(get_resp 4)
R_IMPACT=$(get_resp 5)
R_MISSING=$(get_resp 6)
R_WHY=$(get_resp 7)
R_WARMUP=$(get_resp 8)
R_COUNTERFACTUAL=$(get_resp 9)
R_PREDICT=$(get_resp 10)
R_FINGERPRINT=$(get_resp 11)
R_DRIFT=$(get_resp 12)
R_LEARN=$(get_resp 13)
R_RESONATE=$(get_resp 14)
R_HEALTH2=$(get_resp 15)

echo ""
echo "--- Step 1: Initialize ---"
assert_ok "initialize returns protocolVersion" "$R_INIT" '.result.protocolVersion == "2024-11-05"'
assert_ok "initialize returns serverInfo" "$R_INIT" '.result.serverInfo.name == "m1nd-mcp"'

echo ""
echo "--- Step 2: m1nd.ingest (self-ingest) ---"
INGEST_TEXT=$(extract_content_text "$R_INGEST")
assert_ok "ingest: no error" "$R_INGEST" '(.result.isError // false) == false'
assert_ok "ingest: nodes_created > 0" "$INGEST_TEXT" '(.nodes_created // 0) > 0'
assert_ok "ingest: edges_created > 0" "$INGEST_TEXT" '(.edges_created // 0) > 0'
INGEST_NODES=$(echo "$INGEST_TEXT" | jq '.nodes_created // 0')
INGEST_EDGES=$(echo "$INGEST_TEXT" | jq '.edges_created // 0')
echo "         Nodes: $INGEST_NODES, Edges: $INGEST_EDGES"

echo ""
echo "--- Step 3: m1nd.health (post-ingest) ---"
HEALTH1_TEXT=$(extract_content_text "$R_HEALTH1")
assert_ok "health1: no error" "$R_HEALTH1" '(.result.isError // false) == false'
assert_ok "health1: node_count > 100" "$HEALTH1_TEXT" '(.node_count // 0) > 100'
assert_ok "health1: edge_count > 200" "$HEALTH1_TEXT" '(.edge_count // 0) > 200'
H1_NODES=$(echo "$HEALTH1_TEXT" | jq '.node_count // 0')
H1_EDGES=$(echo "$HEALTH1_TEXT" | jq '.edge_count // 0')
echo "         Health: $H1_NODES nodes, $H1_EDGES edges"

echo ""
echo "--- Step 4: m1nd.activate ('spreading activation') ---"
ACTIVATE_TEXT=$(extract_content_text "$R_ACTIVATE")
assert_ok "activate: no error" "$R_ACTIVATE" '(.result.isError // false) == false'
assert_ok "activate: results non-empty" "$ACTIVATE_TEXT" '((.activated // []) | length) > 0'
assert_ok "activate: top result activation > 0" "$ACTIVATE_TEXT" '(.activated[0].activation // 0) > 0'
# Sanity: all activations in valid range (not NaN, not negative)
ACTIVATE_SANE=$(echo "$ACTIVATE_TEXT" | jq '[.activated[]?.activation // empty | . >= 0 and . <= 1000] | all')
assert_ok "activate: all activations >= 0 and finite" "$ACTIVATE_SANE" '. == true'
echo "         Top result: $(echo "$ACTIVATE_TEXT" | jq -r '.activated[0].label // "N/A"') ($(echo "$ACTIVATE_TEXT" | jq '.activated[0].activation // 0'))"

echo ""
echo "--- Step 5: m1nd.impact ---"
IMPACT_TEXT=$(extract_content_text "$R_IMPACT")
assert_ok "impact: no error" "$R_IMPACT" '(.result.isError // false) == false'
assert_ok "impact: blast_radius non-empty" "$IMPACT_TEXT" '((.blast_radius // []) | length) > 0'
# Sanity: all signal_strengths >= 0
IMPACT_SANE=$(echo "$IMPACT_TEXT" | jq '[.blast_radius[]?.signal_strength // empty | . >= 0] | all')
assert_ok "impact: all signal_strengths >= 0" "$IMPACT_SANE" '. == true'
echo "         Blast radius: $(echo "$IMPACT_TEXT" | jq '(.blast_radius // []) | length') nodes"

echo ""
echo "--- Step 6: m1nd.missing ---"
MISSING_TEXT=$(extract_content_text "$R_MISSING")
assert_ok "missing: no error" "$R_MISSING" '(.result.isError // false) == false'
assert_ok "missing: returns structural_holes field" "$MISSING_TEXT" 'has("structural_holes")'
echo "         Structural holes: $(echo "$MISSING_TEXT" | jq '(.structural_holes // []) | length')"

echo ""
echo "--- Step 7: m1nd.why ---"
WHY_TEXT=$(extract_content_text "$R_WHY")
assert_ok "why: no error" "$R_WHY" '(.result.isError // false) == false'
assert_ok "why: has paths field" "$WHY_TEXT" 'has("paths")'
# Path may or may not exist depending on graph connectivity
WHY_FOUND=$(echo "$WHY_TEXT" | jq '.found // false')
echo "         Path found: $WHY_FOUND, Paths: $(echo "$WHY_TEXT" | jq '(.paths // []) | length')"

echo ""
echo "--- Step 8: m1nd.warmup ---"
WARMUP_TEXT=$(extract_content_text "$R_WARMUP")
assert_ok "warmup: no error" "$R_WARMUP" '(.result.isError // false) == false'
assert_ok "warmup: has seeds field" "$WARMUP_TEXT" 'has("seeds")'
assert_ok "warmup: total_seeds > 0" "$WARMUP_TEXT" '(.total_seeds // 0) > 0'
echo "         Seeds: $(echo "$WARMUP_TEXT" | jq '.total_seeds // 0'), Priming: $(echo "$WARMUP_TEXT" | jq '.total_priming // 0')"

echo ""
echo "--- Step 9: m1nd.counterfactual ---"
CF_TEXT=$(extract_content_text "$R_COUNTERFACTUAL")
assert_ok "counterfactual: no error" "$R_COUNTERFACTUAL" '(.result.isError // false) == false'
assert_ok "counterfactual: has total_impact" "$CF_TEXT" 'has("total_impact")'
assert_ok "counterfactual: total_impact >= 0" "$CF_TEXT" '(.total_impact // -1) >= 0'
# Check pct_activation_lost in [0, 100]
PCT_LOST=$(echo "$CF_TEXT" | jq '.pct_activation_lost // -1')
assert_ok "counterfactual: pct_activation_lost in [0,100]" "$CF_TEXT" '(.pct_activation_lost // -1) >= 0 and (.pct_activation_lost // 101) <= 100'
echo "         Impact: $(echo "$CF_TEXT" | jq '.total_impact // 0'), Pct lost: $PCT_LOST%"

echo ""
echo "--- Step 10: m1nd.predict ---"
PREDICT_TEXT=$(extract_content_text "$R_PREDICT")
assert_ok "predict: no error" "$R_PREDICT" '(.result.isError // false) == false'
assert_ok "predict: has predictions field" "$PREDICT_TEXT" 'has("predictions")'
echo "         Predictions: $(echo "$PREDICT_TEXT" | jq '(.predictions // []) | length')"

echo ""
echo "--- Step 11: m1nd.fingerprint ---"
FP_TEXT=$(extract_content_text "$R_FINGERPRINT")
assert_ok "fingerprint: no error" "$R_FINGERPRINT" '(.result.isError // false) == false'
# Should have either equivalents or equivalent_pairs
assert_ok "fingerprint: has data" "$FP_TEXT" 'has("equivalents") or has("equivalent_pairs") or has("target_node")'
echo "         Equivalents: $(echo "$FP_TEXT" | jq '(.equivalents // []) | length')"

echo ""
echo "--- Step 12: m1nd.drift ---"
DRIFT_TEXT=$(extract_content_text "$R_DRIFT")
assert_ok "drift: no error" "$R_DRIFT" '(.result.isError // false) == false'
assert_ok "drift: has weight_drift" "$DRIFT_TEXT" 'has("weight_drift")'
assert_ok "drift: has top_velocities" "$DRIFT_TEXT" 'has("top_velocities")'
echo "         Drifted edges: $(echo "$DRIFT_TEXT" | jq '(.weight_drift // []) | length'), Top velocities: $(echo "$DRIFT_TEXT" | jq '(.top_velocities // []) | length')"

echo ""
echo "--- Step 13: m1nd.learn ---"
LEARN_TEXT=$(extract_content_text "$R_LEARN")
assert_ok "learn: no error" "$R_LEARN" '(.result.isError // false) == false'
assert_ok "learn: edges_modified >= 0" "$LEARN_TEXT" '(.edges_modified // -1) >= 0'
echo "         Edges modified: $(echo "$LEARN_TEXT" | jq '.edges_modified // 0')"

echo ""
echo "--- Step 14: m1nd.resonate ---"
RESONATE_TEXT=$(extract_content_text "$R_RESONATE")
assert_ok "resonate: no error" "$R_RESONATE" '(.result.isError // false) == false'
assert_ok "resonate: has harmonics" "$RESONATE_TEXT" 'has("harmonics")'
assert_ok "resonate: has sympathetic_pairs" "$RESONATE_TEXT" 'has("sympathetic_pairs")'
assert_ok "resonate: has resonant_frequencies" "$RESONATE_TEXT" 'has("resonant_frequencies")'
assert_ok "resonate: has wave_pattern" "$RESONATE_TEXT" 'has("wave_pattern")'
echo "         Harmonics: $(echo "$RESONATE_TEXT" | jq '(.harmonics // []) | length'), Sympathetic pairs: $(echo "$RESONATE_TEXT" | jq '(.sympathetic_pairs // []) | length')"

echo ""
echo "--- Step 15: m1nd.health (post-all-tools) ---"
HEALTH2_TEXT=$(extract_content_text "$R_HEALTH2")
assert_ok "health2: no error" "$R_HEALTH2" '(.result.isError // false) == false'
assert_ok "health2: queries_processed > 0" "$HEALTH2_TEXT" '(.queries_processed // 0) > 0'
assert_ok "health2: active_sessions non-empty" "$HEALTH2_TEXT" '((.active_sessions // []) | length) > 0'
# Consistency: node/edge count should match health1
H2_NODES=$(echo "$HEALTH2_TEXT" | jq '.node_count // 0')
H2_EDGES=$(echo "$HEALTH2_TEXT" | jq '.edge_count // 0')
assert_ok "health2: node_count consistent ($H1_NODES == $H2_NODES)" "$HEALTH2_TEXT" ".node_count == $H1_NODES"
assert_ok "health2: edge_count consistent ($H1_EDGES == $H2_EDGES)" "$HEALTH2_TEXT" ".edge_count == $H1_EDGES"
echo "         Queries processed: $(echo "$HEALTH2_TEXT" | jq '.queries_processed // 0')"

# --- Phase 3: Persistence round-trip -----------------------------------------

echo ""
echo "================================================================"
echo " PHASE 3: Persistence round-trip"
echo "================================================================"

# Verify snapshot was written
if [ -f "$GRAPH_SNAP" ]; then
    echo "  Snapshot exists: $GRAPH_SNAP ($(wc -c < "$GRAPH_SNAP") bytes)"
else
    echo "  [FAIL] No snapshot file at $GRAPH_SNAP"
    FAIL=$((FAIL + 1))
    TOTAL=$((TOTAL + 1))
fi

# Restart server and verify graph survives
echo "  Restarting server..."

MSG_INIT2=$(init_msg)
MSG_HEALTH3=$(rpc "m1nd.health" '{"agent_id":"e2e-test-session2"}' 3)
MSG_ACTIVATE2=$(rpc "m1nd.activate" '{"query":"spreading activation","agent_id":"e2e-test-session2","top_k":20}' 4)

ALL_MSGS2=$(printf '%s\n' "$MSG_INIT2" "$MSG_HEALTH3" "$MSG_ACTIVATE2")

RESPONSES2=$(echo "$ALL_MSGS2" | M1ND_GRAPH_SOURCE="$GRAPH_SNAP" M1ND_PLASTICITY_STATE="$PLASTICITY" timeout 60 "$BINARY" 2>"$WORKDIR/stderr2.log")

R_HEALTH3=$(echo "$RESPONSES2" | while IFS= read -r line; do
    rid=$(echo "$line" | jq -r '.id // empty' 2>/dev/null)
    if [ "$rid" = "3" ]; then echo "$line"; break; fi
done)

R_ACTIVATE2=$(echo "$RESPONSES2" | while IFS= read -r line; do
    rid=$(echo "$line" | jq -r '.id // empty' 2>/dev/null)
    if [ "$rid" = "4" ]; then echo "$line"; break; fi
done)

echo ""
echo "--- Persistence: health after restart ---"
HEALTH3_TEXT=$(extract_content_text "$R_HEALTH3")
H3_NODES=$(echo "$HEALTH3_TEXT" | jq '.node_count // 0')
H3_EDGES=$(echo "$HEALTH3_TEXT" | jq '.edge_count // 0')
assert_ok "persistence: node_count matches pre-restart ($H1_NODES == $H3_NODES)" "$HEALTH3_TEXT" ".node_count == $H1_NODES"
assert_ok "persistence: edge_count matches pre-restart ($H1_EDGES == $H3_EDGES)" "$HEALTH3_TEXT" ".edge_count == $H1_EDGES"
echo "         Post-restart: $H3_NODES nodes, $H3_EDGES edges"

echo ""
echo "--- Persistence: activate after restart ---"
ACTIVATE2_TEXT=$(extract_content_text "$R_ACTIVATE2")
assert_ok "persistence: activate returns results" "$ACTIVATE2_TEXT" '((.activated // []) | length) > 0'
echo "         Top result: $(echo "$ACTIVATE2_TEXT" | jq -r '.activated[0].label // "N/A"')"

# --- Phase 4: Sanity assertions -----------------------------------------------

echo ""
echo "================================================================"
echo " PHASE 4: Sanity assertions"
echo "================================================================"

# Check for any JSON parse errors in responses
PARSE_ERRORS=0
for resp_var in "$R_INIT" "$R_INGEST" "$R_HEALTH1" "$R_ACTIVATE" "$R_IMPACT" "$R_MISSING" "$R_WHY" "$R_WARMUP" "$R_COUNTERFACTUAL" "$R_PREDICT" "$R_FINGERPRINT" "$R_DRIFT" "$R_LEARN" "$R_RESONATE" "$R_HEALTH2"; do
    if [ -z "$resp_var" ]; then
        PARSE_ERRORS=$((PARSE_ERRORS + 1))
    elif ! echo "$resp_var" | jq . >/dev/null 2>&1; then
        PARSE_ERRORS=$((PARSE_ERRORS + 1))
    fi
done
TOTAL=$((TOTAL + 1))
if [ "$PARSE_ERRORS" -eq 0 ]; then
    PASS=$((PASS + 1))
    echo "  [PASS] No JSON parse errors in any response"
else
    FAIL=$((FAIL + 1))
    echo "  [FAIL] $PARSE_ERRORS responses had JSON parse errors"
fi

# Check for isError in any tool response
ERROR_COUNT=0
for resp_var in "$R_INGEST" "$R_HEALTH1" "$R_ACTIVATE" "$R_IMPACT" "$R_MISSING" "$R_WHY" "$R_WARMUP" "$R_COUNTERFACTUAL" "$R_PREDICT" "$R_FINGERPRINT" "$R_DRIFT" "$R_LEARN" "$R_RESONATE" "$R_HEALTH2"; do
    is_err=$(echo "$resp_var" | jq -r '.result.isError // false' 2>/dev/null)
    if [ "$is_err" = "true" ]; then
        ERROR_COUNT=$((ERROR_COUNT + 1))
        tool_err=$(echo "$resp_var" | jq -r '.result.content[0].text // "unknown"' 2>/dev/null)
        echo "  [WARN] Tool error: $tool_err"
    fi
done
TOTAL=$((TOTAL + 1))
if [ "$ERROR_COUNT" -eq 0 ]; then
    PASS=$((PASS + 1))
    echo "  [PASS] No tool returned isError for valid inputs"
else
    FAIL=$((FAIL + 1))
    echo "  [FAIL] $ERROR_COUNT tools returned isError"
fi

# Check NaN/negative in activation scores
NAN_CHECK=$(echo "$ACTIVATE_TEXT" | jq '[.activated[]?.activation // empty | isnan or . < 0] | any' 2>/dev/null || echo "true")
TOTAL=$((TOTAL + 1))
if [ "$NAN_CHECK" = "false" ]; then
    PASS=$((PASS + 1))
    echo "  [PASS] No NaN or negative activation scores"
else
    FAIL=$((FAIL + 1))
    echo "  [FAIL] Found NaN or negative activation scores"
fi

# Check impact percentages in [0, 100]
if [ -n "$CF_TEXT" ]; then
    PCT_CHECK=$(echo "$CF_TEXT" | jq '(.pct_activation_lost // 0) >= 0 and (.pct_activation_lost // 0) <= 100' 2>/dev/null || echo "false")
    TOTAL=$((TOTAL + 1))
    if [ "$PCT_CHECK" = "true" ]; then
        PASS=$((PASS + 1))
        echo "  [PASS] Impact percentages in [0, 100]"
    else
        FAIL=$((FAIL + 1))
        echo "  [FAIL] Impact percentages out of range"
    fi
fi

# Check stderr for panics
STDERR_LOG="$WORKDIR/stderr.log"
if grep -qi "panic\|thread.*panicked\|SIGSEGV\|SIGABRT" "$STDERR_LOG" 2>/dev/null; then
    TOTAL=$((TOTAL + 1))
    FAIL=$((FAIL + 1))
    echo "  [FAIL] Server panicked! See: $STDERR_LOG"
else
    TOTAL=$((TOTAL + 1))
    PASS=$((PASS + 1))
    echo "  [PASS] No panics in server stderr"
fi

# --- Final report -------------------------------------------------------------

echo ""
echo "================================================================"
echo " FINAL REPORT"
echo "================================================================"
echo ""
echo "  Results: $PASS passed / $FAIL failed / $TOTAL total"
echo ""
echo "  Ingest stats:"
echo "    Nodes created: $INGEST_NODES"
echo "    Edges created: $INGEST_EDGES"
echo ""
echo "  Final health:"
echo "    Node count: $H1_NODES"
echo "    Edge count: $H1_EDGES"
echo "    Queries processed: $(echo "$HEALTH2_TEXT" | jq '.queries_processed // 0')"
echo ""
echo "  Persistence:"
echo "    Pre-restart:  $H1_NODES nodes, $H1_EDGES edges"
echo "    Post-restart: $H3_NODES nodes, $H3_EDGES edges"
echo ""

if [ "$FAIL" -gt 0 ]; then
    echo "  Issues found:"
    echo -e "$ISSUES"
    echo ""
    echo "  STATUS: FAIL — $FAIL issues need fixing"
    # Save detailed responses for debugging
    echo "$R_INIT" > "$WORKDIR/resp_init.json"
    echo "$R_INGEST" > "$WORKDIR/resp_ingest.json"
    echo "$R_HEALTH1" > "$WORKDIR/resp_health1.json"
    echo "$R_ACTIVATE" > "$WORKDIR/resp_activate.json"
    echo "$R_IMPACT" > "$WORKDIR/resp_impact.json"
    echo "$R_MISSING" > "$WORKDIR/resp_missing.json"
    echo "$R_WHY" > "$WORKDIR/resp_why.json"
    echo "$R_WARMUP" > "$WORKDIR/resp_warmup.json"
    echo "$R_COUNTERFACTUAL" > "$WORKDIR/resp_counterfactual.json"
    echo "$R_PREDICT" > "$WORKDIR/resp_predict.json"
    echo "$R_FINGERPRINT" > "$WORKDIR/resp_fingerprint.json"
    echo "$R_DRIFT" > "$WORKDIR/resp_drift.json"
    echo "$R_LEARN" > "$WORKDIR/resp_learn.json"
    echo "$R_RESONATE" > "$WORKDIR/resp_resonate.json"
    echo "$R_HEALTH2" > "$WORKDIR/resp_health2.json"
    echo "  Debug responses saved to: $WORKDIR/"
else
    echo "  STATUS: PASS — all tools produce sane results"
    echo "  Session 1 of 3 clean sessions: COMPLETE"
fi

echo ""
echo "  Workdir: $WORKDIR"
echo "  Server logs: $WORKDIR/stderr.log"
echo ""

# Cleanup: remove workdir only on success
if [ "$FAIL" -eq 0 ]; then
    rm -rf "$WORKDIR"
fi

exit "$FAIL"
