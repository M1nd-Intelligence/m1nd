#!/bin/bash
# Test m1nd MCP server with various queries
BINARY="./target/release/m1nd-mcp"

echo "=== m1nd MCP Server Stress Test ==="
echo ""

# Test 1: Initialize
echo "--- Test 1: Initialize ---"
echo '{"jsonrpc":"2.0","method":"initialize","id":1,"params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"0.1"}}}' | timeout 5 $BINARY 2>/dev/null

echo ""
echo "--- Test 2: List tools ---"
echo -e '{"jsonrpc":"2.0","method":"initialize","id":1,"params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"0.1"}}}\n{"jsonrpc":"2.0","method":"tools/list","id":2,"params":{}}' | timeout 5 $BINARY 2>/dev/null

echo ""
echo "--- Test 3: Health check ---"
echo -e '{"jsonrpc":"2.0","method":"initialize","id":1,"params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"0.1"}}}\n{"jsonrpc":"2.0","method":"tools/call","id":3,"params":{"name":"m1nd.status","arguments":{}}}' | timeout 5 $BINARY 2>/dev/null

echo ""
echo "=== Tests complete ==="
