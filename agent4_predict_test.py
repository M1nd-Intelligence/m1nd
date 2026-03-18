#!/usr/bin/env python3
"""
Agent 4 — predict-only follow-up with correct node IDs from activate
Also test validate_plan with correct (relative) file paths
"""
import json
import subprocess
import os
import tempfile
import time
import select
import sys


class M1nd:
    def __init__(self):
        self.w = tempfile.mkdtemp(prefix='m1nd_a4p_')
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


def main():
    print("\n[INIT] Starting m1nd for predict+validate_plan refinement...")
    m = M1nd()
    time.sleep(2)

    # Ingest
    print("[INGEST] Loading backend...")
    ing = m.tool('m1nd.ingest', {
        'agent_id': 'agent4',
        'path': '/Users/cosmophonix/clawd/roomanizer-os/backend',
        'adapter': 'code',
        'mode': 'replace'
    }, timeout=300)
    print(f"  nodes={ing.get('node_count')}, edges={ing.get('edge_count')}, files={ing.get('files_parsed')}")

    results = {}

    # ── PREDICT: discover node IDs correctly ─────────────────
    print("\n\n[PREDICT SETUP] Harvesting node IDs from activate...")
    act = m.tool('m1nd.activate', {
        'agent_id': 'agent4',
        'query': 'chat_handler config spawner main stormender worker_pool opencode principal',
        'top_k': 50
    })

    # Key is 'activated', not 'nodes'
    activated_nodes = act.get('activated', [])
    print(f"  Got {len(activated_nodes)} nodes from activate")

    nid_map = {}
    for n in activated_nodes:
        label = n.get('label', '')
        nid = n.get('node_id', '')
        ntype = n.get('type', '')
        if nid:
            nid_map[label] = {'node_id': nid, 'type': ntype, 'activation': n.get('activation', 0)}

    # Print all discovered for documentation
    print("\n  All node IDs (first 30):")
    for i, (label, info) in enumerate(list(nid_map.items())[:30]):
        print(f"    [{info['type']}] {label} -> {info['node_id']} (activation={info['activation']:.3f})")

    # Map to target files
    target_map = {
        'chat_handler.py': None,
        'config.py': None,
        'spawner.py': None,
        'main.py': None,
        'stormender_v2.py': None,
    }

    for target_file in target_map.keys():
        base = target_file
        # Look for file-level node
        if base in nid_map:
            target_map[target_file] = nid_map[base]['node_id']
        else:
            # Try file:: prefix
            fid = f'file::{base}'
            for label, info in nid_map.items():
                if info['node_id'] == fid:
                    target_map[target_file] = fid
                    break

    print(f"\n  Target file -> node_id map:")
    for f, nid in target_map.items():
        print(f"    {f} -> {nid}")

    # Also try impact to get node IDs for files we need
    # impact might give us what we need
    print("\n[IMPACT CHECK] Testing impact on chat_handler.py to verify node IDs...")
    impact_result = m.tool('m1nd.impact', {
        'agent_id': 'agent4',
        'changed_node': 'file::chat_handler.py',
        'depth': 2
    })
    print(f"  Impact result keys: {list(impact_result.keys())}")
    impact_str = json.dumps(impact_result, indent=2, default=str)
    if len(impact_str) > 2000:
        impact_str = impact_str[:2000] + '...'
    print(impact_str)

    # ── PREDICT tests ─────────────────────────────────────────
    print("\n\n" + "#"*60)
    print("# PREDICT TESTS")
    print("#"*60)

    predict_targets = [
        ('chat_handler.py', 'file::chat_handler.py'),
        ('config.py', 'file::config.py'),
        ('spawner.py', 'file::spawner.py'),
        ('main.py', 'file::main.py'),
        ('stormender_v2.py', 'file::stormender_v2.py'),
    ]

    results['predict'] = []
    for fname, node_id in predict_targets:
        print(f"\n[PREDICT] {fname} (node_id={node_id})")
        r = m.tool('m1nd.predict', {
            'agent_id': 'agent4',
            'changed_node': node_id,
            'include_velocity': True,
            'top_k': 10
        })
        elapsed = r.get('_elapsed', 0)
        print(f"  elapsed: {elapsed:.3f}s")
        print(f"  keys: {[k for k in r.keys() if not k.startswith('_')]}")
        # Print predictions
        preds = r.get('predictions', r.get('co_change_predictions', r.get('results', [])))
        print(f"  predictions count: {len(preds) if isinstance(preds, list) else '?'}")
        if isinstance(preds, list):
            for p in preds[:5]:
                pfile = p.get('label', p.get('file_path', p.get('node_id', '?')))
                prob = p.get('probability', p.get('score', p.get('confidence', '?')))
                vel = p.get('velocity', '?')
                print(f"    -> {pfile} prob={prob} vel={vel}")

        # Full result
        r_str = json.dumps({k: v for k, v in r.items() if k != '_elapsed'}, indent=2, default=str)
        if len(r_str) > 3000:
            r_str = r_str[:3000] + '...'
        print(f"\n  FULL:\n{r_str}")

        results['predict'].append({
            'file': fname,
            'node_id': node_id,
            'elapsed': elapsed,
            'result': {k: v for k, v in r.items() if k != '_elapsed'}
        })

    # ── VALIDATE_PLAN: test with shorter relative paths ───────
    print("\n\n" + "#"*60)
    print("# VALIDATE_PLAN — test with shorter file paths")
    print("#"*60)

    # Try paths matching what ingest sees: just the filename
    plan_cases = [
        {
            'name': 'Plan A: single file (short path)',
            'actions': [
                {
                    'action_type': 'modify',
                    'file_path': 'chat_handler.py',
                    'description': 'Increase deep-work escalation threshold'
                }
            ]
        },
        {
            'name': 'Plan B: 3 files (short paths)',
            'actions': [
                {'action_type': 'modify', 'file_path': 'opencode_engine.py',
                 'description': 'Add exponential backoff retry on 429'},
                {'action_type': 'modify', 'file_path': 'cli_adapters.py',
                 'description': 'Add rate-limit detection in CLIAdapter base'},
                {'action_type': 'modify', 'file_path': 'config.py',
                 'description': 'Add RATE_LIMIT_MAX_RETRIES constant'}
            ]
        },
        {
            'name': 'Plan C: 7 actions cross-cutting (short paths)',
            'actions': [
                {'action_type': 'modify', 'file_path': 'whatsapp_manager.py',
                 'description': 'Add broadcast_to_group() method'},
                {'action_type': 'modify', 'file_path': 'whatsapp_routes.py',
                 'description': 'Add POST /api/whatsapp/broadcast endpoint'},
                {'action_type': 'modify', 'file_path': 'whatsapp_models.py',
                 'description': 'Add BroadcastRequest and BroadcastResult Pydantic models'},
                {'action_type': 'modify', 'file_path': 'whatsapp_store.py',
                 'description': 'Add group_participants() query and broadcast_log table'},
                {'action_type': 'create', 'file_path': 'whatsapp_broadcast.py',
                 'description': 'New module: broadcast engine with rate limiting'},
                {'action_type': 'test', 'file_path': 'tests/test_whatsapp_broadcast.py',
                 'description': 'Unit tests for broadcast engine'},
                {'action_type': 'modify', 'file_path': 'main.py',
                 'description': 'Register whatsapp_broadcast router'}
            ]
        }
    ]

    results['validate_plan_v2'] = []
    for case in plan_cases:
        print(f"\n[VALIDATE_PLAN] {case['name']}")
        r = m.tool('m1nd.validate_plan', {
            'agent_id': 'agent4',
            'actions': case['actions'],
            'include_risk_score': True,
            'include_test_impact': True
        })
        elapsed = r.get('_elapsed', 0)
        print(f"  elapsed: {elapsed:.3f}s")
        r_str = json.dumps({k: v for k, v in r.items() if k != '_elapsed'}, indent=2, default=str)
        if len(r_str) > 3000:
            r_str = r_str[:3000] + '...'
        print(r_str)
        results['validate_plan_v2'].append({
            'test_name': case['name'],
            'elapsed': elapsed,
            'result': {k: v for k, v in r.items() if k != '_elapsed'}
        })

    m.close()

    out = '/Users/cosmophonix/clawd/roomanizer-os/mcp/m1nd/agent4_predict_results.json'
    with open(out, 'w') as f:
        json.dump(results, f, indent=2, default=str)
    print(f"\n[SAVED] {out}")
    return results


if __name__ == '__main__':
    main()
