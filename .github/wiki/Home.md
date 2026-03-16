# m1nd Wiki

<p align="center">
  <strong>The adaptive code graph. It learns.</strong>
</p>

Neuro-symbolic connectome engine with Hebbian plasticity, spreading activation, and 61 MCP tools. Built in Rust for AI agents.

**39 bugs found in one audit session · 89% hypothesis accuracy · 12/12 verification accuracy · Zero LLM tokens**

---

## What is m1nd?

m1nd doesn't search your codebase — it *activates* it. Fire a query into the graph and watch signal propagate across structural, semantic, temporal, and causal dimensions. Noise cancels out. Relevant connections amplify. And the graph *learns* from every interaction via Hebbian plasticity.

```
335 files → 9,767 nodes → 26,557 edges in 0.91 seconds.
Then: activate in 31ms. impact in 5ms. trace in 3.5ms. learn in <1ms.
```

It is not a search engine. It is not a RAG pipeline. It is not a static analysis tool. It is a **living graph** that evolves with your codebase and your team's understanding of it.

---

## Key Numbers

| Metric | Value |
|--------|-------|
| Tools | **61 MCP tools** |
| Languages supported | **27+** (5 built-in + tier1 + tier2 via tree-sitter) |
| Bugs found in one session | **39** (production Python backend, 52K lines) |
| Bugs invisible to grep | **8 of 28 (28.5%)** — required structural analysis |
| Hypothesis accuracy | **89%** over 10 live claims |
| Verification accuracy | **12/12 (100%)** — post-write verification catches silent failures |
| LLM tokens per query | **0** — pure Rust, local binary |
| `activate` speed | **1.36µs** (bench) · 31–77ms (production) |
| `impact` speed | **543ns** (bench) · 5–52ms (production) |
| `lock.diff` speed | **0.08µs** — essentially free change detection |
| Binary size | ~8MB |
| Memory for 10K nodes | ~2MB |

---

## Why It Exists

AI agents are powerful reasoners but terrible navigators. They can analyze what you show them, but they can't *find* what matters in a codebase of 10,000 files.

| Approach | Why It Fails |
|----------|-------------|
| Full-text search | Finds what you *said*, not what you *meant* |
| RAG | Each retrieval is amnesiac. No relationships between results. |
| Static analysis | Frozen snapshot. Can't answer "what if?". Can't learn. |
| Knowledge graphs | Manual curation. Only returns what was explicitly encoded. |

**m1nd fires signal into a weighted graph and watches where the energy goes.** The signal propagates, reflects, interferes, and decays according to physics-inspired rules. The graph learns which paths matter.

---

## The 8 Differentiators

### 1. The graph learns — Hebbian Plasticity
Confirm results as useful → edge weights strengthen along those paths. Mark results as wrong → they weaken. Over time, the graph evolves to match how *your* team thinks about *your* codebase. No other code intelligence tool does this.

### 2. The graph cancels noise — XLR Differential Processing
Borrowed from professional audio engineering. Transmits signal on two inverted channels and subtracts common-mode noise at the receiver. Activation queries return signal, not the noise that grep drowns you in.

### 3. The graph remembers investigations — Trail System
Save mid-investigation state (hypotheses, graph weights, open questions). End the session. Resume days later from the exact same cognitive position. Two agents investigating the same bug? Merge their trails — conflict detection on shared nodes included.

### 4. The graph tests claims — Hypothesis Engine
"Does the worker pool have a hidden runtime dependency on the WhatsApp manager?" — m1nd explores 25,015 paths in 58ms and returns a Bayesian confidence verdict. **89% accuracy validated on a live production codebase.**

### 5. The graph simulates alternatives — Counterfactual Engine
"What breaks if I delete `spawner.py`?" In 3ms: 4,189 affected nodes, cascade explosion at depth 3. Numbers impossible to derive from text search.

### 6. The graph ingests memory — Memory Adapter
Pass `adapter: "memory"` to ingest any `.md`, `.txt`, or `.markdown` file as a typed graph — then merge it with your code graph. One graph, one query across code and documentation.

### 7. The graph detects pre-failure bugs — Superpowers Extended
Antibody system (bug immune memory), Epidemic engine (SIR propagation prediction), Tremor detection (change acceleration = earthquake precursor), Trust ledger (actuarial risk scores), Layer detection (auto-detect architectural violations).

