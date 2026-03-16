# Roadmap

m1nd is early-stage and moving fast. This page documents what exists today, what's being built
next, and where the project is headed long-term.

---

## Shipped — v0.1.0

The first public release. 43 MCP tools across four categories.

**43 MCP tools across 4 categories:**
- Foundation (13 tools): ingest, activate, impact, why, learn, drift, health, seek, scan, timeline, diverge, warmup, federate
- Perspective Navigation (12 tools): start, routes, follow, back, peek, inspect, suggest, affinity, branch, compare, list, close
- Lock System (5 tools): create, watch, diff, rebase, release
- Superpowers (13 tools): hypothesize, counterfactual, missing, resonate, fingerprint, trace, validate_plan, predict, trail.save, trail.resume, trail.merge, trail.list, differential

**Three ingest adapters:** Code (27+ languages), Memory/Markdown, JSON.

---

## Shipped — v0.2.0

### What's New in v0.2

Everything on this page marked as "current" has been empirically validated in production use.
The numbers below come from a live audit session on 2026-03-14 against a ~52K line Python/FastAPI
codebase.

**Added in v0.2 — 6 Cross-Domain / Superpowers Extended tools:**
- `antibody_scan`, `antibody_create`, `antibody_list` — Immune memory: scan codebase for known bug patterns
- `flow_simulate` — Race condition detection via agent-based particle simulation
- `epidemic` — Bug spread prediction via SIR model (R₀ estimation, blast radius)
- `tremor` — Change acceleration detection (second-derivative of edge weight time series)
- `trust` — Actuarial per-module defect density (Bayesian risk scoring)
- `layers` + `layer_inspect` — Auto-detect architectural layers, count violations

**Added in v0.2 — HTTP server + embedded GUI:**
- `--serve` flag starts an axum HTTP server with REST API for all tools
- React UI embedded in the binary via `rust-embed` (no external assets)
- Server-Sent Events stream for real-time tool execution visibility
- `--stdio` dual-transport: stdio + HTTP simultaneously
- Cross-process SSE bridge via `--event-log` + `--watch-events`

**Total in v0.2: 52 MCP tools across 5 categories:**
- Foundation (13 tools): ingest, activate, impact, why, learn, drift, health, seek, scan, timeline, diverge, warmup, federate
- Perspective Navigation (12 tools): start, routes, follow, back, peek, inspect, suggest, affinity, branch, compare, list, close
- Lock System (5 tools): create, watch, diff, rebase, release
- Superpowers (13 tools): hypothesize, counterfactual, missing, resonate, fingerprint, trace, validate_plan, predict, trail.save, trail.resume, trail.merge, trail.list, differential
- Superpowers Extended (9 tools): antibody_scan, antibody_list, antibody_create, flow_simulate, epidemic, tremor, trust, layers, layer_inspect

**Proven results from live production audit (2026-03-14):**
- 39 bugs found in one session (28 confirmed fixed + 9 new high-confidence)
- 89% hypothesis accuracy across 10 live claims
- 223 turbulence points detected in WhatsApp subsystem — feature hold decision made in 3 minutes
- Layer detection: score 0.0 separation + 13,618 violations found in production codebase
- Flow simulation: 1,126 turbulence points total in backend, only 3 locks found
- Memory adapter: 82 docs ingested in 138ms → 19,797 nodes, cross-domain queries working
- 1.36µs spreading activation on 1K-node graph (criterion benchmark)
- Zero LLM tokens — pure Rust, local binary
- $600–4,800/year savings per developer vs grep-based audit workflows

**Supported MCP clients:** Claude Code, Cursor, Windsurf, Zed, Cline, Roo Code, Continue, OpenCode, Amazon Q, GitHub Copilot, and any stdio MCP client.

---

## Shipped — v0.3.0

**Added in v0.3 — Surgical nervous system (DONE):**
- `surgical_context_v2` — extended context with dependency graph edges, cross-file import chain, pre-formatted edit payload, BFS neighbourhood up to radius 2
- `apply` — write code back to file, trigger incremental re-ingest, run 5-layer verification pipeline (pattern detection + GraphDiff, antibody scan, CSR BFS blast radius, test identification, compile check)
- `apply_batch` — atomic multi-file edits with per-file graph re-ingest and blast-radius delta reporting

The 5-layer verification pipeline runs automatically on every `apply` call:
- **Layer A** — GraphDiff: pre/post node snapshot diff to detect structural changes and turbulence
- **Layer B** — Anti-pattern analysis: antibody registry scanned against the changed subgraph
- **Layer C** — BFS blast radius: CSR forward + reverse edge traversal to enumerate transitively affected nodes
- **Layer D** — Test identification: test nodes adjacent to changed files flagged for execution
- **Layer E** — Compile check: syntax/parse validation on the written file

**Total in v0.3: 56 MCP tools** (v0.2 base + 3 surgical tools: surgical_context_v2, apply, apply_batch)

