#!/usr/bin/env python3
"""
file-watcher.py — triggered by launchd WatchPaths when files change.
Detects which namespace changed and runs smart-ingest for that namespace only.
"""
import sys
import os
import subprocess
import time

CLAWD = "/Users/cosmophonix/clawd"

PATH_TO_NAMESPACE = {
    f"{CLAWD}/crush":        "crush",
    f"{CLAWD}/roomanizer-os": "roomanizer",
    f"{CLAWD}/openclaw":     "openclaw",
    f"{CLAWD}/RESON":        "reson",
    f"{CLAWD}/kosmo-only":   "kosmo",
    f"{CLAWD}/jimi-cli-ui":  "jimi-cli",
    f"{CLAWD}/crush-work":   "crush-work",
}

SMART_INGEST = "/Users/cosmophonix/.m1nd/smart-ingest.py"
LOCK_FILE = "/tmp/m1nd-watcher.lock"
DEBOUNCE_SECS = 3

def get_namespace(changed_path: str) -> str:
    for prefix, ns in PATH_TO_NAMESPACE.items():
        if changed_path.startswith(prefix):
            return ns
    return None


def debounce():
    """Avoid running multiple times for a burst of file changes."""
    if os.path.exists(LOCK_FILE):
        try:
            age = time.time() - os.path.getmtime(LOCK_FILE)
            if age < DEBOUNCE_SECS:
                return False
        except Exception:
            pass
    with open(LOCK_FILE, "w") as f:
        f.write(str(time.time()))
    return True


def main():
    if not debounce():
        sys.exit(0)

    # launchd passes changed paths as env var or we just do all namespaces incremental
    # Since WatchPaths doesn't tell us WHICH file changed, we do incremental for all
    print("[m1nd-watcher] files changed, running incremental ingest...")
    
    env = os.environ.copy()
    env["M1ND_PORT"] = "1337"
    
    result = subprocess.run(
        ["python3", SMART_INGEST, "--incremental"],
        env=env,
        capture_output=True,
        text=True,
        timeout=120,
    )
    
    if result.returncode == 0:
        print("[m1nd-watcher] ✅ incremental ingest ok")
        print(result.stdout)
    else:
        print("[m1nd-watcher] ❌", result.stderr)


if __name__ == "__main__":
    main()
