# awesome-rust Submission — m1nd

## Target Repository
https://github.com/rust-unofficial/awesome-rust

## Contribution Guidelines
- Accepted entries require: stars > 50 OR crates.io downloads > 2000
- Template: `[ACCOUNT/REPO](https://github.com/ACCOUNT/REPO) [[CRATE](https://crates.io/crates/CRATE)] - DESCRIPTION`
- CI badge goes after description, separated by a space
- Entries sorted alphabetically within section
- Full guidelines: https://github.com/rust-unofficial/awesome-rust/blob/main/CONTRIBUTING.md

## Note on Thresholds
m1nd currently has 1 star. The threshold is 50 stars OR 2000 crates.io downloads.
**Submit after reaching one of these thresholds.** Track at: https://crates.io/crates/m1nd-core

---

## Recommended Category: Libraries > Artificial Intelligence > Tooling

The `#### Tooling` subsection under `### Artificial Intelligence` is the exact fit.
Existing entries there include agent memory layers, secure Python sandboxes for LLMs,
and AI workflow languages — m1nd fits as the code-graph / intelligence layer for agents.

Current entries in that section (for alphabetical placement reference):
- BAML
- Cortex Memory
- memvid/memvid
- pydantic/monty

m1nd would slot **between `memvid/memvid` and `pydantic/monty`** alphabetically.

---

## Exact Line to Add

```markdown
* [maxkle1nz/m1nd](https://github.com/maxkle1nz/m1nd) [[m1nd-core](https://crates.io/crates/m1nd-core)] - Neuro-symbolic code graph with Hebbian plasticity and spreading activation. 52 MCP tools give AI agents structural code intelligence — impact analysis, gap detection, co-change prediction — at 1.36μs per query with zero LLM tokens. [![CI](https://github.com/maxkle1nz/m1nd/actions/workflows/ci.yml/badge.svg)](https://github.com/maxkle1nz/m1nd/actions/workflows/ci.yml)
```

---

## Alternative Category (if Tooling is rejected): Development tools (top-level)

The top-level `## Development tools` section lists tools that enhance the dev workflow.
m1nd qualifies as a developer tool for AI-assisted development environments.

Alphabetical placement: between `Racer` and `Rust Search Extension`.

```markdown
* [maxkle1nz/m1nd](https://github.com/maxkle1nz/m1nd) [[m1nd-core](https://crates.io/crates/m1nd-core)] - Neuro-symbolic code graph with Hebbian plasticity. 52 MCP tools for AI agents: spreading activation, impact analysis, co-change prediction, gap detection. 1.36μs queries, zero LLM tokens. [![CI](https://github.com/maxkle1nz/m1nd/actions/workflows/ci.yml/badge.svg)](https://github.com/maxkle1nz/m1nd/actions/workflows/ci.yml)
```

---

## How to Submit the PR

1. Fork https://github.com/rust-unofficial/awesome-rust
2. Edit `README.md` — insert the line above in the correct alphabetical position within `#### Tooling`
3. PR title: `Add m1nd — neuro-symbolic code graph for AI agents`
4. PR body: explain what m1nd does, link to crates.io, mention the CI badge

---

## Also Submit To: punkpeye/awesome-mcp-servers

https://github.com/punkpeye/awesome-mcp-servers  (20k+ stars, most active MCP list)

**Target section:** `### 💻 Developer Tools`

**Alphabetical placement:** between `muvon/octocode` and `nullptr-z/code-rag-golang`
(octocode is a code graph too — m1nd is complementary but distinct: it adds Hebbian plasticity + MCP tooling)

**Exact line to add:**
```markdown
- [maxkle1nz/m1nd](https://github.com/maxkle1nz/m1nd) 🦀 🏠 🍎 🪟 🐧 - Neuro-symbolic code graph with Hebbian plasticity. 52 MCP tools give AI agents structural code intelligence: spreading activation, impact radius, co-change prediction, gap detection, and hypothesis testing — 1.36μs per query, zero LLM tokens.
```

**How to submit:** The repo accepts PRs. Insert line alphabetically in `### Developer Tools`.

**Note:** `muvon/octocode` is a close neighbor (GraphRAG code indexer in Rust). Differentiate in PR description: m1nd adds Hebbian learning (the graph improves with every agent query), is not RAG-based, and exposes 52 specialized MCP tools vs a general semantic search interface.

---

## Also Consider: appcypher/awesome-mcp-servers

https://github.com/appcypher/awesome-mcp-servers

**Target section:** `💻 Development Tools`

**Format used there (simpler, no emoji legend):**
```markdown
- [m1nd](https://github.com/maxkle1nz/m1nd) ⭐ - Neuro-symbolic code graph for AI agents. Hebbian plasticity + spreading activation. 52 MCP tools: impact analysis, co-change prediction, gap detection, hypothesis testing. Pure Rust, 1.36μs queries, zero LLM tokens.
```

---

## Summary Table

| List | Stars | Category | Threshold | Status |
|------|-------|----------|-----------|--------|
| awesome-rust | 47k | Libraries > AI > Tooling | 50 stars OR 2k downloads | Wait for threshold |
| punkpeye/awesome-mcp-servers | 20k | Developer Tools | Open PRs | Ready to submit |
| appcypher/awesome-mcp-servers | 8k | Development Tools | Open PRs | Ready to submit |
