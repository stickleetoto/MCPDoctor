# MCPDoctor Roadmap

## v0.1 — Discovery Doctor

Goal: turn common modern MCP HTTP compatibility failures into actionable diagnostics.

- [x] CLI and stable JSON report model
- [x] PASS / WARN / FAIL / INFO severities
- [x] modern per-request \`_meta\`
- [x] \`MCP-Protocol-Version\` and \`Mcp-Method\`
- [x] \`Accept: application/json, text/event-stream\`
- [x] direct JSON discovery responses
- [x] request-scoped SSE discovery responses
- [x] response-id correlation
- [x] protocol error classification (\`-32020\`, \`-32021\`, \`-32022\`)
- [x] \`supportedVersions\`, \`capabilities\`, \`resultType\`, \`ttlMs\`, \`cacheScope\`
- [x] server identity and legacy-session diagnostics
- [x] response size limit
- [x] CI-oriented JSON and exit codes
- [ ] deterministic fixture server
- [ ] broader response-classification tests
- [ ] Linux / Windows / macOS CI closeout
- [ ] v0.1 release package

## v0.2 — Migration Doctor

Goal: explain the difference between a working 2025-era implementation and \`2026-07-28\`.

- [ ] automatic legacy fallback probe
- [ ] detect initialize-only servers
- [ ] detect protocol-level session dependency
- [ ] per-request \`_meta\` diagnostics
- [ ] removed/deprecated surface diagnostics
- [ ] \`subscriptions/listen\` diagnostics
- [ ] migration report grouped by required / recommended / deprecated

## v0.3 — Full Endpoint Doctor

Add \`tools/list\`, \`prompts/list\`, \`resources/list\`, \`resources/read\`, cacheable-result validation, tool-call result validation, request-scoped SSE, subscriptions, and MRTR diagnostics.

## v0.4 — Authorization Doctor

Add protected-resource metadata, authorization-server discovery, common OAuth deployment mistakes, and safe reporting that never prints credentials.

## v0.5 — Conformance Integration

Do not duplicate the official suite. Detect/import official conformance results and merge failures with MCPDoctor remediation text.

## Later

stdio doctor mode, CI baseline/suppression, SDK-specific hints, SARIF, editor integration, GitHub Action, and an MCPDoctor self-hosted MCP tool.

## Non-goals

MCPDoctor is not the MCP specification authority, does not replace official conformance, does not store credentials/raw sensitive payloads, and does not benchmark performance.
