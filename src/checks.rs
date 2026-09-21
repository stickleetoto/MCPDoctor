use reqwest::{
    header::{HeaderMap, CONTENT_TYPE, WWW_AUTHENTICATE},
    StatusCode,
};
use serde_json::{Map, Value};

use crate::{
    cli::DEFAULT_PROTOCOL,
    report::{fail, info, pass, warn, Report},
};

const SERVER_INFO_META_KEY: &str = "io.modelcontextprotocol/serverInfo";
const HEADER_MISMATCH: i64 = -32020;
const MISSING_REQUIRED_CLIENT_CAPABILITY: i64 = -32021;
const UNSUPPORTED_PROTOCOL_VERSION: i64 = -32022;

pub fn check_http_status(status: StatusCode, headers: &HeaderMap, report: &mut Report) {
    if status.is_success() {
        report.push(pass(
            "http.status",
            format!("HTTP status {}", status.as_u16()),
            "The endpoint accepted the discovery probe.",
        ));
        return;
    }

    let detail = match status {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
            let challenge = headers
                .get(WWW_AUTHENTICATE)
                .and_then(|value| value.to_str().ok())
                .map(|value| format!(" Authentication challenge: {value}"))
                .unwrap_or_default();
            format!(
                "The endpoint requires authorization or rejected the current authorization context.{challenge}"
            )
        }
        _ => "The endpoint did not return a successful HTTP response for server/discover.".into(),
    };

    report.push(fail(
        "http.status",
        format!("HTTP status {}", status.as_u16()),
        detail,
        "Inspect the protocol-specific JSON-RPC error below when available. Also confirm endpoint path, protocol revision, routing headers, and authorization requirements.",
    ));
}

pub fn check_content_type(headers: &HeaderMap, report: &mut Report) -> ResponseKind {
    let content_type = headers
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    let lower = content_type.to_ascii_lowercase();

    if lower.contains("application/json") {
        report.push(pass(
            "http.content_type",
            "JSON response content type",
            content_type,
        ));
        ResponseKind::Json
    } else if lower.contains("text/event-stream") {
        report.push(pass(
            "http.content_type",
            "SSE response content type",
            content_type,
        ));
        ResponseKind::Sse
    } else if content_type.is_empty() {
        report.push(fail(
            "http.content_type",
            "Missing Content-Type header",
            "A JSON-RPC request over Streamable HTTP must return application/json or text/event-stream.",
            "Return Content-Type: application/json for a direct response or text/event-stream for a request-scoped SSE response.",
        ));
        ResponseKind::Unknown
    } else {
        report.push(fail(
            "http.content_type",
            "Unsupported response content type",
            content_type,
            "Return application/json or text/event-stream for Streamable HTTP JSON-RPC requests.",
        ));
        ResponseKind::Unknown
    }
}

pub fn check_session_header(headers: &HeaderMap, protocol: &str, report: &mut Report) {
    if protocol != DEFAULT_PROTOCOL {
        return;
    }

    if let Some(session) = headers
        .get("Mcp-Session-Id")
        .and_then(|value| value.to_str().ok())
    {
        report.push(warn(
            "transport.session_header",
            "Legacy Mcp-Session-Id observed on a 2026-07-28 probe",
            format!("Mcp-Session-Id: {session}"),
            "The 2026-07-28 protocol removed protocol-level sessions. Keep legacy session behavior isolated to older protocol revisions.",
        ));
    } else {
        report.push(pass(
            "transport.session_header",
            "No legacy session header observed",
            "The response did not expose Mcp-Session-Id for the stateless 2026-07-28 probe.",
        ));
    }
}

