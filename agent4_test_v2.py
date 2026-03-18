#!/usr/bin/env python3
"""
AGENT 4 v2 — Correct API schemas for trace, validate_plan, predict, warmup
"""
import json
import subprocess
import os
import tempfile
import time
import select
import sys
import traceback as tb


class M1nd:
    def __init__(self):
        self.w = tempfile.mkdtemp(prefix='m1nd_a4v2_')
        env = os.environ.copy()
        env['M1ND_GRAPH_SOURCE'] = os.path.join(self.w, 'g.json')
        env['M1ND_PLASTICITY_STATE'] = os.path.join(self.w, 'p.json')
        self.p = subprocess.Popen(
            ['./target/release/m1nd-mcp'],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            env=env,
            bufsize=0,
            cwd='/Users/cosmophonix/clawd/roomanizer-os/mcp/m1nd'
        )
        self.req_id = 0

    def _call(self, method, params=None, timeout=60):
        self.req_id += 1
        msg = {'jsonrpc': '2.0', 'id': self.req_id, 'method': method, 'params': params or {}}
        raw = json.dumps(msg)
        frame = f'Content-Length: {len(raw)}\r\n\r\n{raw}'
        self.p.stdin.write(frame.encode())
        self.p.stdin.flush()
        headers = b''
        deadline = time.time() + timeout
        while time.time() < deadline:
            r, _, _ = select.select([self.p.stdout], [], [], 1.0)
            if r:
                b = self.p.stdout.read(1)
                if not b:
                    return {'error': 'EOF'}
                headers += b
                if headers.endswith(b'\r\n\r\n'):
                    break
        else:
            return {'error': 'TIMEOUT'}
        cl = int(headers.decode().split('Content-Length: ')[1].split('\r\n')[0])
        body = b''
        while len(body) < cl and time.time() < deadline:
            r, _, _ = select.select([self.p.stdout], [], [], 1.0)
            if r:
                chunk = self.p.stdout.read(min(cl - len(body), 65536))
                if not chunk:
                    return {'error': 'EOF'}
                body += chunk
        return json.loads(body)

    def tool(self, name, args=None, timeout=180):
        t0 = time.time()
        r = self._call('tools/call', {'name': name, 'arguments': args or {}}, timeout=timeout)
        elapsed = time.time() - t0
        if 'error' in r and isinstance(r['error'], str):
            return {'_error': r['error'], '_elapsed': elapsed}
        # Check for error in result content
        if 'error' in r.get('result', {}):
            return {'_server_error': r['result']['error'], '_elapsed': elapsed}
        content = r.get('result', {}).get('content', [{}])
        text = content[0].get('text', '') if content else ''
        try:
            result = json.loads(text)
        except Exception:
            result = {'_raw': text[:8000]}
        result['_elapsed'] = elapsed
        return result

    def close(self):
        try:
            self.p.stdin.close()
            self.p.wait(timeout=10)
        except Exception:
            self.p.kill()


def pp(label, data, truncate=3000):
    print(f"\n{'='*60}")
    print(f"  {label}")
    print('='*60)
    s = json.dumps(data, indent=2, default=str)
    if len(s) > truncate:
        s = s[:truncate] + f'\n... [truncated, total {len(s)} chars]'
    print(s)


