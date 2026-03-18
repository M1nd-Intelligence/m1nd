---
title: I found 8 bugs grep can never find — they have no keyword, only structure
published: true
tags: rust, ai, devtools, opensource
---

I was three months into debugging a 52K line Python/FastAPI backend when I hit the same wall for the fifth time.

The test says something is wrong. The logs agree. But I open the codebase and there's nothing to search for.

No suspicious keyword. No obvious import chain. No text pattern that would lead me there. The bug doesn't exist as a string — it exists as a *relationship between things*. A TOCTOU race in a dict read-check-modify. An orphan process that only leaks when a specific cancellation sequence fires. A two-hop dependency that grep says doesn't exist because there's no direct import.

I was spending 35 minutes per investigation doing ~210 grep operations, reading 228 files, burning 193K tokens just to build a mental map of what depends on what. Then I'd answer my structural question and throw that map away.

So I built something to keep the map.

---

## The insight that changed how I think about this

Code is a graph. We all know this abstractly, but we treat it like text in practice. grep, ripgrep, AST search, semantic search — they're all fundamentally scanning sequences. They're good at "find this pattern where it appears."

What they can't do is answer the absence question: "what *should* be here that isn't?" A missing lock on a concurrent dict access has no text signature. A module that was supposed to implement an interface but doesn't — no text signature. A guard that was written for 4 out of 5 code paths — you'd have to read all 5 paths to notice the gap.

For that, you need a graph you can reason over. Not just "here's a visualization of the AST" — you need a graph that can *propagate* information, test hypotheses, and notice structural holes.

---

## What I built

I wrote a Rust binary called m1nd that:

1. Parses your codebase into a weighted directed graph (28 languages via tree-sitter)
2. Stores it in CSR format with 4-dimensional edge weights (structural, semantic, temporal, causal)
3. Answers structural questions via spreading activation — query hits a node, signal propagates across the graph, relevant connections amplify, noise cancels

335 files parse in 910ms. Then:

```
activate in 31ms
impact in 5ms
trace in 3.5ms
```

It works as an MCP server so it plugs into Claude Code, Cursor, Windsurf, Zed — anything that supports MCP.

---

## The 8 bugs grep couldn't find

Here's the actual workflow from the audit session that produced these findings. Every query below is real, the latency is real, the results are real.

**Bug 1: TOCTOU race on worker pool `is_alive` check**

```jsonc
{"method":"tools/call","params":{"name":"m1nd.missing","arguments":{
  "query": "worker pool session reuse timeout cleanup",
  "agent_id": "audit"
}}}
// Returns: is_alive TOCTOU hole at worker_pool.py (score 1.12)
// "hole detected: dict read-check-modify without lock in concurrency context"
// Time: 67ms. LLM tokens: 0.
```

The bug is the *absence* of a lock. There is no text to grep for. `missing()` detects structural holes by looking for patterns that are present in similar code contexts but absent here.

**Bug 2: Session leak on storm cancellation**

```jsonc
{"method":"tools/call","params":{"name":"m1nd.hypothesize","arguments":{
  "claim": "session_pool leaks on storm cancel",
  "depth": 8,
  "agent_id": "audit"
}}}
// Returns: 99% confidence, 25,015 paths analyzed
// Evidence: 3 supporting groups found via path analysis
// Time: 58ms.
```

This is graph path reasoning. m1nd explored 25,000 paths from `session_pool` to `storm_cancel` and found three independent evidence groups pointing to a real leak. grep says "there's no direct import" — and that's correct. There isn't. The dependency is two hops through `process_manager.cancel()`. Confirmed real. Fixed.

**Bug 3-5: Concurrent state corruption cluster**

```jsonc
{"method":"tools/call","params":{"name":"m1nd.resonate","arguments":{
  "query": "stormender phase timeout cancel",
  "harmonics": 5,
  "agent_id": "audit"
}}}
// Returns: 9 "cancel" nodes at amplitude 1.4 — not converging
// Interpretation: lifecycle.py and control.py both cancel phases -> TOCTOU race
// Time: 52ms.
```

`resonate()` runs a standing wave analysis — fire a signal into the graph and look for nodes that keep amplifying instead of settling. Nine cancel-related nodes were resonating at 1.4 instead of converging. That pattern means multiple writers to shared state. Three bugs in the same cluster.

**Bugs 6-8: Orphan process cascade**

```jsonc
{"method":"tools/call","params":{"name":"m1nd.flow_simulate","arguments":{
  "entry_node": "storm_launcher",
  "particles": 4,
  "agent_id": "audit"
}}}
// Returns: 2 of 4 particles deadlock at process_manager junction
// Time: 552µs.
```

Flow simulation models concurrent execution. Two of four simulated execution paths deadlock. The deadlock path leads to processes that launch but whose cleanup isn't reachable from the normal cancellation flow. Three orphan process bugs found this way.

---

## The comparison table (grounded in the audit)

The audit ran 46 m1nd queries total vs an estimated 210+ grep operations for the same coverage.