### 8. The graph verifies its own writes — Post-Write Verification
`apply_batch` with `verify: true` re-reads each file immediately after writing and compares it against the intended content. **12/12 accuracy** in production testing — catches silent failures (encoding issues, partial writes, permission races) before they propagate downstream. See [API Reference — apply_batch](API-Reference#surgical-4-tools) for details.

---

## 61 Tools — Quick Reference

| Category | Tools | Count |
|----------|-------|-------|
| [Foundation](API-Reference#foundation-13-tools) | ingest, activate, impact, why, learn, drift, health, seek, scan, timeline, diverge, warmup, federate | 13 |
| [Perspective Navigation](API-Reference#perspective-navigation-12-tools) | perspective.start/routes/follow/back/peek/inspect/suggest/affinity/branch/compare/list/close | 12 |
| [Lock System](API-Reference#lock-system-5-tools) | lock.create, lock.watch, lock.diff, lock.rebase, lock.release | 5 |
| [Superpowers](API-Reference#superpowers-13-tools) | hypothesize, counterfactual, missing, resonate, fingerprint, trace, validate_plan, predict, trail.save/resume/merge/list, differential | 13 |
| [Superpowers Extended](API-Reference#superpowers-extended-9-tools) | antibody_scan, antibody_list, antibody_create, flow_simulate, epidemic, tremor, trust, layers, layer_inspect | 9 |
| [Surgical](API-Reference#surgical-4-tools) | surgical_context, apply, surgical_context_v2, apply_batch | 4 |
| [v0.4.0 — Search & Efficiency](API-Reference#v040-search--efficiency-5-tools) | search, help, panoramic, savings, report | 5 |
| [v0.5.0 — Verified Writes & File Tools](API-Reference#v050-verified-writes--file-tools) | apply_batch verify, view, glob | 3 |

---

## Common Workflows

**Bug hunt:**
```
hypothesize("worker pool leaks on task cancel")  → 99% confidence, 3 bugs
missing("cancellation cleanup timeout")          → 2 structural holes
flow_simulate(seeds=["worker_pool.py"])          → 223 turbulence points
trace(stacktrace_text)                           → suspects ranked by suspiciousness
```

**Pre-deploy gate:**
```
antibody_scan(scope="changed")    → known bug shapes
validate_plan(files=changed)      → blast radius + gaps
epidemic(infected=[changed])      → infection spread prediction
```

**Architecture audit:**
```
layers()                          → auto-detected layers + violations
layer_inspect(level=2)            → drill into a specific layer
counterfactual(node_ids=[...])    → simulate removal cascade
```

**Onboarding:**
```
activate("how does auth work")    → graph lights up the path
layers()                          → architecture overview
perspective.start(query="auth")   → guided navigation
perspective.follow                → drill in
```

**v0.4.0 — Search, efficiency, panoramic analysis, and verified writes:**
```
search(query="async def.*cancel", mode="regex")  → grep replacement with graph context
panoramic()                                       → full module risk map ranked by blast radius
savings()                                         → session + global token economy report
report()                                          → query log + session statistics
help(tool_name="activate")                        → runtime-discoverable docs with visual identity
apply_batch(files=[...], verify=true)             → write multiple files + verify each one landed
```

---

## Quick Links

- [Getting Started](Getting-Started) — installation, first query, Claude Code setup
- [API Reference](API-Reference) — all 61 tools with schemas, examples, benchmark times
- [EXAMPLES.md](../EXAMPLES.md) — raw examples from a production codebase
- [README.md](../README.md) — full project overview

---

## Works With Any MCP Client

Claude Code · Cursor · Windsurf · GitHub Copilot · Zed · Cline · Roo Code · Continue · OpenCode · Amazon Q

---

## On crates.io

m1nd v0.5.0 is published on [crates.io](https://crates.io/crates/m1nd-core). Add as a library dependency or build the binary from source — both paths are fully supported.

---

## Architecture — 3 Crates

```
m1nd/
  m1nd-core/     Graph engine, plasticity, spreading activation, hypothesis engine
                 antibody, flow, epidemic, tremor, trust, layer detection, domain config
  m1nd-ingest/   Language extractors (27+ languages), memory adapter, JSON adapter,
                 git enrichment, cross-file resolver, incremental diff
  m1nd-mcp/      MCP server, 61 tool handlers, JSON-RPC over stdio
```

Pure Rust. No runtime dependencies. No LLM calls. No API keys.

---

*Created by [Max Elias Kleinschmidt](https://github.com/cosmophonix) · The graph must learn.*
