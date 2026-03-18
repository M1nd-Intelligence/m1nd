# Reddit Launch Posts

---

## POST 1 — r/rust

**Title:** m1nd: code graph in Rust with Hebbian plasticity + 52 MCP tools. zero LLM calls.

built this because i kept hitting the wall where grep finds text but can't answer structural questions.

three crates:
- m1nd-core: CSR graph, spreading activation, Hebbian LTP/LTD, hypothesis engine, antibody system, epidemic, tremor, trust
- m1nd-ingest: 28 languages via tree-sitter, git enrichment, memory adapter, incremental diff
- m1nd-mcp: 52 tool handlers, JSON-RPC over stdio, HTTP server + GUI

benchmarks from Criterion (real hardware, not cherry-picked):
- activate 1K nodes: 1.36µs
- impact depth=3: 543ns
- lock.diff: 0.08µs
- antibody_scan 50 patterns: 2.68ms
- epidemic SIR: 110µs

the graph learns. you fire a query, results come back ranked. mark them as "correct" or "wrong" and edge weights update via Hebbian LTP/LTD. 740 edges strengthened in one session. the graph got noticeably better at my codebase by day 3.

tested on a 52K line python/fastapi backend: 39 bugs in one session. 8 required structural analysis, grep would've never found them.

the part i'm least sure about: the noise cancellation layer (called XLR internally). it dampens low-activation paths during spreading activation so you don't get garbage returns on unrelated nodes. works well empirically but i don't have strong theoretical justification for the dampening coefficient. if anyone has experience with this kind of signal propagation i'd actually love to compare notes.

~8MB binary, MIT, no API keys. published to crates.io as m1nd-core, m1nd-ingest, m1nd-mcp.

github.com/maxkle1nz/m1nd

---

## POST 2 — r/LocalLLaMA

**Title:** a, 

local LLMs are great but they navigate your codebase blind. they grep, they embed, they lose context across files. m1nd is an MCP server that gives them a structural map of the code and lets them ask real questions.

tested on a 52K line codebase. the agent ran 46 m1nd queries instead of an estimated 210 grep operations, finished in ~3 seconds total query time versus 35 minutes estimated for a grep-based equivalent investigation. found 39 bugs, 28 confirmed.

the interesting part for local setups: m1nd runs as a local binary, no API calls, no tokens, no internet required. your local agent queries it via MCP tools like any other tool call. the graph builds from your codebase and persists between sessions.

what it does:
- spreading activation across code structure (not text matching)
- hypothesis testing: "does A depend on B at runtime?" → Bayesian verdict on graph paths, 89% accuracy
- counterfactual: "what breaks if i remove this module?" → full cascade in 3ms
- missing detection: specs with no implementation, implementations with no tests
- antibody system: remembers bug shapes from past sessions, scans new ingests for recurrence
- trail system: save investigation state, resume days later, merge two agents working the same bug

works with ollama + opencode, lm studio + continue, anything that supports MCP.

28 languages supported, ~8MB binary, MIT.

github.com/maxkle1nz/m1nd

happy to answer questions about the MCP integration or how to set it up with your local stack.

---

## POST 3 — r/programming

**Title:** I built a code graph that learns from how you use it. 39 bugs found in one session, 8 invisible to grep.

been working on a production backend (52K lines, python/fastapi) and kept running into the same situation: i know something is wrong, i can feel the dependency structure is off, but text search gives me nothing useful.

built m1nd to answer questions grep can't. not "find files containing X" but "does A actually depend on B at runtime?", "what's the blast radius if i change this?", "where are the bugs hiding that haven't surfaced yet?"

the thing that surprised me most in practice: the graph learns. you confirm which results were useful after each query. Hebbian plasticity updates edge weights, so the graph evolves to match how you actually think about the code. by session 3 it was surfacing connections i would have missed.

ran a live audit session and found 39 bugs. 28 confirmed fixed. 8 of those 28 were invisible to any text search, they only appeared through structural analysis. the hypothesis engine ran at 89% accuracy across 10 live claims, including confirming a session_pool leak at 99% confidence and correctly rejecting a circular dependency at 1%.

five unusual capabilities beyond standard graph traversal:
1. antibody system: stores patterns of past bugs. every time you ingest code it scans for the same shapes.
2. epidemic engine: SIR propagation model predicts which modules are likely harboring undiscovered bugs, based on structural proximity to known infected nodes
3. tremor detection: tracks change acceleration (d²churn/dt²) not just churn. acceleration precedes bugs better than raw frequency
4. trust ledger: per-module risk score built from defect history
5. trail save/resume: mid-investigation snapshots. come back 3 days later from the exact same cognitive position

it's a rust binary (~8MB), 52 MCP tools, MIT license, works with claude code, cursor, windsurf, whatever MCP client you use.

honest limitations: no neural semantic search (trigram only in v1), not optimized above 400K files, no dataflow/taint analysis, not real-time on every save.

github.com/maxkle1nz/m1nd

curious what approaches others have used for structural code analysis, especially anything that handles co-change patterns. most of what i found was static only.