pub fn check_jsonrpc_message(
    value: &Value,
    expected_id: &Value,
    protocol: &str,
    report: &mut Report,
) {
    match value.get("jsonrpc").and_then(Value::as_str) {
        Some("2.0") => report.push(pass(
            "jsonrpc.version",
            "JSON-RPC 2.0 envelope",
            "jsonrpc is exactly \"2.0\".",
        )),
        other => report.push(fail(
            "jsonrpc.version",
            "Invalid JSON-RPC version",
            format!("Observed: {other:?}"),
            "Set the top-level jsonrpc field to \"2.0\".",
        )),
    }

    match value.get("id") {
        Some(id) if id == expected_id => report.push(pass(
            "jsonrpc.id",
            "Response id matches request id",
            compact_json(id),
        )),
        Some(id) => report.push(fail(
            "jsonrpc.id",
            "Response id does not match request id",
            format!(
                "Expected {}, observed {}",
                compact_json(expected_id),
                compact_json(id)
            ),
            "Echo the request id unchanged in the JSON-RPC response.",
        )),
        None => report.push(fail(
            "jsonrpc.id",
            "Missing response id",
            "The discovery request used an id but the response did not return one.",
            "Echo the JSON-RPC request id in the response.",
        )),
    }

    let has_result = value.get("result").is_some();
    let has_error = value.get("error").is_some();
    match (has_result, has_error) {
        (true, false) | (false, true) => report.push(pass(
            "jsonrpc.shape",
            "Response contains exactly one of result or error",
            if has_result { "result" } else { "error" },
        )),
        (true, true) => report.push(fail(
            "jsonrpc.shape",
            "Response contains both result and error",
            "JSON-RPC responses must contain one or the other, not both.",
            "Return exactly one top-level result or error member.",
        )),
        (false, false) => report.push(fail(
            "jsonrpc.shape",
            "Response contains neither result nor error",
            "The JSON-RPC response has no outcome member.",
            "Return exactly one top-level result or error member.",
        )),
    }

    if let Some(error) = value.get("error") {
        check_error(error, protocol, report);
        return;
    }

    let Some(result) = value.get("result").and_then(Value::as_object) else {
        report.push(fail(
            "discover.result",
            "Missing server/discover result object",
            "No object-valued result field was found.",
            "Return a DiscoverResult containing supportedVersions, capabilities, resultType, ttlMs, and cacheScope.",
        ));
        return;
    };

    report.push(pass(
        "discover.result",
        "DiscoverResult object present",
        "The response contains an object-valued result.",
    ));

    check_supported_versions(result.get("supportedVersions"), protocol, report);
    check_capabilities(result.get("capabilities"), report);
    check_result_type(result.get("resultType"), report);
    check_cache_hints(result.get("ttlMs"), result.get("cacheScope"), report);
    check_server_info(result.get("_meta"), report);
    check_instructions(result.get("instructions"), report);
}

fn check_error(error: &Value, protocol: &str, report: &mut Report) {
    let code = error.get("code").and_then(Value::as_i64);
    let message = error
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or("<missing message>");

    match code {
        Some(UNSUPPORTED_PROTOCOL_VERSION) => {
            let supported = error
                .get("data")
                .and_then(|data| data.get("supported"))
                .map(compact_json)
                .unwrap_or_else(|| "<not provided>".into());
            report.push(fail(
                "protocol.unsupported",
                format!("Server rejected protocol {protocol}"),
                format!("{message}; supported={supported}"),
                "Retry using a mutually supported modern protocol revision. Do not interpret this recognized modern error as a reason to fall back to the legacy initialize handshake.",
            ));
        }
        Some(HEADER_MISMATCH) => report.push(fail(
            "transport.header_mismatch",
            "Server reported HeaderMismatch",
            message,
            "Verify MCP-Protocol-Version matches request _meta and that Mcp-Method mirrors the JSON-RPC method. Check intermediaries for header rewriting.",
        )),
        Some(MISSING_REQUIRED_CLIENT_CAPABILITY) => {
            let required = error
                .get("data")
                .and_then(|data| data.get("requiredCapabilities"))
                .map(compact_json)
                .unwrap_or_else(|| "<not provided>".into());
            report.push(fail(
                "protocol.client_capability",
                "Server requires a client capability not declared by MCPDoctor",
                format!("{message}; requiredCapabilities={required}"),
                "Confirm whether discovery truly requires that capability. server/discover should normally be usable with an empty clientCapabilities object.",
            ));
        }
        Some(-32601) => report.push(fail(
            "discover.method",
            "server/discover is not implemented",
            message,
            "For MCP 2026-07-28, implement server/discover. If this is intentionally a 2025-era server, probe or configure it as a legacy revision instead.",
        )),
        Some(-32602) => report.push(fail(
            "discover.params",
            "server/discover rejected request parameters",
            message,
            "Accept the standard per-request _meta containing protocolVersion, clientInfo, and clientCapabilities. Inspect any error data for the rejected field.",
        )),
        Some(other) => report.push(fail(
            "discover.error",
            format!("server/discover returned JSON-RPC error {other}"),
            message,
            "Inspect the server error and confirm support for modern per-request metadata and server/discover.",
        )),
        None => report.push(fail(
            "discover.error",
            "server/discover returned an invalid JSON-RPC error",
            compact_json(error),
            "Return an error object containing an integer code and string message.",
        )),
    }
}

fn check_supported_versions(value: Option<&Value>, protocol: &str, report: &mut Report) {
    let Some(versions) = value.and_then(Value::as_array) else {
        report.push(fail(
            "discover.supported_versions",
            "supportedVersions is missing or not an array",
            "The server did not advertise protocol versions in the expected shape.",
            "Return supportedVersions as a non-empty array of protocol revision strings.",
        ));
        return;
    };

    let strings: Vec<&str> = versions.iter().filter_map(Value::as_str).collect();
    if strings.len() != versions.len()
        || strings.is_empty()
        || strings.iter().any(|version| version.is_empty())
    {
        report.push(fail(
            "discover.supported_versions",
            "supportedVersions contains invalid entries",
            compact_json(value.unwrap_or(&Value::Null)),
            "Use a non-empty array containing only non-empty protocol revision strings.",
        ));
        return;
    }

    if strings.contains(&protocol) {
        report.push(pass(
            "discover.supported_versions",
            format!("Server advertises {protocol}"),
            strings.join(", "),
        ));
    } else {
        report.push(fail(
            "discover.supported_versions",
            format!("Server does not advertise {protocol}"),
            strings.join(", "),
            format!(
                "Retry using one of the server's advertised revisions or add {protocol} support."
            ),
        ));
    }
}

