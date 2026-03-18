# Show HN — m1nd: code graph that learns from every query (Rust, 52 MCP tools, zero tokens)

---

**Title:** Show HN: m1nd – code graph with Hebbian plasticity, 52 MCP tools, zero LLM tokens

**Body:**

i've been debugging a 52K line python backend for the past few months and hit a wall with grep/ripgrep. not because they're bad at what they do, but because what i needed wasn't text search. i needed structural answers. "does A actually depend on B at runtime?" "if i remove this module, what breaks?" "where are bugs hiding that i haven't found yet?"

so i built m1nd. rust binary, builds a graph from your codebase, answers structural questions at microsecond speeds. 335 files → 9,767 nodes → 26,557 edges in 0.91 seconds. then activate in 31ms, impact in 5ms, trace in 3.5ms.

the part i'm most proud of: the graph learns. mark results as useful, edge weights strengthen. mark them wrong, they weaken. standard Hebbian plasticity, but applied to code structure. after a few sessions the graph reflects how your team actually thinks about the codebase, not just what's in the source.

tested it in a live audit session: 39 bugs found, 28 confirmed fixed. 8 of those 28 were invisible to any text search, required structural analysis to surface. hypothesis engine ran at 89% accuracy across 10 live claims.

5 systems beyond basic graph traversal:
- antibody system: stores bug patterns, scans for recurrence on every ingest
- epidemic engine: SIR propagation predicts which modules have undiscovered bugs
- tremor detection: change acceleration (second derivative of churn) is a better leading indicator than churn itself
- trust ledger: per-module risk scores from defect history
- trail system: save an investigation, come back 3 days later from the exact same state

works as an MCP server with claude code, cursor, windsurf, zed, whatever. 52 tools total. ~8MB binary, no API keys, no cloud, no LLM calls at query time.

three crates: m1nd-core (graph engine + all the weird stuff), m1nd-ingest (28 languages via tree-sitter), m1nd-mcp (the server + HTTP GUI).

what it can't do: neural semantic search (v1 is trigram, not embeddings), dataflow/taint analysis, sub-symbol tracking, or real-time indexing on every keystroke. it's session-level intelligence, not a replacement for your LSP.

github.com/maxkle1nz/m1nd, MIT license, i'm here for questions.

one thing i'm curious about: does anyone have experience applying learning systems to static analysis tools? the plasticity side is the part i feel least confident about from a theory perspective, even though it's working in practice.
