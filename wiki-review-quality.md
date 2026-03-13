# m1nd Wiki Quality Review

Reviewer: Editorial review agent
Date: 2026-03-13
Scope: All 23 .md files in `/tmp/m1nd-pages/docs/wiki/src/`
Reference: `/Users/cosmophonix/clawd/roomanizer-os/mcp/m1nd/README.md`

---

## Executive Summary

This is an unusually strong set of open source documentation for a v0.1 project. The writing is clear, technically precise, and well-organized. The architecture pages would be at home in a Rust RFC. The concept pages explain genuinely complex ideas (spreading activation, XLR noise cancellation) in a way that a developer with no neuroscience or audio engineering background can follow.

That said, there are concrete improvements that would meaningfully improve the experience for a developer encountering m1nd for the first time. The findings below are prioritized by impact.

---

## 1. COMPLETENESS

### 1.1 Tool Coverage: 43/43 documented -- PASS

All 43 tools listed in the README are documented across the six API reference pages. Every tool has:
- Description
- Parameters table with types, required/optional, defaults
- JSON-RPC example request
- JSON-RPC example response
- "When to Use" section
- "Related Tools" cross-references

This is comprehensive and consistent. No tool is missing.

### 1.2 Missing Content (gaps worth filling)

| Gap | Location | Impact |
|-----|----------|--------|
| **No "Glossary" page** | SUMMARY.md | Terms like "CSR", "PPMI", "LTP", "LTD", "scatter-max", "anti-seed" are explained in their respective concept pages but there is no single-page glossary. A new user skimming the API reference will encounter these terms without context. | HIGH |
| **No error catalog** | api-reference/overview.md | Tool-specific errors (node not found, perspective not found, lock ownership violation, CAS retry exhausted, generation mismatch) are mentioned in architecture pages but never collected into a user-facing error reference. Developers debugging integration issues need this. | MEDIUM |
| **No "Configuration Reference" standalone page** | Scattered across quickstart.md, mcp-server.md | Environment variables, config parameters, and their defaults appear in at least three different files. A single authoritative page would prevent contradictions as the project evolves. | MEDIUM |
| **No EXAMPLES.md in wiki** | benchmarks.md line 172 references `../EXAMPLES.md` | The benchmarks page links to an EXAMPLES.md file that exists in the README repo but is not present in the wiki source tree. This is a broken cross-reference. | LOW |
| **`m1nd.scan` patterns undocumented** | api-reference/exploration.md lines 104-108 | The 8 built-in scan patterns are listed by name (`error_handling`, `resource_cleanup`, etc.) but never described. What does each pattern look for? What are the heuristics? A developer cannot use `scan` effectively without knowing what each pattern detects. | MEDIUM |

### 1.3 Tutorials: Step-by-Step Followability

**quickstart.md** -- GOOD. Clear linear flow: prerequisites, install, configure, ingest, verify, query. The troubleshooting section at the end is a nice touch.

**first-query.md** -- VERY GOOD. The activate-learn-activate loop with before/after comparison table (Step 4) is the most effective demonstration of m1nd's value proposition in the entire wiki. The progression through structural holes and counterfactual simulation builds naturally.

**multi-agent.md** -- GOOD but could improve. The ROOMANIZER OS example at the end is specific to the project's own multi-agent system. A more generic example (e.g., "two Cursor windows, one frontend dev and one backend dev") would resonate with a broader open source audience. The current example reads slightly like internal documentation rather than public docs.

---

## 2. CLARITY

### 2.1 Jargon That Needs Explanation or a Glossary Link

| Term | Where First Used | Issue |
|------|-----------------|-------|
| **CSR** | introduction.md line 27 | Defined on first use ("CSR (Compressed Sparse Row)") but subsequent uses in other files (activation.md, graph-engine.md) use "CSR" without expansion. A new reader entering through the API reference would not know what this means. |
| **PPMI** | graph-engine.md line 234 | "Positive Pointwise Mutual Information" -- expanded once, then used as an abbreviation. Not explained at the conceptual level (what it measures, why it matters for co-occurrence). |
| **scatter-max** | spreading-activation.md line 48 | Used as a technical term without a formal definition. The pseudocode shows what it does, but the term itself is jargon from parallel computing. One sentence explaining "scatter-max means only the strongest signal arriving at a node survives" would help. |
| **SoA** | architecture/overview.md line 116 | "Struct-of-Arrays" -- expanded once. Readers unfamiliar with data-oriented design will not understand why this matters. One sentence about cache performance would help. |
| **FM-XXX codes** | Throughout architecture pages | Fix-me codes like FM-PL-008, FM-ING-004, FM-RES-007 appear in several places. These are internal issue/audit tracking codes. They are never explained, and a reader has no way to look them up. Either remove them from public docs or add a footnote explaining the convention. |

