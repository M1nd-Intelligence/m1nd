#!/bin/bash
# E2E smoke test for m1nd GUI
# Usage: ./test_gui_e2e.sh
#
# Prerequisites:
#   - Rust toolchain (cargo build)
#   - Node.js + npm (frontend build)
#   - graph_snapshot.json in current directory (or an empty graph)
#
# Tests:
#   1. Frontend builds without TypeScript errors
#   2. Rust binary builds with "serve" feature
#   3. Server starts and serves index.html
#   4. Health endpoint responds with node_count
#   5. Activate tool returns activated nodes
#   6. Subgraph endpoint returns nodes array

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

PORT=11337
PASS=0
FAIL=0
SERVER_PID=""

cleanup() {
  if [ -n "$SERVER_PID" ] && kill -0 "$SERVER_PID" 2>/dev/null; then
    kill "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT

ok() {
  echo "  PASS: $1"
  PASS=$((PASS + 1))
}

fail() {
  echo "  FAIL: $1"
  FAIL=$((FAIL + 1))
}

echo "=== m1nd GUI E2E Smoke Test ==="
echo ""

# --- Step 1: Build frontend ---
echo "[1/6] Building frontend (npm run build)..."
cd m1nd-ui
if npm run build 2>&1 | tail -5; then
  ok "frontend build (tsc + vite)"
else
  fail "frontend build"
  echo "  ERROR: TypeScript or Vite build failed. Aborting."
  exit 1
fi
cd "$SCRIPT_DIR"
echo ""

# --- Step 2: Build Rust binary with serve feature ---
echo "[2/6] Building Rust binary (cargo build --release --features serve)..."
if cargo build --release --features serve 2>&1 | tail -3; then
  ok "cargo build --features serve"
else
  fail "cargo build"
  echo "  ERROR: Rust build failed. Aborting."
  exit 1
fi
echo ""

# --- Step 3: Start server ---
echo "[3/6] Starting m1nd server on port $PORT..."
GRAPH_FILE="./graph_snapshot.json"
if [ ! -f "$GRAPH_FILE" ]; then
  echo '{"nodes":[],"edges":[]}' > "$GRAPH_FILE"
  echo "  (created empty graph_snapshot.json)"
fi

M1ND_GRAPH_SOURCE="$GRAPH_FILE" ./target/release/m1nd-mcp --serve --port "$PORT" &
SERVER_PID=$!
sleep 2

if ! kill -0 "$SERVER_PID" 2>/dev/null; then
  fail "server start"
  echo "  ERROR: Server failed to start."
  exit 1
fi
ok "server started (pid=$SERVER_PID)"
echo ""

# --- Step 4: Test index.html ---
echo "[4/6] Testing index.html..."
if curl -sf "http://localhost:$PORT/" | grep -q "m1nd"; then
  ok "index.html serves and contains 'm1nd'"
else
  fail "index.html"
fi

# --- Step 5: Test health endpoint ---
echo "[5/6] Testing /api/health..."
HEALTH=$(curl -sf "http://localhost:$PORT/api/health" 2>/dev/null || echo "")
if echo "$HEALTH" | grep -q "node_count"; then
  ok "health endpoint returns node_count"
else
  fail "health endpoint"
fi

# --- Step 6: Test activate + subgraph ---
echo "[6/6] Testing tool endpoints..."
ACTIVATE=$(curl -sf -X POST "http://localhost:$PORT/api/tools/m1nd.activate" \
  -H "Content-Type: application/json" \
  -d '{"agent_id":"test","query":"test","top_k":3}' 2>/dev/null || echo "")
if echo "$ACTIVATE" | grep -q "result"; then
  ok "activate tool"
else
  fail "activate tool"
fi

SUBGRAPH=$(curl -sf "http://localhost:$PORT/api/graph/subgraph?query=test&top_k=3" 2>/dev/null || echo "")
if echo "$SUBGRAPH" | grep -q "nodes"; then
  ok "subgraph endpoint"
else
  fail "subgraph endpoint"
fi

# --- Summary ---
echo ""
echo "=== Results: $PASS passed, $FAIL failed ==="
if [ "$FAIL" -gt 0 ]; then
  exit 1
fi
exit 0
