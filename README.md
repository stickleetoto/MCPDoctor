# MCPDoctor

**Diagnose MCP compatibility problems and explain how to fix them.**

MCPDoctor is a practical diagnostic CLI for Model Context Protocol servers.

It complements the official MCP conformance suite:

- conformance answers whether an implementation satisfies the protocol test suite;
- MCPDoctor focuses on fast diagnosis, likely root causes, and actionable remediation.

> Status: early v0.1 development.

## Initial target

The first milestone focuses on MCP `2026-07-28` Streamable HTTP discovery.

```text
MCP server
    |
    v
MCPDoctor
    |
    +-- HTTP reachability
    +-- JSON-RPC envelope
    +-- server/discover
    +-- supported protocol versions
    +-- capabilities
    +-- cacheable-result fields
    +-- stateless-session sanity checks
    +-- server identity metadata
    |
    v
PASS / WARN / FAIL + remediation
```

## Planned usage

```bash
mcp-doctor check http://127.0.0.1:3000/mcp
mcp-doctor check http://127.0.0.1:3000/mcp --json
mcp-doctor check http://127.0.0.1:3000/mcp --protocol 2026-07-28
```

## MCPDoctor vs MCPMeter

```text
MCPMeter  -> What did this MCP traffic cost?
MCPDoctor -> What is wrong with this MCP endpoint, and how do I fix it?
```

MCPDoctor owns diagnostics. MCPMeter owns measurement and benchmarking.

## License

MIT