### 2.2 Sentences That Are Hard to Parse

| File | Line | Issue |
|------|------|-------|
| graph-engine.md | 142 | "Two CAS operations are provided: `atomic_max_weight` (only increases, for activation scatter-max) and `atomic_write_weight` (unconditional, for plasticity). Both retry up to 64 times (constant `CAS_RETRY_LIMIT`)." -- This sentence packs three concepts (CAS, the two variants, the retry limit) into one dense block. Breaking it into a small table would improve scannability. |
| graph-engine.md | 236-237 | "DeepWalk-lite random walks (20 walks/node, length 10, window 4) generate co-occurrence counts, normalized to Positive Pointwise Mutual Information (PPMI)." -- This is a lot of parameters dropped without context. What do these numbers mean? Why these values? |
| architecture/overview.md | 99-104 | The 10-step "Request Lifecycle" is excellent content but presented as a dense numbered list. A sequence diagram (Mermaid) would make this much easier to follow. The data flow diagram above is already Mermaid -- matching the style would be consistent. |

### 2.3 Particularly Clear Writing (worth preserving)

- The XLR noise cancellation page (`concepts/xlr-noise-cancellation.md`) is outstanding. The analogy from balanced audio cables to code graph noise is precise, accessible, and technically accurate. The before/after table (lines 189-227) is the best single illustration in the wiki.
- The "visual example" in `concepts/spreading-activation.md` (lines 157-217) with the Mermaid graph and step-by-step activation trace is the kind of worked example that makes complex algorithms understandable. More pages should follow this pattern.
- The Hebbian plasticity page explains the five-step learning cycle without dumbing it down. The Rust code snippets are minimal and relevant.

---

## 3. STRUCTURE AND NAVIGATION

### 3.1 SUMMARY.md Organization -- GOOD

The four-section structure (Architecture, Concepts, API Reference, Tutorials) with Reference appendix pages is logical. The order makes sense for both sequential reading and reference lookup.

### 3.2 Structural Issues

| Issue | Impact |
|-------|--------|
| **Concepts and Architecture overlap** | The graph-engine.md page repeats significant content from the spreading-activation.md and hebbian-plasticity.md concept pages. For example, the Hebbian plasticity five-step cycle appears in both `concepts/hebbian-plasticity.md` and `architecture/graph-engine.md` (lines 275-312). The architecture page should reference the concept page instead of duplicating. Currently a reader encounters the same material twice without knowing which is authoritative. |
| **API reference grouping does not match README** | The README groups tools as Foundation (13), Perspective Navigation (12), Lock System (5), Superpowers (13). The API reference groups them as Core Activation (3), Analysis (7), Memory & Learning (7), Exploration (6), Perspectives (12), Lifecycle & Locks (8). Both groupings are reasonable, but the inconsistency will confuse readers who arrive from the README. Pick one and use it everywhere. |
| **"How to Read This Wiki" buried at line 97** | introduction.md puts the reading guide at the very end. This should be near the top (after the problem statement, before Key Capabilities) so that readers can choose their path early. |
| **No breadcrumb or "parent" links** | Individual pages (e.g., `api-reference/activation.md`) do not link back to the API overview or to SUMMARY.md. mdBook handles this via the sidebar, but readers using the raw markdown (GitHub) will have no navigation. |

### 3.3 Cross-References Between Pages

Cross-references are consistently formatted as `[text](relative-path.md)` or `[text](page.md#anchor)`. I verified all internal links:

**All links resolve to files that exist.** No broken internal links detected.

The one external-reference issue: `benchmarks.md` line 172 references `../EXAMPLES.md` which is not in the wiki tree. This would be a 404 in the published wiki.

---

## 4. CONSISTENCY

### 4.1 JSON-RPC Example Style

All API reference pages use the same structure:
1. Full JSON-RPC request with `jsonrpc`, `id`, `method`, `params`
2. Full JSON-RPC response (content body only, not the wrapper)

This is consistent across all 43 tool examples. The style is clean and the examples are realistic (not toy data).

**One inconsistency**: The tutorials (`quickstart.md`, `first-query.md`, `multi-agent.md`) use a shortened format without the `jsonrpc` and `id` fields in some examples, and include `jsonc` comments. This is fine for tutorials but creates a style gap with the API reference pages. Consider adding a note in the first tutorial: "Tutorials use a shortened JSON-RPC format for readability. See the API Reference for the full wire format."

