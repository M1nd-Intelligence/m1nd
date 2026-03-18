# Medium Comment — Response to "My Favorite 8 CLI Tools" by Bhavyansh

**Target post:** https://medium.com/@bhavyansh001/my-favorite-8-cli-tools-for-everyday-development-2025-edition-12340fad4b67

---

nice list, rg and fzf are in my daily stack too.

one thing tho, all these tools work on text. they find what you type. but code isn't text, it's a graph. dependencies, call chains, stuff that breaks when you touch something else.

i've been working on something for that. m1nd. it's a rust binary that builds a graph from your codebase (takes about 1 second for 335 files) and then you can ask it structural questions like "what breaks if i remove this module?" (3ms) or "does A actually depend on B at runtime?" (89% accuracy) or "where are the bugs i haven't found yet?" using an SIR propagation model.

it doesn't replace grep. it answers what grep can't. hidden dependencies, architectural violations, patterns with no keyword to search for.

ran it against a 52K line python backend, found 39 bugs in one session. 8 of them were invisible to any text search.

pure rust, MIT, no LLM tokens, no cloud. works as MCP server with claude code, cursor, windsurf, whatever.

github.com/maxkle1nz/m1nd

if anyone tries it alongside rg/fzf lmk how it goes
