# Reddit Posts v2 — feedback style, not self-promotion

---

## r/rust

**Title:** feedback on a CSR graph engine with spreading activation — unsure about my noise cancellation approach

been working on a code graph engine in rust and wanted to get feedback from people who actually know the language better than me. i'm a system designer, not really a rust dev, so the AI writes most of the code but the architecture is mine.

it builds a CSR graph from your codebase (28 langs via tree-sitter) and does spreading activation queries across 4 dimensions (structural, semantic, temporal, causal). the part i'm most unsure about is the noise cancellation layer i call XLR — borrowed the concept from balanced audio cables. it dampens low-activation paths during signal propagation so you don't get garbage results on unrelated nodes. works well empirically but i don't have strong theoretical justification for the dampening coefficient.

also added Hebbian plasticity (edges strengthen when you confirm results, weaken when you reject). not sure how this compares to actual graph neural network approaches.

tested it on a 52K line python/fastapi backend: 39 bugs in one session, 8 required structural analysis that grep can't do. the hypothesis engine got 89% accuracy on 10 live claims.

three crates on crates.io: m1nd-core, m1nd-ingest, m1nd-mcp. ~20K lines, 354+ tests, MIT.

anyone here worked with graph signal propagation or spectral graph theory? curious how this compares to established methods.

repo: github.com/maxkle1nz/m1nd

---

## r/LocalLLaMA

**Title:** cut my agent's token usage by 60% with a local code graph — curious what approaches others use

been trying to reduce how many tokens my AI agents waste on grep calls when navigating codebases. the agent would do 200+ grep searches, read 228 files, burn 193K tokens just to understand dependencies in a 160K line backend. cost about $7 per investigation.

built a local rust binary that ingests the codebase into a graph (takes ~1 second) and answers structural questions directly. "what breaks if i remove this?" "does A depend on B at runtime?" "where are bugs hiding?" — stuff grep literally can't answer.

went from 193K tokens to 0 per investigation. the graph runs on CPU, no API calls, 8MB binary. 52 MCP tools so it works with claude code, cursor, whatever supports MCP.

the interesting part: the graph learns from feedback. tell it "yeah these results were useful" and edge weights strengthen via Hebbian plasticity. by day 3 it was noticeably better at my codebase.

curious if anyone else has tried graph-based approaches for reducing agent context. embeddings? RAG? something else? what's actually working for you?

tool if curious: github.com/maxkle1nz/m1nd

---

## r/programming

**Title:** found 8 bugs that grep can never find — they have no keyword, only structure

been debugging a 160K line python backend and kept hitting cases where i know something is wrong but there's no string to search for. TOCTOU race conditions, orphan process cascades, concurrent state corruption — these bugs live in the structure of the code, not the text.

built a tool that models the codebase as a weighted graph and queries it with spreading activation (like how neurons fire). ran it against the backend: 39 bugs in one session. 8 of them were completely invisible to any text search because they existed only as structural patterns between modules.

example: "does worker_pool depend on whatsapp_manager at runtime?" — grep says no, there's no direct import. the graph found a 2-hop dependency through process_manager's cancel function. confirmed real bug, 92% confidence.

the question i keep coming back to: how do you find bugs that have no keyword? manual code review? static analysis? curious what others do for structural bugs specifically.

the tool (rust, MIT): github.com/maxkle1nz/m1nd