### 4.2 Terminology

| Term Pair | Usage | Recommendation |
|-----------|-------|----------------|
| "activate" vs "query" vs "search" | Used interchangeably in some places | `activate` = the tool/operation. `query` = what the user provides. `search` should be avoided for activation (it implies text matching). Currently mostly consistent but the FAQ uses "search" loosely in a few answers. |
| "node_id" vs "external_id" vs "node identifier" | Used interchangeably in API reference | The API parameters use `node_id` as the parameter name. The architecture docs use `external_id` for the string key. These are the same thing but a user will not know that. Add one sentence to the API overview: "In parameters, `node_id` refers to the node's external identifier (e.g., `file::auth.py`)." |
| "edge weight" vs "weight" vs "signal strength" | Used across concept and API pages | Generally consistent, but `signal_strength` in API responses vs `edge_weight` in concept pages could confuse. A glossary would resolve this. |

### 4.3 Formatting Consistency

- **Tables**: Consistently use GitHub-flavored markdown tables. Column alignment varies (some left-aligned, some centered) but this is cosmetic.
- **Code blocks**: Consistently use triple backticks with language hints (`rust`, `json`, `jsonc`, `bash`). Good.
- **Headers**: H1 for page title, H2 for major sections, H3 for subsections. Consistent across all pages.
- **Mermaid diagrams**: Used in architecture pages and some concept pages. The concept pages that lack diagrams (structural-holes.md) would benefit from one.
- **"Constants reference" tables**: Appear at the end of concept pages (hebbian-plasticity.md, xlr-noise-cancellation.md, spreading-activation.md). This is a good pattern. The structural-holes.md page does not have one but should -- `min_sibling_activation` default, threshold of 2 neighbors, etc.

---

## 5. TONE

The tone is appropriate for open source technical documentation. It is:
- Confident without being arrogant
- Technical without being academic
- Direct without being terse

A few sentences lean slightly toward marketing copy:

| File | Line | Text | Suggestion |
|------|------|------|------------|
| introduction.md | 3 | "It ships as an MCP server with 43 tools, runs on stdio, and works with any MCP-compatible client." | This is fine -- factual and useful. |
| hebbian-plasticity.md | 3 | "No other code intelligence tool does this." | This claim appears twice (also in introduction.md). While likely true, it reads as marketing. Consider softening to "This is uncommon in code intelligence tooling." or simply removing it -- the technical explanation is compelling enough on its own. |
| faq.md | 33 | "Sourcegraph is a search engine. m1nd is a reasoning engine." | This is a useful framing but could offend Sourcegraph users/contributors. Consider "Sourcegraph excels at precise code search. m1nd excels at structural reasoning." |
| structural-holes.md | 132 | "## Why no other tool can do this" | Strong claim. Followed by specific comparisons that are accurate. The heading itself is the issue -- "Why this requires a graph" would be less confrontational. |

Overall the tone is well-calibrated. These are minor adjustments.

---

## 6. BROKEN LINKS AND REFERENCES

### Internal Links: ALL VERIFIED

Every `[text](path.md)` and `[text](path.md#anchor)` link in the wiki resolves to an existing file. Anchors use `#m1ndtoolname` format consistently. mdBook auto-generates these from H2/H3 headers, so they will work as long as the header text does not change.

### External Links

| File | Link | Status |
|------|------|--------|
| benchmarks.md line 172 | `../EXAMPLES.md` | **BROKEN** -- file does not exist in wiki tree. Exists in repo root. Either add EXAMPLES.md to the wiki or change the link. |

### Cross-Page Reference Patterns

The API reference pages use two cross-reference patterns:
1. Same-page anchors: `[m1nd.warmup](#m1ndwarmup)` -- correct
2. Cross-page links: `[m1nd.seek](exploration.md#m1ndseek)` -- correct

Both patterns are used consistently. The anchor naming convention (`#m1ndtoolname` with dots removed) is consistent across all pages.

---

## 7. TOP 5 IMPROVEMENTS BY IMPACT

### 1. Add a Glossary Page (HIGH IMPACT, LOW EFFORT)

Create `glossary.md` and add it to SUMMARY.md under Reference. Define: CSR, PPMI, LTP, LTD, scatter-max, anti-seed, seed, PageRank, SoA, FiniteF32, generation counter, trail, perspective, route, focus, anchor, ghost edge, structural hole. Link to it from the introduction's "How to Read This Wiki" section.

This single page would eliminate the largest barrier for new readers. Every concept page and architecture page assumes familiarity with terms that are only defined once, elsewhere.

