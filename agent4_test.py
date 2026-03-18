#!/usr/bin/env python3
"""
AGENT 4 — Test m1nd trace, validate_plan, predict, warmup
Real execution, real data.
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
        self.w = tempfile.mkdtemp(prefix='m1nd_a4_')
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

    def tool(self, name, args=None, timeout=120):
        t0 = time.time()
        r = self._call('tools/call', {'name': name, 'arguments': args or {}}, timeout=timeout)
        elapsed = time.time() - t0
        if 'error' in r and isinstance(r['error'], str):
            return {'_error': r['error'], '_elapsed': elapsed}
        content = r.get('result', {}).get('content', [{}])
        text = content[0].get('text', '') if content else ''
        try:
            result = json.loads(text)
        except Exception:
            result = {'_raw': text[:5000]}
        result['_elapsed'] = elapsed
        return result

    def list_tools(self):
        r = self._call('tools/list', {}, timeout=30)
        tools = r.get('result', {}).get('tools', [])
        return [t['name'] for t in tools]

    def close(self):
        try:
            self.p.stdin.close()
            self.p.wait(timeout=10)
        except Exception:
            self.p.kill()


def pp(label, data):
    """Pretty print with label"""
    print(f"\n{'='*60}")
    print(f"  {label}")
    print('='*60)
    if isinstance(data, dict):
        for k, v in data.items():
            if k.startswith('_'):
                continue
            if isinstance(v, (dict, list)):
                print(f"  {k}: {json.dumps(v, indent=2)}")
            else:
                print(f"  {k}: {v}")
    else:
        print(json.dumps(data, indent=2))


def run_all():
    results = {}

    print("\n" + "="*60)
    print("  AGENT 4 — m1nd TRACE + VALIDATE_PLAN + PREDICT + WARMUP")
    print("="*60)

    # ── INIT ──────────────────────────────────────────────────
    print("\n[INIT] Starting m1nd server...")
    t_start = time.time()
    m = M1nd()
    time.sleep(2)  # let server boot

    # List tools to confirm what's available
    tools = m.list_tools()
    print(f"[INIT] Tools available ({len(tools)}): {sorted(tools)}")
    results['available_tools'] = sorted(tools)

    # ── INGEST ────────────────────────────────────────────────
    print("\n[INGEST] Loading backend codebase...")
    t_ingest = time.time()
    ingest_result = m.tool('m1nd.ingest', {
        'agent_id': 'agent4-research',
        'paths': ['/Users/cosmophonix/clawd/roomanizer-os/backend'],
        'incremental': False
    }, timeout=300)
    ingest_elapsed = time.time() - t_ingest
    print(f"[INGEST] Done in {ingest_elapsed:.1f}s")
    pp("INGEST RESULT", ingest_result)
    results['ingest'] = {
        'elapsed': ingest_elapsed,
        'result': ingest_result
    }

    # ── TRACE TESTS ──────────────────────────────────────────
    print("\n\n" + "#"*60)
    print("# SECTION 1: m1nd.trace")
    print("#"*60)

    trace_tests = [
        {
            'name': 'RuntimeError — worker_pool exhaustion in spawner.py',
            'stacktrace': """Traceback (most recent call last):
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
            'name': 'httpx.ReadTimeout — SSE stream in opencode_engine.py',
            'stacktrace': """Traceback (most recent call last):
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
            'name': 'KeyError — principal not found in principal_registry.py',
            'stacktrace': """Traceback (most recent call last):
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
    for i, test in enumerate(trace_tests, 1):
        print(f"\n[TRACE {i}] {test['name']}")
        t0 = time.time()
        r = m.tool('m1nd.trace', {
            'agent_id': 'agent4-research',
            'stacktrace': test['stacktrace'],
            'language': test['language']
        }, timeout=60)
        elapsed = time.time() - t0
        print(f"  elapsed: {elapsed:.3f}s")
        pp(f"TRACE {i}: {test['name']}", r)
        results['trace'].append({
            'test_name': test['name'],
            'elapsed': elapsed,
            'result': {k: v for k, v in r.items() if k != '_elapsed'}
        })

    # ── VALIDATE_PLAN TESTS ──────────────────────────────────
    print("\n\n" + "#"*60)
    print("# SECTION 2: m1nd.validate_plan")
    print("#"*60)

    plan_tests = [
        {
            'name': 'Plan 1 — Simple single-file: patch chat_handler.py deep-work threshold',
            'plan': {
                'agent_id': 'agent4-research',
                'actions': [
                    {
                        'action_type': 'modify',
                        'file_path': 'backend/chat_handler.py',
                        'description': 'Increase deep-work escalation threshold from 3 to 5 consecutive long messages'
                    }
                ]
            }
        },
        {
            'name': 'Plan 2 — Medium multi-file: add rate-limit retry to 3 engine files',
            'plan': {
                'agent_id': 'agent4-research',
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
            }
        },
        {
            'name': 'Plan 3 — Complex cross-cutting: new WhatsApp group broadcast feature',
            'plan': {
                'agent_id': 'agent4-research',
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
        }
    ]

    results['validate_plan'] = []
    for i, test in enumerate(plan_tests, 1):
        print(f"\n[VALIDATE_PLAN {i}] {test['name']}")
        t0 = time.time()
        r = m.tool('m1nd.validate_plan', test['plan'], timeout=60)
        elapsed = time.time() - t0
        print(f"  elapsed: {elapsed:.3f}s")
        pp(f"VALIDATE_PLAN {i}: {test['name']}", r)
        results['validate_plan'].append({
            'test_name': test['name'],
            'elapsed': elapsed,
            'result': {k: v for k, v in r.items() if k != '_elapsed'}
        })

    # ── PREDICT TESTS ────────────────────────────────────────
    print("\n\n" + "#"*60)
    print("# SECTION 3: m1nd.predict")
    print("#"*60)

    predict_targets = [
        'backend/chat_handler.py',
        'backend/config.py',
        'backend/spawner.py',
        'backend/main.py',
        'backend/stormender_v2.py',
    ]

    results['predict'] = []
    for target in predict_targets:
        print(f"\n[PREDICT] {target}")
        t0 = time.time()
        r = m.tool('m1nd.predict', {
            'agent_id': 'agent4-research',
            'file_path': target
        }, timeout=60)
        elapsed = time.time() - t0
        print(f"  elapsed: {elapsed:.3f}s")
        pp(f"PREDICT: {target}", r)
        results['predict'].append({
            'file': target,
            'elapsed': elapsed,
            'result': {k: v for k, v in r.items() if k != '_elapsed'}
        })

    # ── WARMUP TESTS ─────────────────────────────────────────
    print("\n\n" + "#"*60)
    print("# SECTION 4: m1nd.warmup")
    print("#"*60)

    warmup_tasks = [
        "adding WhatsApp group message support",
        "fixing race condition in session pool",
        "refactoring storm manager error handling"
    ]

    results['warmup'] = []
    for task in warmup_tasks:
        print(f"\n[WARMUP] Task: '{task}'")
        t0 = time.time()
        r = m.tool('m1nd.warmup', {
            'agent_id': 'agent4-research',
            'task': task
        }, timeout=60)
        elapsed = time.time() - t0
        print(f"  elapsed: {elapsed:.3f}s")
        pp(f"WARMUP: {task}", r)
        results['warmup'].append({
            'task': task,
            'elapsed': elapsed,
            'result': {k: v for k, v in r.items() if k != '_elapsed'}
        })

    # ── TEARDOWN ─────────────────────────────────────────────
    m.close()
    total_elapsed = time.time() - t_start
    results['total_elapsed'] = total_elapsed
    print(f"\n[DONE] Total elapsed: {total_elapsed:.1f}s")

    return results


if __name__ == '__main__':
    try:
        results = run_all()
        # Dump raw results for the report writer
        out_path = '/Users/cosmophonix/clawd/roomanizer-os/mcp/m1nd/agent4_raw_results.json'
        with open(out_path, 'w') as f:
            json.dump(results, f, indent=2, default=str)
        print(f"\n[SAVED] Raw results: {out_path}")
    except Exception as e:
        print(f"\n[FATAL] {e}")
        tb.print_exc()
        sys.exit(1)