fn check_capabilities(value: Option<&Value>, report: &mut Report) {
    match value {
        Some(Value::Object(map)) => report.push(pass(
            "discover.capabilities",
            "Capabilities object present",
            format!("{} top-level capability entries", map.len()),
        )),
        _ => report.push(fail(
            "discover.capabilities",
            "Missing capabilities object",
            "DiscoverResult must advertise server capabilities.",
            "Return capabilities as an object, even when no optional capability is enabled.",
        )),
    }
}

fn check_result_type(value: Option<&Value>, report: &mut Report) {
    match value.and_then(Value::as_str) {
        Some("complete") => report.push(pass(
            "discover.result_type",
            "resultType is complete",
            "complete",
        )),
        Some(kind) if !kind.trim().is_empty() => report.push(warn(
            "discover.result_type",
            "Unexpected resultType for discovery",
            kind,
            "server/discover is normally a complete cacheable result. Confirm that the selected resultType is intentional and supported by clients.",
        )),
        _ => report.push(fail(
            "discover.result_type",
            "Missing resultType",
            "2026-07-28 results require the resultType discriminator.",
            "Include resultType in DiscoverResult; a normal discovery response uses \"complete\".",
        )),
    }
}

fn check_cache_hints(ttl: Option<&Value>, scope: Option<&Value>, report: &mut Report) {
    match ttl.and_then(Value::as_u64) {
        Some(value) => report.push(pass(
            "discover.cache.ttl",
            "ttlMs present",
            format!("{value} ms"),
        )),
        None => report.push(fail(
            "discover.cache.ttl",
            "Missing or invalid ttlMs",
            "2026-07-28 DiscoverResult requires a non-negative integer ttlMs.",
            "Return ttlMs. Use 0 when the response should be considered immediately stale.",
        )),
    }

    match scope.and_then(Value::as_str) {
        Some(scope @ ("public" | "private")) => report.push(pass(
            "discover.cache.scope",
            "cacheScope present",
            scope,
        )),
        Some(other) => report.push(fail(
            "discover.cache.scope",
            "Invalid cacheScope",
            other,
            "Use \"private\" for authorization-context-specific results or \"public\" only when safe to share across authorization contexts.",
        )),
        None => report.push(fail(
            "discover.cache.scope",
            "Missing cacheScope",
            "2026-07-28 DiscoverResult requires cacheScope.",
            "Return cacheScope as either \"private\" or \"public\".",
        )),
    }
}

fn check_server_info(meta: Option<&Value>, report: &mut Report) {
    let server_info = meta
        .and_then(Value::as_object)
        .and_then(|map| map.get(SERVER_INFO_META_KEY))
        .and_then(Value::as_object);

    match server_info {
        Some(info) if implementation_is_complete(info) => report.push(pass(
            "discover.server_info",
            "Server identity metadata present",
            compact_json(&Value::Object(info.clone())),
        )),
        _ => report.push(warn(
            "discover.server_info",
            "Server identity metadata is absent or incomplete",
            format!(
                "Expected result._meta[\"{SERVER_INFO_META_KEY}\"] with non-empty name and version."
            ),
            "Servers SHOULD identify themselves on 2026-07-28 responses. Add name/version metadata for display and debugging; do not use it for security decisions.",
        )),
    }
}

fn implementation_is_complete(info: &Map<String, Value>) -> bool {
    let name_ok = info
        .get("name")
        .and_then(Value::as_str)
        .is_some_and(|value| !value.trim().is_empty());
    let version_ok = info
        .get("version")
        .and_then(Value::as_str)
        .is_some_and(|value| !value.trim().is_empty());
    name_ok && version_ok
}

fn check_instructions(value: Option<&Value>, report: &mut Report) {
    match value {
        Some(Value::String(text)) if !text.trim().is_empty() => report.push(info(
            "discover.instructions",
            "Server instructions advertised",
            format!("{} characters", text.chars().count()),
        )),
        Some(Value::String(_)) => report.push(warn(
            "discover.instructions",
            "Server instructions are empty",
            "instructions is optional, but an empty value provides no guidance.",
            "Either omit instructions or provide concise guidance that helps clients and models use the server.",
        )),
        Some(other) => report.push(fail(
            "discover.instructions",
            "instructions is not a string",
            compact_json(other),
            "Omit instructions or return it as a string.",
        )),
        None => {}
    }
}

pub fn compact_json(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "<unserializable>".into())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseKind {
    Json,
    Sse,
    Unknown,
}