def run_all():
    results = {}
    t_start = time.time()

    print("\n" + "="*60)
    print("  AGENT 4 v2 — m1nd REAL API TEST")
    print("="*60)

    print("\n[INIT] Starting m1nd...")
    m = M1nd()
    time.sleep(2)

    # ── HEALTH CHECK ─────────────────────────────────────────
    print("\n[HEALTH] Checking m1nd server...")
    health = m.tool('m1nd.health', {'agent_id': 'agent4'})
    pp("HEALTH", health)
    results['health'] = health

    # ── INGEST ────────────────────────────────────────────────
    print("\n[INGEST] Loading backend codebase (path, not paths)...")
    t_ingest = time.time()
    ingest = m.tool('m1nd.ingest', {
        'agent_id': 'agent4',
        'path': '/Users/cosmophonix/clawd/roomanizer-os/backend',
        'adapter': 'code',
        'mode': 'replace'
    }, timeout=300)
    ingest_elapsed = time.time() - t_ingest
    print(f"[INGEST] Done in {ingest_elapsed:.1f}s")
    pp("INGEST RESULT", ingest)
    results['ingest'] = {'elapsed': ingest_elapsed, 'result': ingest}

    # Post-ingest health to confirm graph loaded
    print("\n[HEALTH-POST-INGEST] Verify graph populated...")
    health2 = m.tool('m1nd.health', {'agent_id': 'agent4'})
    pp("HEALTH POST-INGEST", health2)
    results['health_post_ingest'] = health2

    # ── discover some real node IDs for predict ───────────────
    # Use activate to find nodes related to our target files
    print("\n[DISCOVER] Getting node IDs via activate (chat_handler, spawner, etc.)...")
    activate_result = m.tool('m1nd.activate', {
        'agent_id': 'agent4',
        'query': 'chat_handler worker_pool spawner config main stormender_v2',
        'top_k': 20
    })
    pp("ACTIVATE (node discovery)", activate_result)
    results['activate_discovery'] = activate_result

    # Extract node IDs from activate result
    node_ids = {}
    if isinstance(activate_result, dict) and 'nodes' in activate_result:
        for n in activate_result.get('nodes', []):
            label = n.get('label', '')
            nid = n.get('node_id', n.get('id', ''))
            if nid:
                node_ids[label] = nid
                print(f"  Found node: {label} -> {nid}")
    elif isinstance(activate_result, dict) and '_raw' in activate_result:
        print(f"  Raw: {activate_result['_raw'][:500]}")

    print(f"\n  Node IDs discovered: {list(node_ids.keys())}")

    # ── SECTION 1: m1nd.trace ─────────────────────────────────
    print("\n\n" + "#"*60)
    print("# SECTION 1: m1nd.trace (field: error_text)")
    print("#"*60)

    trace_cases = [
        {
            'name': 'RuntimeError — worker_pool exhaustion via spawner',
            'error_text': """Traceback (most recent call last):
  File "/Users/cosmophonix/clawd/roomanizer-os/backend/chat_handler.py", line 287, in handle_chat_message
    result = await asyncio.wait_for(
  File "/usr/lib/python3.12/asyncio/tasks.py", line 520, in wait_for
    return await fut
  File "/Users/cosmophonix/clawd/roomanizer-os/backend/worker_pool.py", line 134, in acquire_worker
    worker = await self._pool_queue.get()
  File "/Users/cosmophonix/clawd/roomanizer-os/backend/spawner.py", line 219, in spawn_agent
    slot = await self.worker_pool.acquire(timeout=30.0)
  File "/Users/cosmophonix/clawd/roomanizer-os/backend/worker_pool.py", line 89, in acquire
    raise RuntimeError("Worker pool exhausted: all 8 slots occupied, timeout after 30s")
RuntimeError: Worker pool exhausted: all 8 slots occupied, timeout after 30s""",
            'language': 'python'
        },
        {
            'name': 'httpx.ReadTimeout — SSE stream in opencode_engine',
            'error_text': """Traceback (most recent call last):
  File "/Users/cosmophonix/clawd/roomanizer-os/backend/stormender_v2_runtime.py", line 156, in execute_lane
    async for event in engine.stream_completion(prompt, task_id):
  File "/Users/cosmophonix/clawd/roomanizer-os/backend/opencode_engine.py", line 341, in stream_completion
    async for chunk in response.aiter_lines():
  File "/usr/lib/python3.12/site-packages/httpx/_client.py", line 1842, in aiter_lines
    async for line in self._client._transport.handle_async_request(request):
  File "/Users/cosmophonix/clawd/roomanizer-os/backend/opencode_engine.py", line 298, in _connect_sse
    response = await self._client.get(self._sse_url, timeout=httpx.Timeout(120.0, read=60.0))
httpx.ReadTimeout: Read timed out after 60.0 seconds while streaming SSE from opencode daemon""",
            'language': 'python'
        },
        {
            'name': 'KeyError — principal not found in principal_registry',
            'error_text': """Traceback (most recent call last):
  File "/Users/cosmophonix/clawd/roomanizer-os/backend/identity_routes.py", line 78, in get_principal_identity
    identity = await registry.get_principal(principal_id)
  File "/Users/cosmophonix/clawd/roomanizer-os/backend/principal_registry.py", line 203, in get_principal
    record = self._principals[principal_id]
KeyError: 'agent-7f3a9b1c-not-found'

During handling of the above exception, another exception occurred:

  File "/Users/cosmophonix/clawd/roomanizer-os/backend/identity_routes.py", line 81, in get_principal_identity
    raise HTTPException(status_code=404, detail=f"Principal {principal_id} not found in registry")
starlette.exceptions.HTTPException: 404 - Principal agent-7f3a9b1c-not-found not found in registry""",
            'language': 'python'
        }
    ]

    results['trace'] = []
    for i, case in enumerate(trace_cases, 1):
        print(f"\n[TRACE {i}] {case['name']}")
        r = m.tool('m1nd.trace', {
            'agent_id': 'agent4',
            'error_text': case['error_text'],
            'language': case['language'],
            'top_k': 10
        })
        elapsed = r.get('_elapsed', 0)
        print(f"  elapsed: {elapsed:.3f}s")
        pp(f"TRACE {i}: {case['name']}", r)
        results['trace'].append({
            'test_name': case['name'],
            'elapsed': elapsed,
            'result': {k: v for k, v in r.items() if k != '_elapsed'}
        })

    # ── SECTION 2: m1nd.validate_plan ────────────────────────
    print("\n\n" + "#"*60)
    print("# SECTION 2: m1nd.validate_plan")
    print("#"*60)

    plan_cases = [
        {
            'name': 'Plan 1 — Simple: single file patch chat_handler.py',
            'actions': [
                {
                    'action_type': 'modify',
                    'file_path': 'backend/chat_handler.py',
                    'description': 'Increase deep-work escalation threshold from 3 to 5 consecutive long messages'
                }
            ]
        },
        {
            'name': 'Plan 2 — Medium: rate-limit retry across 3 engine files',
            'actions': [
                {
                    'action_type': 'modify',
                    'file_path': 'backend/opencode_engine.py',
                    'description': 'Add exponential backoff retry on 429 responses in stream_completion()'
                },
                {
                    'action_type': 'modify',
                    'file_path': 'backend/cli_adapters.py',
                    'description': 'Add rate-limit detection and retry logic in CLIAdapter base class'
                },
                {
                    'action_type': 'modify',
                    'file_path': 'backend/config.py',
                    'description': 'Add RATE_LIMIT_MAX_RETRIES and RATE_LIMIT_BASE_DELAY config constants'
                }
            ]
        },
        {
            'name': 'Plan 3 — Complex: new WhatsApp group broadcast feature (7 actions)',
            'actions': [
                {
                    'action_type': 'modify',
                    'file_path': 'backend/whatsapp_manager.py',
                    'description': 'Add broadcast_to_group() method that sends message to all participants'
                },
                {
                    'action_type': 'modify',
                    'file_path': 'backend/whatsapp_routes.py',
                    'description': 'Add POST /api/whatsapp/broadcast endpoint'
                },
                {
                    'action_type': 'modify',
                    'file_path': 'backend/whatsapp_models.py',
                    'description': 'Add BroadcastRequest and BroadcastResult Pydantic models'
                },
                {
                    'action_type': 'modify',
                    'file_path': 'backend/whatsapp_store.py',
                    'description': 'Add group_participants() query and broadcast_log table'
                },
                {
                    'action_type': 'create',
                    'file_path': 'backend/whatsapp_broadcast.py',
                    'description': 'New module: broadcast engine with rate limiting and delivery tracking'
                },
                {
                    'action_type': 'test',
                    'file_path': 'backend/tests/test_whatsapp_broadcast.py',
                    'description': 'Unit tests for broadcast engine and delivery tracking'
                },
                {
                    'action_type': 'modify',
                    'file_path': 'backend/main.py',
                    'description': 'Register whatsapp_broadcast router in app lifespan'
                }
            ]
        }
    ]

    results['validate_plan'] = []
    for i, case in enumerate(plan_cases, 1):
        print(f"\n[VALIDATE_PLAN {i}] {case['name']}")
        r = m.tool('m1nd.validate_plan', {
            'agent_id': 'agent4',
            'actions': case['actions'],
            'include_risk_score': True,
            'include_test_impact': True
        })
        elapsed = r.get('_elapsed', 0)
        print(f"  elapsed: {elapsed:.3f}s")
        pp(f"VALIDATE_PLAN {i}: {case['name']}", r)
        results['validate_plan'].append({
            'test_name': case['name'],
            'elapsed': elapsed,
            'result': {k: v for k, v in r.items() if k != '_elapsed'}
        })

    # ── SECTION 3: m1nd.predict ───────────────────────────────
    print("\n\n" + "#"*60)
    print("# SECTION 3: m1nd.predict (uses changed_node, need real node IDs)")
    print("#"*60)

    # First try an activate query to get exact node IDs
    print("\n[NODE DISCOVERY] Querying each file via activate...")

    file_to_query = {
        'chat_handler.py': 'chat_handler handle_chat',
        'config.py': 'config settings Config',
        'spawner.py': 'spawner spawn_agent',
        'main.py': 'main app FastAPI lifespan',
        'stormender_v2.py': 'stormender_v2 storm phase',
    }

    discovered_nodes = {}
    for fname, query in file_to_query.items():
        act = m.tool('m1nd.activate', {
            'agent_id': 'agent4',
            'query': query,
            'top_k': 5
        })
        nodes = act.get('nodes', [])
        print(f"\n  Query '{query}' -> {len(nodes)} nodes:")
        for n in nodes[:5]:
            label = n.get('label', '')
            nid = n.get('node_id', '')
            score = n.get('activation', n.get('score', 0))
            print(f"    {label} [{nid}] score={score:.3f}")
            if fname.replace('.py', '').lower() in label.lower():
                discovered_nodes[fname] = nid

    print(f"\n  Discovered: {discovered_nodes}")

    results['predict'] = []
    # If we have node IDs, test predict; otherwise use the node IDs from activate
    # Get all nodes from a broader activate to have IDs
    all_act = m.tool('m1nd.activate', {
        'agent_id': 'agent4',
        'query': 'chat_handler config spawner main stormender worker_pool opencode principal',
        'top_k': 30
    })
    pp("ACTIVATE ALL (node ID harvest)", all_act)
    results['activate_all'] = {k: v for k, v in all_act.items() if k != '_elapsed'}

    # Build node_id map from all_act
    nid_map = {}
    for n in all_act.get('nodes', []):
        label = n.get('label', '')
        nid = n.get('node_id', '')
        if label and nid:
            nid_map[label] = nid

    print(f"\n  All harvested node IDs ({len(nid_map)}):")
    for label, nid in list(nid_map.items())[:20]:
        print(f"    {label} -> {nid}")

    # Map filenames to likely node IDs
    predict_targets = []
    # Try direct label match
    for label, nid in nid_map.items():
        clean = label.lower()
        for fname in ['chat_handler', 'config', 'spawner', 'main', 'stormender_v2']:
            if fname in clean and (fname, nid) not in [(p['file'], p['node_id']) for p in predict_targets]:
                predict_targets.append({'file': fname, 'node_id': nid, 'label': label})
                break

    # Fallback: use first nodes
    if len(predict_targets) < 5:
        all_nodes = all_act.get('nodes', [])
        for n in all_nodes[:10]:
            nid = n.get('node_id', '')
            label = n.get('label', '')
            if nid and not any(p['node_id'] == nid for p in predict_targets):
                predict_targets.append({'file': label, 'node_id': nid, 'label': label})
                if len(predict_targets) >= 5:
                    break

    print(f"\n  Predict targets: {[(p['file'], p['node_id']) for p in predict_targets]}")

    for pt in predict_targets:
        print(f"\n[PREDICT] node={pt['node_id']} (label={pt['label']}, file={pt['file']})")
        r = m.tool('m1nd.predict', {
            'agent_id': 'agent4',
            'changed_node': pt['node_id'],
            'include_velocity': True,
            'top_k': 10
        })
        elapsed = r.get('_elapsed', 0)
        print(f"  elapsed: {elapsed:.3f}s")
        pp(f"PREDICT: {pt['label']}", r)
        results['predict'].append({
            'node_id': pt['node_id'],
            'label': pt['label'],
            'file': pt['file'],
            'elapsed': elapsed,
            'result': {k: v for k, v in r.items() if k != '_elapsed'}
        })

    # ── SECTION 4: m1nd.warmup ────────────────────────────────
    print("\n\n" + "#"*60)
    print("# SECTION 4: m1nd.warmup (field: task_description)")
    print("#"*60)

    warmup_cases = [
        "adding WhatsApp group message support",
        "fixing race condition in session pool",
        "refactoring storm manager error handling"
    ]

    results['warmup'] = []
    for task in warmup_cases:
        print(f"\n[WARMUP] '{task}'")
        r = m.tool('m1nd.warmup', {
            'agent_id': 'agent4',
            'task_description': task,  # correct field name
            'boost_strength': 0.15
        })
        elapsed = r.get('_elapsed', 0)
        print(f"  elapsed: {elapsed:.3f}s")
        pp(f"WARMUP: {task}", r)
        results['warmup'].append({
            'task': task,
            'elapsed': elapsed,
            'result': {k: v for k, v in r.items() if k != '_elapsed'}
        })

    # ── TEARDOWN ─────────────────────────────────────────────
    m.close()
    results['total_elapsed'] = time.time() - t_start
    print(f"\n[DONE] Total: {results['total_elapsed']:.1f}s")
    return results


if __name__ == '__main__':
    try:
        results = run_all()
        out_path = '/Users/cosmophonix/clawd/roomanizer-os/mcp/m1nd/agent4_raw_v2.json'
        with open(out_path, 'w') as f:
            json.dump(results, f, indent=2, default=str)
        print(f"[SAVED] {out_path}")
    except Exception as e:
        print(f"\n[FATAL] {e}")
        tb.print_exc()
        sys.exit(1)