| What you're looking for | grep | m1nd |
|---|---|---|
| "find all uses of function X" | Yes | Yes |
| "does module A depend on B at runtime" | No (indirect deps invisible) | Yes |
| "what breaks if I remove this module" | No | Yes (`counterfactual`) |
| "is there a missing guard here" | No | Yes (`missing`) |
| "which modules are highest risk" | No | Yes (`trust`, `epidemic`) |
| "does this race condition exist" | No | Yes (`hypothesize`, `flow_simulate`) |
| "what changed since last session" | git diff | Yes + structural context (`drift`) |
| Cost per query | Free | Free |
| Tokens consumed | 0 | 0 |

The 8 structural bugs were invisible to text search because they fell into three categories:
- **Absence bugs**: a guard/lock/check that should exist but doesn't
- **Indirect dependency bugs**: A depends on B but only via C — no direct import
- **Convergence bugs**: multiple code paths that should coordinate but don't (detectable via resonance)

---

## The part I wasn't expecting: the graph learns

I added Hebbian plasticity mostly because I found it interesting from a theory standpoint. Tell the graph a result was useful, edge weights strengthen. Tell it a result was wrong, they weaken. Standard LTP/LTD from neuroscience applied to graph edge weights.

What I didn't expect: after three sessions, the graph had noticeably adapted to how I actually think about this codebase. The activation patterns that were useful kept getting stronger. The noise paths kept getting weaker. It started surfacing connections I'd confirmed weeks earlier without me asking.

```jsonc
{"method":"tools/call","params":{"name":"m1nd.learn","arguments":{
  "feedback": "correct",
  "node_ids": ["file::session_pool.py", "file::process_manager.py"],
  "agent_id": "audit",
  "context": "session_pool TOCTOU via process_manager.cancel"
}}}
// -> 740 edges strengthened via Hebbian LTP
// Time: <1ms.
```

I don't have strong theoretical justification for the exact dampening coefficient I chose. It works empirically. I'd genuinely welcome feedback from anyone who's done formal work on learning in graph structures.

---

## Setup (the short version)

```bash
git clone https://github.com/maxkle1nz/m1nd.git
cd m1nd && cargo build --release
./target/release/m1nd-mcp
```

Add to your MCP client config:

```json
{
  "mcpServers": {
    "m1nd": {
      "command": "/path/to/m1nd-mcp",
      "env": {
        "M1ND_GRAPH_SOURCE": "/tmp/m1nd-graph.json",
        "M1ND_PLASTICITY_STATE": "/tmp/m1nd-plasticity.json"
      }
    }
  }
}
```

Then in Claude Code (or Cursor, or any MCP client):

```jsonc
// Ingest your codebase
{"method":"tools/call","params":{"name":"m1nd.ingest","arguments":{
  "path": "/your/project",
  "agent_id": "dev"
}}}
// -> graph built. For 335 files: 9,767 nodes, 26,557 edges, 910ms.

// Ask a structural question
{"method":"tools/call","params":{"name":"m1nd.activate","arguments":{
  "query": "authentication session middleware",
  "agent_id": "dev"
}}}
// -> auth fires -> propagates to session, JWT, middleware, user model
// -> ghost edges reveal undocumented connections

// Tell it what was useful
{"method":"tools/call","params":{"name":"m1nd.learn","arguments":{
  "feedback": "correct",
  "node_ids": ["file::auth.py", "file::middleware.py"],
  "agent_id": "dev"
}}}
// -> edges strengthened. Next query is smarter.
```

MIT license, ~8MB binary, no API keys, no cloud, no LLM calls at query time.

Three crates on crates.io: `m1nd-core`, `m1nd-ingest`, `m1nd-mcp`.

Full repo: **github.com/maxkle1nz/m1nd**

---

## What it can't do (being honest)

- **Neural semantic search**: v1 uses trigram matching, not embeddings. "Find code that *means* auth but never uses the word" won't work yet.
- **Dataflow / taint analysis**: m1nd tracks structural and co-change relationships, not data propagation through variables. Use Semgrep or CodeQL for that.
- **Real-time indexing**: ingest is fast (910ms for 335 files) but not keystroke-fast. It's session-level intelligence. Your LSP handles keystroke feedback.
- **400K+ file repos**: the graph lives in memory. It works at that scale, but it wasn't optimized for it.

---

## Questions I'd genuinely like answers to

The three things I'm least confident about:

1. **Plasticity coefficient**: I chose 0.1 for the Hebbian learning rate based on what felt empirically right. Is there a more principled approach for sparse, heterogeneous graphs? I've read some GNN literature but it doesn't map cleanly.

2. **XLR noise cancellation**: I borrowed the concept from balanced audio (common-mode rejection). During spreading activation, I dampen paths where the forward and backward signals are out of phase, on the theory that they're noise. Works well. But I don't have a strong theoretical argument for why this should generalize.

3. **Hypothesis confidence calibration**: the 89% accuracy was across 10 live claims. That's not enough samples to trust the calibration. Has anyone done formal evaluation of Bayesian confidence scores over graph path analysis?

If you've done work in graph signal processing, static analysis, or formal program analysis — I'd actually love feedback on any of these. The structural detection stuff works. The learning side is the part where I know I'm operating on empirical intuition more than theory.

---

*repo: github.com/maxkle1nz/m1nd — issues and PRs welcome*
