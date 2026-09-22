# MCPDoctor check catalog

MCPDoctor keeps stable check IDs so CI systems can baseline or suppress a finding without parsing human prose.

## Severity model

- **PASS** — expected condition observed.
- **WARN** — interoperability, migration, debugging, or best-practice concern; not necessarily blocking.
- **FAIL** — blocking incompatibility or required protocol shape is absent or invalid.
- **INFO** — useful context without a verdict.

## v0.1 check IDs

| Check ID | Purpose |
|---|---|
| \`target.url\` | target URL is usable |
| \`http.reachability\` | endpoint can be reached |
| \`http.redirect\` | reveal endpoint redirects |
| \`http.status\` | HTTP outcome of discovery probe |
| \`http.content_type\` | JSON or request-scoped SSE response type |
| \`transport.session_header\` | legacy \`Mcp-Session-Id\` leakage into 2026 traffic |
| \`body.size\` | response stays inside inspection limit |
| \`body.utf8\` | body is UTF-8 |
| \`body.fallback\` | fallback parser used because media type was wrong |
| \`body.parse\` | body is interpretable as MCP JSON/SSE |
| \`json.parse\` | direct JSON response parses |
| \`sse.data\` | SSE includes data events |
| \`sse.json\` | SSE data events carry JSON |
| \`sse.final_response\` | request-scoped SSE includes the correlated final response |
| \`jsonrpc.version\` | JSON-RPC 2.0 |
| \`jsonrpc.id\` | response id matches request id |
| \`jsonrpc.shape\` | exactly one of result or error exists |
| \`protocol.unsupported\` | \`-32022\` unsupported protocol version |
| \`transport.header_mismatch\` | \`-32020\` routing/body mismatch |
| \`protocol.client_capability\` | \`-32021\` missing client capability |
| \`discover.method\` | \`server/discover\` missing |
| \`discover.params\` | modern discovery metadata rejected |
| \`discover.error\` | other discovery error |
| \`discover.result\` | DiscoverResult exists |
| \`discover.supported_versions\` | requested revision is advertised |
| \`discover.capabilities\` | capabilities object exists |
| \`discover.result_type\` | result discriminator exists |
| \`discover.cache.ttl\` | \`ttlMs\` exists and is non-negative |
| \`discover.cache.scope\` | \`cacheScope\` is private or public |
| \`discover.server_info\` | recommended server identity exists |
| \`discover.instructions\` | optional instructions field has a useful shape |

## Future families

v0.2 adds legacy/migration diagnosis. Later versions add tools, resources, prompts, MRTR, authorization, and advisory tool-surface quality checks.
