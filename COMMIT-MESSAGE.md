# m1nd v0.2.0 — "The Graph That Learns"

## Engine (Rust)
- 9 new MCP tools (43→52): antibody_scan/list/create, flow_simulate, epidemic, tremor, trust, layers, layer_inspect
- 6 cross-domain features: Bug Antibodies, Flow Simulation, Epidemic Prediction, Code Tremors, Module Trust, Layer Detection
- 15 agent-controllable calibration knobs
- 54 new unit tests (182 total), 10 criterion benchmarks (activate 1.36µs)
- Inline docs, clippy clean, error handling verified
- Tuning: epidemic auto-calibrate, antibody substring DFS, flow depth caps

## Backend (Python — 39 bugs fixed)
- Security: forged attestation, command injection, path traversal, API key redaction
- Concurrency: worker_pool shutdown, ws_relay, circuit breaker, session_pool leak, sacred_memory, deep work dedup, settings lock
- Reliability: CancelledError handling (4 modules), whatsapp reconnect/dedup/buffer, observer DB lock retry, lifespan shutdown order, stormender TOCTOU/orphan
- Nerve: cookies action, intercept endpoint, sessionStorage, network truncation, 9→90 tests
- ~230 new tests, zero regressions

## Documentation
- README rewritten (52 tools, 28 languages, proven results)
- EXAMPLES expanded (real output, savings ledger, security pipeline)
- USE-CASES.md created (5 audiences, 4 proven pipelines)
- CONTRIBUTING expanded (7 sections), CHANGELOG created
- 9 GitHub Wiki pages (~90KB)
- 6 language translations (PT-BR, ES, DE, FR, JA, ZH)
- 428 previously undocumented features documented

## Key Metrics
- 52 tools, 338 tests, 10 benchmarks
- 39 bugs found via m1nd (89% hypothesis accuracy)
- $600-4,800/year savings per developer
- Zero LLM tokens consumed
