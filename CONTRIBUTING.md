# Contributing to m1nd

Thanks for your interest in contributing to m1nd. This document covers the basics.

## Getting Started

```bash
git clone https://github.com/cosmophonix/m1nd.git
cd m1nd
cargo build
cargo test --all
```

## Project Structure

```
m1nd-core/     Graph engine, plasticity, spreading activation, hypothesis engine
m1nd-ingest/   Language extractors (Python, Rust, TS/JS, Go, Java, generic)
m1nd-mcp/      MCP server, 43 tool handlers, JSON-RPC over stdio
```

## What to Work On

### Language Extractors (high impact)

m1nd currently has extractors for Python, Rust, TypeScript/JavaScript, Go, Java, and a generic fallback. Adding tree-sitter integration or new language-specific extractors would expand m1nd's reach significantly.

Extractors live in `m1nd-ingest/src/`. Each extractor implements a trait that returns nodes and edges from source files.

### Graph Algorithms

The core engine in `m1nd-core/` has room for improvement:
- Community detection algorithms
- Better spreading activation decay functions
- Smarter ghost edge inference
- Embedding-based semantic scoring (V1 is trigram-only)

### MCP Tools

New tools that leverage the graph are welcome. Each tool is a handler in `m1nd-mcp/src/`. The pattern is consistent -- look at existing tools for the structure.

### Benchmarks

Run m1nd on your codebase and report performance. We track real-world numbers, not synthetic benchmarks.

## Code Standards

- `cargo fmt` before committing
- `cargo clippy -- -D warnings` must pass
- All new code needs tests
- No `unsafe` without a comment explaining why

## Pull Requests

1. Fork the repo and create a branch from `main`
2. Make your changes with tests
3. Ensure `cargo test --all` passes
4. Ensure `cargo clippy --all -- -D warnings` passes
5. Ensure `cargo fmt --all -- --check` passes
6. Open a PR with a clear description of what and why

## Issues

Use GitHub issues for bugs, feature requests, and questions. Label your issue:
- `bug` -- something doesn't work
- `enhancement` -- new feature or improvement
- `good first issue` -- suitable for new contributors
- `language-extractor` -- new language support
- `algorithm` -- graph algorithm work

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
