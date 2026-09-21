# MCPDoctor

**Diagnose MCP compatibility problems and explain how to fix them.**

MCPDoctor is a developer-focused diagnostic CLI for Model Context Protocol servers. It complements the official conformance suite: conformance answers whether an implementation satisfies the protocol test suite, while MCPDoctor focuses on fast diagnosis, likely root causes, and actionable remediation.

> Status: **v0.1 development** — the first target is MCP \`2026-07-28\` Streamable HTTP discovery.

## What v0.1 checks

MCPDoctor currently checks endpoint reachability and redirects, HTTP status and response media type, direct JSON and request-scoped SSE responses, JSON-RPC correlation, \`server/discover\`, protocol negotiation, capabilities, cacheable-result fields, legacy session-header leakage, and server identity metadata.

The client sends the modern per-request \`_meta\`, \`MCP-Protocol-Version\`, \`Mcp-Method\`, and \`Accept: application/json, text/event-stream\` expected by the 2026 Streamable HTTP transport.

## Usage

Build from source:

\`\`\`bash
cargo build --release
\`\`\`

Probe an endpoint:

\`\`\`bash
mcp-doctor check http://127.0.0.1:3000/mcp
\`\`\`

Machine-readable output:

\`\`\`bash
mcp-doctor check http://127.0.0.1:3000/mcp --json
\`\`\`

Make warnings fail CI:

\`\`\`bash
mcp-doctor check http://127.0.0.1:3000/mcp --fail-on-warn
\`\`\`

Bound the amount of response data inspected:

\`\`\`bash
mcp-doctor check http://127.0.0.1:3000/mcp --max-body-bytes 1048576
\`\`\`

## Exit codes

| Code | Meaning |
|---:|---|
| \`0\` | no blocking findings |
| \`1\` | FAIL finding present, or WARN with \`--fail-on-warn\` |
| \`2\` | MCPDoctor itself could not execute the diagnostic |

Network failures are represented as diagnostics and therefore normally exit \`1\`, including in JSON mode.

## MCPDoctor vs MCPMeter

\`\`\`text
MCPMeter  -> What did this traffic cost?
MCPDoctor -> What is wrong with this MCP endpoint, and how do I fix it?
\`\`\`

MCPDoctor owns compatibility diagnosis and remediation. MCPMeter owns measurement and benchmarking.

## Relationship to official conformance

MCPDoctor is **not** intended to become a competing conformance authority. The official conformance suite remains the comprehensive machine-checkable source for protocol conformance.

MCPDoctor should instead detect common deployment and migration problems quickly, explain likely root causes, suggest concrete remediation, and eventually annotate imported conformance results with developer-friendly guidance.

## Project documents

- \`docs/CHECKS.md\` — stable diagnostic IDs and severity semantics
- \`docs/ROADMAP.md\` — staged development plan
- \`docs/ARCHITECTURE.md\` — v0.1 structure and boundaries

## License

MIT
