# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| latest  | Yes       |

## Reporting a Vulnerability

If you discover a security vulnerability in m1nd, please report it responsibly.

**Do not open a public issue.**

Email: [security@m1nd.world](mailto:security@m1nd.world)

Include:
- Description of the vulnerability
- Steps to reproduce
- Impact assessment
- Suggested fix (if any)

You will receive an acknowledgment within 48 hours. We aim to release a fix within 7 days for critical issues.

## Scope

m1nd runs as a local MCP server over stdio. It does not open network ports, accept remote connections, or execute arbitrary code. The primary attack surface is:

- **Graph data integrity**: Malformed JSON input to `m1nd.ingest`
- **Path traversal**: File paths passed to ingest operations
- **Resource exhaustion**: Extremely large graphs or recursive queries

## Design Principles

- No network listeners. Stdio only.
- No code execution. Read-only analysis.
- No credential storage. No secrets in graph state.
- Sandboxed to the working directory by default.