---

## Shipped — v0.5.0

**Added in v0.5 — Verified writes, file tools, enhanced search (DONE):**
- `apply_batch` **5-layer post-write verification** (`verify=true`) — trivial-return detection, compilation check, structural diff, semantic coherence, and regression guard
- `m1nd.view` — lightweight file reader with auto-ingest. Zero-token alternative to Read for graph-tracked files
- `m1nd.glob` — file pattern matching with auto-ingest. Graph-aware alternative to Glob
- `m1nd.search` **enhanced** — invert matching, count mode, multiline, auto-ingest on miss, glob filtering

**Total in v0.5: 63 MCP tools**

---

## Shipped — v0.4.0

**Added in v0.4 — Search, efficiency, and panoramic analysis (DONE):**
- `m1nd.search` — unified literal/regex/semantic search across graph nodes and file contents. Graph-aware grep replacement with context lines and scope filtering.
- `m1nd.help` — runtime-discoverable tool documentation with m1nd's visual identity (⍌⍐⍂𝔻⟁ glyphs + ANSI color palette). Returns full index or per-tool docs with NEXT suggestions.
- `m1nd.panoramic` — full module risk panorama ranked by combined risk score (blast radius + centrality + churn). Critical alerts for modules with risk ≥ 0.7.
- `m1nd.savings` — session + global token economy report with CO2 estimation. Every m1nd query that replaces grep saves ~500 tokens; tracked cumulatively.
- `m1nd.report` — session summary: query log (last 10), timing statistics, tokens saved, graph state. Markdown output ready for display or logging.

**Total in v0.4: 61 MCP tools** across 11 categories:

| Category | Count | Tools |
|----------|-------|-------|
| Foundation | 13 | ingest, activate, impact, why, learn, drift, health, seek, scan, timeline, diverge, warmup, federate |
| Superpowers | 13 | hypothesize, counterfactual, missing, resonate, fingerprint, trace, validate_plan, predict, differential + trail (4) |
| Superpowers Extended | 9 | antibody_scan/create/list, flow_simulate, epidemic, tremor, trust, layers, layer_inspect |
| Perspective Navigation | 12 | start, routes, follow, back, peek, inspect, suggest, affinity, branch, compare, list, close |
| Lock System | 5 | create, watch, diff, rebase, release |
| Trail System | 4 | trail.save, trail.resume, trail.merge, trail.list |
| Surgical | 3 | surgical_context_v2, apply, apply_batch |
| Panoramic | 1 | panoramic |
| Efficiency | 1 | savings |
| Report | 1 | report |
| Help | 1 | help |

---

## Near-Term — v0.5

### ast-grep Pattern Integration

`antibody_create` currently uses topological subgraph fingerprinting. Adding ast-grep support
would allow antibody patterns expressed as concrete syntactic patterns (AST node shapes), making
antibody authoring more accessible to developers who prefer concrete code patterns over graph
abstractions.

The two approaches are complementary: graph fingerprints catch structural shape; ast-grep patterns
catch syntactic forms. Supporting both would increase antibody recall.

### Desktop Tray Application

A lightweight tray app (system menu bar) that wraps `--serve`, providing:
- One-click graph server start/stop
- System notification when turbulence or antibody matches are detected
- Status indicator (green/yellow/red) based on trust scores and tremor levels
- No terminal required for human developers who want a zero-config experience

### Visual TUI Layer

A terminal-based interactive graph explorer built on ratatui or similar:
- Navigate the code graph without leaving the terminal
- Live activation view: watch spreading activation propagate across nodes as you type queries
- Blast radius preview before `apply` executes
- Tremor heatmap across modules
- Designed for keyboard-driven workflows in remote/SSH environments where the browser UI is not available

### Multi-Repo Federation Improvements

`federate` already works (1.3s for two repos, 11,217 unified nodes). The current limitation:
cross-repo edges are inferred from namespace matching, not from actual import resolution.

v0.5 will add explicit cross-repo edge declaration — you define which modules in repo A export
to repo B, and m1nd resolves those edges explicitly. This makes `impact` and `why` accurate
across repo boundaries.

---

## Medium-Term — v0.6+

### TEMPESTA Orchestration Integration

