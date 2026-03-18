#!/usr/bin/env python3
"""
Probe actual tool schemas for ingest, trace, validate_plan, predict, warmup
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
        self.w = tempfile.mkdtemp(prefix='m1nd_schema_')
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

    def close(self):
        try:
            self.p.stdin.close()
            self.p.wait(timeout=10)
        except Exception:
            self.p.kill()


def main():
    m = M1nd()
    time.sleep(2)

    # Get full tools list with schemas
    r = m._call('tools/list', {}, timeout=30)
    tools = r.get('result', {}).get('tools', [])

    # Print schemas for our target tools
    target_names = {'m1nd.ingest', 'm1nd.trace', 'm1nd.validate_plan', 'm1nd.predict', 'm1nd.warmup', 'm1nd.health'}
    for t in tools:
        if t['name'] in target_names:
            print(f"\n{'='*60}")
            print(f"TOOL: {t['name']}")
            print(f"Description: {t.get('description', '')[:200]}")
            print(f"Input schema:")
            schema = t.get('inputSchema', {})
            props = schema.get('properties', {})
            required = schema.get('required', [])
            for pname, pdef in props.items():
                req_marker = '*' if pname in required else ' '
                print(f"  {req_marker} {pname}: {pdef.get('type', '?')} — {pdef.get('description', '')[:100]}")
            print(f"  Required: {required}")
            print(f"  Full schema: {json.dumps(schema, indent=2)}")

    m.close()


if __name__ == '__main__':
    main()
