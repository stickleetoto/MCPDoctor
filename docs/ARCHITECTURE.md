# MCPDoctor Architecture

## v0.1 data flow

\`\`\`text
CLI
 |
 v
probe.rs
 |  builds standards-aware discovery request
 |  sends Streamable HTTP POST
 |  bounds response size
 |  decodes JSON or request-scoped SSE
 v
checks.rs
 |  classifies protocol/transport findings
 |  emits stable check IDs
 v
report.rs
 |  human output
 |  JSON output
 |  exit-status policy
\`\`\`

## Module boundaries

\`cli.rs\` owns command-line syntax and defaults.

\`probe.rs\` owns active I/O, timeout/size limits, required Streamable HTTP request metadata, direct JSON decoding, and request-scoped SSE extraction.

\`checks.rs\` owns diagnostic policy. Findings have stable IDs so protocol revisions can evolve without coupling the reporting layer to prose.

\`report.rs\` owns the stable JSON shape, human rendering, counters, and failure aggregation.

## Safety boundaries

MCPDoctor is a diagnostic client, not a traffic recorder.

- no raw response persistence by default;
- no authorization request headers printed;
- v0.1 has no bearer-token CLI flag;
- response inspection is bounded by \`--max-body-bytes\`;
- server identity is self-reported debugging data, not trusted identity.

## Version policy

The v0.1 catalog is primarily authored for \`2026-07-28\`. Rules whose meaning changes by revision should become explicitly version-scoped. v0.2 adds migration/legacy probing rather than pretending one request format covers both eras.