### 2. Deduplicate Architecture and Concept Pages (MEDIUM IMPACT, MEDIUM EFFORT)

`architecture/graph-engine.md` duplicates substantial content from the concept pages. Refactor so that:
- **Concept pages** explain the *what* and *why* (theory, analogies, examples)
- **Architecture pages** explain the *how* (code structure, data types, implementation details)
- Architecture pages link to concept pages for conceptual background instead of repeating it

Specifically, the Hebbian plasticity five-step cycle, the XLR pipeline description, and the spreading activation algorithm all appear in both places. Keep the detailed versions in the concept pages and add "For the full explanation, see [Concept Name](../concepts/page.md)" links from the architecture pages.

### 3. Document the 8 Scan Patterns (MEDIUM IMPACT, LOW EFFORT)

`m1nd.scan` lists 8 built-in pattern IDs without describing what they detect. Add a subsection to `api-reference/exploration.md` after the scan tool documentation:

```markdown
### Built-in Pattern Reference

| Pattern | Detects | Example Finding |
|---------|---------|-----------------|
| `error_handling` | Bare except, missing error propagation | "Bare except in spawn_agent catches KeyboardInterrupt" |
| `resource_cleanup` | Unclosed files, missing context managers | ... |
| `api_surface` | Unvalidated inputs, missing auth checks | ... |
| ... | ... | ... |
```

Without this, `scan` is a black box. Users cannot make informed decisions about which pattern to run.

### 4. Standardize Tool Groupings Between README and Wiki (LOW IMPACT, LOW EFFORT)

The README uses: Foundation (13), Perspective Navigation (12), Lock System (5), Superpowers (13).
The wiki API reference uses: Core Activation (3), Analysis (7), Memory & Learning (7), Exploration (6), Perspectives (12), Lifecycle & Locks (8).
The MCP server architecture page uses yet another grouping (Core Query (13), Graph Mutation, Perspective (12), Lock (5), Trail (4), Topology).

Pick one grouping and use it everywhere. The wiki's six-group system is more granular and more useful for navigation. Align the README and the mcp-server.md page to match.

### 5. Add a Tutorial Note About JSON-RPC Format Differences (LOW IMPACT, LOW EFFORT)

At the top of `tutorials/quickstart.md`, add:

> **Note on examples**: Tutorials use a shortened JSON-RPC format with `jsonc` comments for readability. The `"jsonrpc": "2.0"` and `"id"` fields are omitted from some examples. If you are sending raw JSON-RPC, include these fields. See the [API Reference](../api-reference/overview.md) for the full wire format.

This prevents confusion when readers move between tutorials and the API reference and notice the format differences.

---

## Minor Issues (Low Priority)

| File | Line | Issue |
|------|------|-------|
| changelog.md | 42 | `seek` appears twice in the Superpowers list: once in Foundation and once in Superpowers. It should only appear once. |
| faq.md | 7 | "43 MCP tools for querying, learning, and navigating it" -- the verb "navigating" does not quite fit code. "navigating the graph" would be more precise. |
| quickstart.md | 38 | "The binary should start and wait for JSON-RPC input on stdin. Press Ctrl+C to exit." -- This is slightly misleading. `m1nd-mcp --help` would print help and exit, not wait for input. The wait-for-input behavior happens when you run `m1nd-mcp` with no arguments. |
| quickstart.md | 97-102 | Advanced configuration lists `learning_rate`, `decay_rate`, etc. as "set via environment" but does not show the environment variable names (e.g., `M1ND_LEARNING_RATE`?). The MCP server page shows `M1ND_XLR_ENABLED` but not the others. Are they configurable via env vars or only via JSON config? Clarify. |
| multi-agent.md | 158-172 | The cross-agent perspective.compare example has Agent A calling compare on both perspectives. The perspectives.md API reference (line 658) says "Both perspectives must belong to the same agent (V1 restriction)." This contradicts the tutorial's cross-agent example. One of them is wrong. |
| architecture/overview.md | 158-168 | Performance table lists "Predict (co-change) ~8ms" but the benchmarks page lists it under "Hebbian learn <1ms" without a separate predict entry. The README lists predict as "<1ms". Clarify: is predict <1ms or ~8ms? |

---

## Verdict

**Overall quality: 8.5/10.** This is substantially above average for open source documentation, especially for a v0.1 project. The concept pages are genuinely excellent -- they would work as standalone technical articles. The API reference is comprehensive and consistent. The main gaps are navigational (glossary, deduplication) rather than content gaps. The five improvements listed above would bring the wiki to a 9.5/10.