TEMPESTA is a parallel build pattern where 16+ agents build modules simultaneously. m1nd is
already used as the coordination layer in TEMPESTA workflows (see [Use Cases](Use-Cases#13-build-orchestration)).

v0.6 will add first-class TEMPESTA support: a build manifest format that describes module
dependencies, and m1nd tooling that coordinates agent assignments, tracks build state, and
prevents agents from taking modules with unresolved dependencies.

The goal: m1nd becomes the scheduler and coordinator for parallel agent build workflows, not
just a passive graph query engine.

### Runtime Dimension

Today m1nd operates in four dimensions: structural, semantic, temporal, causal.

The fifth dimension: **runtime**. Ingest OpenTelemetry traces/spans as graph nodes. A function
that's structurally unrelated to another function may still be causally connected at runtime (via
a message queue, a shared database row, a timing dependency). Traces make those connections visible.

When runtime edges exist alongside structural edges, `flow_simulate` becomes dramatically more
accurate — it simulates actual observed execution paths, not just structural possibilities.

### Neural Semantic Search (Embeddings, full)

v0.4 ships the `search(mode="semantic")` path via trigram TF-IDF. v0.5 adds optional embedding
support: a bundled small model (no external API) for true semantic distance queries. When enabled,
the `search` and `seek` semantic dimension switches from trigram to embedding cosine similarity.

**Trade-off:** ~100ms added ingest per file, ~50MB model weight. Trigram mode remains default.

### Confidence Calibration Dashboard

The `hypothesize` tool reports confidence scores. Those scores are only useful if they're
calibrated — 99% confidence should mean the claim is true 99% of the time, not 70% of the time.

v0.6 will add a calibration dashboard: track every `hypothesize` call, record whether the
subsequent code investigation confirmed or denied the claim, and display calibration curves
over time. This makes the confidence scores falsifiable and improvable.

Current empirical calibration (10 claims, production codebase): 89% accuracy at >69% confidence
threshold. The goal is to get to 95%+ with the expanded data from real-world usage.

### Multi-Workspace Federation

Scale `federate` from two-repo to N-repo: a federation manifest that describes the full graph
of workspaces (microservices, monorepo subdirectories, cross-language projects). m1nd federates
all graphs into a unified view, resolves cross-workspace edges by namespace + export declarations,
and exposes the combined graph through a single query interface.

Use case: `impact(file::payments/auth.py)` propagates not just through the payments service but
through every downstream service that imports from it, across repo boundaries, in a single query.

---

## Long-Term Vision

### OVERVISION — Knowledge OS (Three Layers)

m1nd currently operates at the **code layer**: source files, functions, imports, edges. OVERVISION
extends perception into two additional layers:

**Layer 2 — Filesystem**: Every file on disk as a graph node. Documents, configs, notebooks,
assets, binaries. Edges from content relationships (YAML imports, markdown links, template
references). `why(config/nginx.conf, src/app.py)` answers "why are these connected" across
file types that have no syntactic relationship — only semantic and temporal.

**Layer 3 — Kernel**: OS-level graph. Process trees, file descriptors, network sockets, system
calls. A running application's live behavior as a graph. Connect what the code says it does
(Layer 1) to what the filesystem contains (Layer 2) to what the kernel observes at runtime
(Layer 3).

Together, the three layers form a complete perception stack: static structure + data topology +
runtime behavior. OVERVISION is the long-term architecture goal. Each layer is independently
useful; together they make AI agents able to answer questions about complex systems that no
single layer can answer alone.

### "The bottleneck for AI agents is never THINKING. It's PERCEIVING."

This is the core insight that drives m1nd's long-term direction. AI agents are powerful reasoners.
They can analyze any code you show them. The problem is FINDING what matters in a codebase of
10,000 files.

m1nd v0.4 solves structural perception (what exists, how it connects, what's missing). The
long-term roadmap extends perception into additional dimensions:

**Intent dimension**: PRDs, tickets, and specs as graph nodes connected to their implementations.
Ask "what was the original intent behind this module?" and get the linked specification document.
Find specs that have no implementation. Find implementations with no spec.

**Metacognition dimension**: The agent's own calibration history as plasticity weights. m1nd
learns not just what YOUR codebase looks like, but how well YOUR team's queries map to actual
bugs. Calibration improves automatically from use.

**Awareness dimension**: Multiple agents' perspectives as live activation state. In a multi-agent
system, each agent has a different view of the codebase based on its investigations. The awareness
dimension makes those views visible to each other — one agent can see where another agent's
investigation is focused without needing to communicate explicitly.

Together, these extensions move m1nd from a structural analysis tool to a full perception layer
for AI agents operating on code.

### AR/VR Graph Exploration

As XR hardware matures, there's an obvious long-term application: navigating the code graph in
three dimensions, walking through the architecture, watching activation propagate in real space.

This is speculative and hardware-dependent. The graph data model doesn't need to change for this.
It's a new rendering layer on top of the same underlying graph.

---

## How to Influence the Roadmap

m1nd is shaped by real usage. If you're using it and hit a limitation, open an issue. If you
have a use case that doesn't fit the current tools, open an issue. The near-term roadmap is
driven by what users actually need.

For contributions: see [CONTRIBUTING.md](https://github.com/cosmophonix/m1nd/blob/main/CONTRIBUTING.md).

For discussions: use GitHub Discussions.

---

*Roadmap as of 2026-03-16. Subject to change based on usage feedback and empirical results.*
