# Tweet Variations — m1nd launch

---

## TWEET 1 — Core value prop (short)

shipped m1nd today. rust binary that builds a code graph and answers structural questions your AI agent can't.

"what breaks if i remove this?" 3ms.
"does A depend on B at runtime?" 89% accuracy.
"where are bugs hiding?" found 39 in one session, 8 invisible to grep.

52 MCP tools, zero LLM tokens, MIT.

github.com/maxkle1nz/m1nd

---

## TWEET 2 — The learning angle

the part about m1nd i didn't expect to work this well: it learns.

mark a query result as useful, edge weights strengthen. mark it wrong, they weaken. after 3 sessions on the same codebase it started surfacing connections i would've missed.

Hebbian plasticity on a code graph. weird idea. seems to work.

github.com/maxkle1nz/m1nd

---

## TWEET 3 — The numbers tweet

ran m1nd on a 52K line python backend.

46 graph queries instead of ~210 grep operations.
~3 seconds total vs 35 minutes estimated.
39 bugs found. 28 confirmed fixed.
8 of those 28 were invisible to any text search.

it's a rust binary. ~8MB. zero tokens. zero cloud.

github.com/maxkle1nz/m1nd

---

## TWEET 4 — The AI agent angle

your AI agent is navigating your codebase with grep and vibes.

m1nd gives it a structural map. 52 MCP tools for questions like:
- what's the blast radius of this change?
- which modules are most likely to have undiscovered bugs?
- does this dependency actually exist at runtime?

zero tokens. pure rust. works with claude code, cursor, windsurf, anything MCP.

github.com/maxkle1nz/m1nd

---

## TWEET 5 — Thread opener (for a longer thread)

shipped something i've been building for months. m1nd: a code graph that learns from how you use it.

thread on what it does, why i built it, and the 5 weird capabilities that surprised me most. 👇

/1

---

/2 the problem: grep finds text. code isn't text, it's a graph. dependencies, call chains, things that break when you touch something else.

i kept asking questions that grep can't answer. "does A actually depend on B at runtime?" "what breaks if i remove this?" "where are the bugs i haven't found yet?"

/3 so i built m1nd. rust binary, 3 crates. ingest your codebase in 910ms for 335 files (9,767 nodes, 26,557 edges). then query at microsecond speeds.

activate: 1.36µs. impact: 543ns. trace: 3.5ms. zero LLM calls. local, offline, ~8MB.

/4 tested it on a 52K line production backend. 39 bugs in one session. 28 confirmed fixed. 8 of those required structural analysis. grep would've never found them.

hypothesis engine ran at 89% accuracy across 10 live claims.

/5 the 5 things that surprised me:

1/ the graph learns (Hebbian plasticity). by session 3 it knew my codebase better than i did.

2/ antibody system: stores bug shapes. every ingest scans for recurrence.

3/ epidemic engine: predicts which modules harbor undiscovered bugs via SIR propagation.

4/ tremor: change acceleration predicts bugs better than raw churn.

5/ trail system: mid-investigation snapshots. resume 3 days later from the exact same state.

/6 52 MCP tools total. works with claude code, cursor, windsurf, zed, opencode, whatever.

limitations: no neural semantic search (trigram in v1), not optimized above 400K files, no dataflow/taint analysis.

MIT. github.com/maxkle1nz/m1nd

if you try it lmk how it goes.
