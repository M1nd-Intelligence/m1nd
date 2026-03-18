#!/usr/bin/env python3
"""
AGENT 2: Semantic Search + Pattern Scanning Test
Tests m1nd seek and scan tools against the roomanizer-os backend
"""
import json
import subprocess
import os
import tempfile
import time
import select
import sys
import traceback
from pathlib import Path

class M1nd:
    def __init__(self):
        self.w = tempfile.mkdtemp(prefix='m1nd_agent2_')
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
        # Initialize MCP
        self._call('initialize', {
            'protocolVersion': '2024-11-05',
            'capabilities': {},
            'clientInfo': {'name': 'agent2-test', 'version': '1.0'}
        })

    def _call(self, method, params=None, timeout=120):
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
        r = self._call('tools/call', {'name': name, 'arguments': args or {}}, timeout=timeout)
        if 'error' in r and isinstance(r['error'], str):
            return {'_error': r['error']}
        content = r.get('result', {}).get('content', [{}])
        text = content[0].get('text', '') if content else ''
        try:
            return json.loads(text)
        except:
            return {'_raw': text[:5000]}

    def close(self):
        self.p.stdin.close()
        self.p.wait(timeout=10)


def fmt_json(obj, indent=2):
    return json.dumps(obj, indent=indent, default=str)


def run_grep(pattern, path, flags='-rn', extra=''):
    """Run grep and return results"""
    cmd = f"grep {flags} '{pattern}' {path} {extra} 2>/dev/null | head -30"
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    lines = [l for l in result.stdout.strip().split('\n') if l]
    return lines


def count_grep(pattern, path, flags='-rn'):
    """Count grep matches"""
    cmd = f"grep {flags} '{pattern}' {path} 2>/dev/null | wc -l"
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    try:
        return int(result.stdout.strip())
    except:
        return 0


def main():
    print("=" * 80)
    print("AGENT 2: M1ND SEMANTIC SEARCH + PATTERN SCANNING TEST")
    print("=" * 80)

    BACKEND = '/Users/cosmophonix/clawd/roomanizer-os/backend'
    results = {}

    # ── INIT ──────────────────────────────────────────────────────────────────
    print("\n[1/6] Initializing m1nd...")
    m = M1nd()
    print("  m1nd process started")

    # ── HEALTH CHECK ──────────────────────────────────────────────────────────
    print("\n[2/6] Health check...")
    t0 = time.time()
    health = m.tool('m1nd.health', {'agent_id': 'agent2-test'})
    health_ms = int((time.time() - t0) * 1000)
    print(f"  health response ({health_ms}ms): {fmt_json(health)[:500]}")
    results['health'] = {'response': health, 'latency_ms': health_ms}

    # ── INGEST ────────────────────────────────────────────────────────────────
    print(f"\n[3/6] Ingesting backend: {BACKEND}")
    t0 = time.time()
    ingest = m.tool('m1nd.ingest', {
        'agent_id': 'agent2-test',
        'path': BACKEND,
        'incremental': False
    }, timeout=300)
    ingest_ms = int((time.time() - t0) * 1000)
    print(f"  ingest response ({ingest_ms}ms):")
    print(f"  {fmt_json(ingest)[:800]}")
    results['ingest'] = {'response': ingest, 'latency_ms': ingest_ms}

    # ── SEEK TESTS ────────────────────────────────────────────────────────────
    print("\n[4/6] Testing m1nd.seek...")

    seek_queries = [
        {
            'query': 'authentication flow',
            'description': 'Concept search: find auth logic',
            'grep_equiv': r"auth|login|token|bearer|oauth",
            'grep_flags': '-rni'
        },
        {
            'query': 'where do we handle errors',
            'description': 'Intent search: error handling locations',
            'grep_equiv': r"except|error|exception|raise|HTTPException",
            'grep_flags': '-rn'
        },
        {
            'query': 'websocket handler disconnect',
            'description': 'Component search: WebSocket lifecycle',
            'grep_equiv': r"websocket|WebSocket|disconnect|on_disconnect",
            'grep_flags': '-rn'
        },
        {
            'query': 'rate limiting and backoff',
            'description': 'Concept search: rate limit handling',
            'grep_equiv': r"rate.limit|backoff|retry|throttle|429",
            'grep_flags': '-rni'
        },
        {
            'query': 'database connection pool cleanup',
            'description': 'Resource management search',
            'grep_equiv': r"pool|connection|cursor|close\(\)|cleanup",
            'grep_flags': '-rn'
        },
        {
            'query': 'agent spawning and lifecycle',
            'description': 'Architecture search: agent management',
            'grep_equiv': r"spawn|agent|launch|process|lifecycle",
            'grep_flags': '-rni'
        },
        {
            'query': 'stream parsing SSE events',
            'description': 'Protocol search: streaming events',
            'grep_equiv': r"stream|sse|event.stream|data:.*\n",
            'grep_flags': '-rn'
        },
    ]

    seek_results = []
    for i, q in enumerate(seek_queries):
        print(f"\n  seek[{i+1}]: '{q['query']}'")
        t0 = time.time()
        seek = m.tool('m1nd.seek', {
            'agent_id': 'agent2-test',
            'query': q['query'],
            'top_k': 10
        })
        seek_ms = int((time.time() - t0) * 1000)

        # Count grep equivalent
        grep_count = count_grep(q['grep_equiv'], BACKEND, q['grep_flags'])
        grep_sample = run_grep(q['grep_equiv'], BACKEND + '/*.py', q['grep_flags'])[:5]

        seek_data = {
            'query': q['query'],
            'description': q['description'],
            'seek_latency_ms': seek_ms,
            'seek_raw': seek,
            'grep_equiv_pattern': q['grep_equiv'],
            'grep_match_count': grep_count,
            'grep_sample': grep_sample
        }

        # Extract key seek results
        if '_error' in seek:
            print(f"    ERROR: {seek['_error']}")
            seek_data['seek_results'] = []
        elif '_raw' in seek:
            print(f"    RAW: {seek['_raw'][:400]}")
            seek_data['seek_results_raw'] = seek['_raw'][:2000]
        else:
            hits = seek.get('results', seek.get('hits', seek.get('matches', [])))
            if isinstance(seek, list):
                hits = seek
            elif isinstance(seek, dict):
                # Try various response shapes
                for key in ['results', 'hits', 'matches', 'nodes', 'items']:
                    if key in seek:
                        hits = seek[key]
                        break
                else:
                    hits = []

            seek_data['seek_results'] = hits
            seek_data['hit_count'] = len(hits)
            print(f"    hits={len(hits)}, grep_matches={grep_count}, latency={seek_ms}ms")
            for j, hit in enumerate(hits[:3]):
                if isinstance(hit, dict):
                    score = hit.get('score', hit.get('relevance', hit.get('similarity', '?')))
                    label = hit.get('label', hit.get('name', hit.get('id', '?')))
                    fp = hit.get('file_path', hit.get('path', hit.get('source', '')))
                    line = hit.get('line', hit.get('line_number', hit.get('start_line', '')))
                    print(f"      [{j+1}] score={score} | {label} | {fp}:{line}")

        seek_results.append(seek_data)

    results['seek'] = seek_results

    # ── WEBSOCKET GREP COMPARISON ─────────────────────────────────────────────
    print("\n[4b] Comparison: 'WebSocket disconnects' — seek vs grep")
    t0 = time.time()
    ws_seek = m.tool('m1nd.seek', {
        'agent_id': 'agent2-test',
        'query': 'websocket disconnect cleanup handler',
        'top_k': 10
    })
    ws_seek_ms = int((time.time() - t0) * 1000)

    ws_grep_exact = run_grep(r"disconnect", BACKEND + '/*.py', '-rn')
    ws_grep_ws = run_grep(r"WebSocket|websocket", BACKEND + '/*.py', '-rn')
    ws_grep_count = count_grep(r"disconnect\|WebSocket\|on_disconnect", BACKEND, '-rni')

    results['ws_comparison'] = {
        'seek_ms': ws_seek_ms,
        'seek_raw': ws_seek,
        'grep_disconnect_lines': ws_grep_exact[:10],
        'grep_websocket_lines': ws_grep_ws[:5],
        'grep_total_matches': ws_grep_count
    }
    print(f"  seek: {ws_seek_ms}ms | grep matches for disconnect/WebSocket: {ws_grep_count}")

    # ── SCAN TESTS ────────────────────────────────────────────────────────────
    print("\n[5/6] Testing m1nd.scan (all 8 patterns)...")

    scan_patterns = [
        {
            'pattern': 'error_handling',
            'description': 'Find error handling concerns',
            'grep_equiv': r"except\s*:\|bare except\|pass$",
            'grep_flags': '-rn'
        },
        {
            'pattern': 'resource_cleanup',
            'description': 'Find resource leaks',
            'grep_equiv': r"open(\|subprocess\|socket\|\.connect(",
            'grep_flags': '-rn'
        },
        {
            'pattern': 'concurrency',
            'description': 'Find concurrency issues',
            'grep_equiv': r"asyncio\|threading\|Lock\|Semaphore\|gather",
            'grep_flags': '-rn'
        },
        {
            'pattern': 'state_mutation',
            'description': 'Find unsafe state changes',
            'grep_equiv': r"global \|self\.\w* =\|_state\[",
            'grep_flags': '-rn'
        },
        {
            'pattern': 'auth_boundary',
            'description': 'Find auth boundary gaps',
            'grep_equiv': r"Depends\|@router\.\|HTTPBearer\|verify_token",
            'grep_flags': '-rn'
        },
        {
            'pattern': 'api_surface',
            'description': 'Find API surface issues',
            'grep_equiv': r"@router\.\|@app\.\|APIRouter",
            'grep_flags': '-rn'
        },
        {
            'pattern': 'test_coverage',
            'description': 'Find test coverage gaps',
            'grep_equiv': r"def test_\|@pytest\|unittest",
            'grep_flags': '-rn'
        },
        {
            'pattern': 'dependency_injection',
            'description': 'Find DI concerns',
            'grep_equiv': r"Depends(\|inject\|provider\|singleton",
            'grep_flags': '-rn'
        },
    ]

    scan_results = []
    for i, sp in enumerate(scan_patterns):
        print(f"\n  scan[{i+1}]: pattern='{sp['pattern']}'")
        t0 = time.time()
        scan = m.tool('m1nd.scan', {
            'agent_id': 'agent2-test',
            'pattern': sp['pattern']
        })
        scan_ms = int((time.time() - t0) * 1000)

        grep_count = count_grep(sp['grep_equiv'], BACKEND, sp['grep_flags'])

        scan_data = {
            'pattern': sp['pattern'],
            'description': sp['description'],
            'scan_latency_ms': scan_ms,
            'scan_raw': scan,
            'grep_equiv_pattern': sp['grep_equiv'],
            'grep_match_count': grep_count
        }

        if '_error' in scan:
            print(f"    ERROR: {scan['_error']}")
        elif '_raw' in scan:
            print(f"    RAW ({scan_ms}ms): {scan['_raw'][:400]}")
            scan_data['scan_raw_text'] = scan['_raw'][:3000]
        else:
            # Extract findings
            findings = []
            for key in ['findings', 'issues', 'results', 'matches', 'items']:
                if key in scan:
                    findings = scan[key]
                    break
            if isinstance(scan, list):
                findings = scan

            scan_data['findings'] = findings
            scan_data['finding_count'] = len(findings)
            print(f"    findings={len(findings)}, grep_matches={grep_count}, latency={scan_ms}ms")
            for j, f in enumerate(findings[:3]):
                if isinstance(f, dict):
                    sev = f.get('severity', f.get('level', '?'))
                    msg = f.get('message', f.get('description', f.get('text', '?')))[:100]
                    fp = f.get('file_path', f.get('path', f.get('file', '')))
                    line = f.get('line', f.get('line_number', ''))
                    print(f"      [{j+1}] sev={sev} | {fp}:{line} | {msg}")
                else:
                    print(f"      [{j+1}] {str(f)[:120]}")

        scan_results.append(scan_data)

    results['scan'] = scan_results

    # ── CONCURRENCY COMPARISON ────────────────────────────────────────────────
    print("\n[5b] Comparison: 'concurrency issues' — scan vs grep")
    conc_grep_patterns = [
        r"asyncio\.",
        r"threading\.",
        r"Lock()\|RLock()\|Semaphore",
        r"await.*gather",
        r"global ",
    ]
    conc_grep_counts = {}
    for pat in conc_grep_patterns:
        conc_grep_counts[pat] = count_grep(pat, BACKEND, '-rn')
        print(f"  grep '{pat}': {conc_grep_counts[pat]} matches")

    # Find the concurrency scan result
    conc_scan = next((s for s in scan_results if s['pattern'] == 'concurrency'), {})
    results['concurrency_comparison'] = {
        'scan_findings': conc_scan.get('finding_count', conc_scan.get('scan_raw', 'N/A')),
        'grep_counts_by_pattern': conc_grep_counts,
        'grep_total': sum(conc_grep_counts.values())
    }

    # ── RESOURCE CLEANUP COMPARISON ───────────────────────────────────────────
    print("\n[5c] Comparison: 'resource cleanup' — scan vs grep")
    res_cleanup_grep = [
        r"open(",
        r"subprocess\.Popen\|subprocess\.run",
        r"socket\.",
        r"asyncio\.create_task",
        r"\.close()",
        r"finally:",
        r"with open\|with asyncio",
    ]
    res_grep_counts = {}
    for pat in res_cleanup_grep:
        res_grep_counts[pat] = count_grep(pat, BACKEND, '-rn')
        print(f"  grep '{pat}': {res_grep_counts[pat]} matches")

    resource_scan = next((s for s in scan_results if s['pattern'] == 'resource_cleanup'), {})
    results['resource_comparison'] = {
        'scan_findings': resource_scan.get('finding_count', resource_scan.get('scan_raw', 'N/A')),
        'grep_counts': res_grep_counts,
        'grep_total': sum(res_grep_counts.values())
    }

    # ── AVAILABLE TOOLS LIST ──────────────────────────────────────────────────
    print("\n[6/6] Listing available tools...")
    tools_resp = m._call('tools/list', {})
    all_tools = [t.get('name', '') for t in tools_resp.get('result', {}).get('tools', [])]
    print(f"  Available tools ({len(all_tools)}): {all_tools}")
    results['available_tools'] = all_tools

    m.close()

    # ── SAVE RAW RESULTS ─────────────────────────────────────────────────────
    raw_path = '/Users/cosmophonix/clawd/roomanizer-os/docs/m1nd/agent2_raw_results.json'
    with open(raw_path, 'w') as f:
        json.dump(results, f, indent=2, default=str)
    print(f"\nRaw results saved to: {raw_path}")

    return results


if __name__ == '__main__':
    try:
        r = main()
        print("\nTest complete.")
    except Exception as e:
        print(f"\nFATAL ERROR: {e}")
        traceback.print_exc()
        sys.exit(1)
