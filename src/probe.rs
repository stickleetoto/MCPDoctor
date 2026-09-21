use std::{
    io::Read,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use reqwest::{
    blocking::{Client, Response},
    header::{ACCEPT, CONTENT_TYPE},
};
use serde_json::{json, Value};
use url::Url;

use crate::{
    checks::{
        check_content_type, check_http_status, check_jsonrpc_message, check_session_header,
        ResponseKind,
    },
    cli::CheckArgs,
    report::{fail, info, pass, warn, Report},
};

const REQUEST_ID: &str = "mcp-doctor-discover-1";

pub fn run_http_discovery_probe(args: &CheckArgs) -> Result<Report> {
    let mut report = Report::new(args.target.clone(), args.protocol.clone());

    let target_url = match Url::parse(&args.target) {
        Ok(url) => url,
        Err(error) => {
            report.push(fail(
                "target.url",
                "Invalid target URL",
                error.to_string(),
                "Use an absolute http:// or https:// URL pointing to the MCP endpoint.",
            ));
            return Ok(report);
        }
    };

    if !matches!(target_url.scheme(), "http" | "https") {
        report.push(fail(
            "target.url",
            "Unsupported target URL scheme",
            target_url.scheme(),
            "Use an http:// or https:// MCP Streamable HTTP endpoint.",
        ));
        return Ok(report);
    }

    report.push(pass(
        "target.url",
        "Target URL is valid",
        target_url.as_str(),
    ));

    let client = Client::builder()
        .timeout(Duration::from_secs(args.timeout))
        .build()
        .context("failed to build HTTP client")?;

    let body = json!({
        "jsonrpc": "2.0",
        "id": REQUEST_ID,
        "method": "server/discover",
        "params": {
            "_meta": {
                "io.modelcontextprotocol/protocolVersion": args.protocol,
                "io.modelcontextprotocol/clientInfo": {
                    "name": "mcp-doctor",
                    "version": env!("CARGO_PKG_VERSION")
                },
                "io.modelcontextprotocol/clientCapabilities": {}
            }
        }
    });

    let started = Instant::now();
    let response = match client
        .post(&args.target)
        .header("MCP-Protocol-Version", &args.protocol)
        .header("Mcp-Method", "server/discover")
        .header(ACCEPT, "application/json, text/event-stream")
        .header(CONTENT_TYPE, "application/json")
        .json(&body)
        .send()
    {
        Ok(response) => response,
        Err(error) => {
            report.push(fail(
                "http.reachability",
                "Failed to reach MCP endpoint",
                error.to_string(),
                "Confirm the endpoint is running, the URL is correct, TLS trust is valid, and network policy permits the connection.",
            ));
            return Ok(report);
        }
    };
    let headers_elapsed = started.elapsed();

    report.push(pass(
        "http.reachability",
        "MCP endpoint reached",
        format!(
            "response headers received in {:.2} ms",
            headers_elapsed.as_secs_f64() * 1000.0
        ),
    ));

    report.effective_url = Some(response.url().to_string());
    if response.url().as_str() != args.target {
        report.push(warn(
            "http.redirect",
            "Request was redirected",
            format!("{} -> {}", args.target, response.url()),
            "Prefer configuring clients with the canonical MCP endpoint directly. Verify that redirects preserve MCP request headers and authorization safely.",
        ));
    } else {
        report.push(pass(
            "http.redirect",
            "No HTTP redirect observed",
            "The configured target handled the request directly.",
        ));
    }

    inspect_response(
        response,
        args,
        body.get("id").unwrap_or(&Value::Null),
        &mut report,
    )?;
    Ok(report)
}

fn inspect_response(
    mut response: Response,
    args: &CheckArgs,
    expected_id: &Value,
    report: &mut Report,
) -> Result<()> {
    let status = response.status();
    let headers = response.headers().clone();

    check_http_status(status, &headers, report);
    let kind = check_content_type(&headers, report);
    check_session_header(&headers, &args.protocol, report);

    let Some(body) = read_limited_body(&mut response, args.max_body_bytes, report)? else {
        return Ok(());
    };

    let text = match String::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            report.push(fail(
                "body.utf8",
                "Response body is not valid UTF-8",
                error.to_string(),
                "MCP JSON-RPC messages must be UTF-8 encoded.",
            ));
            return Ok(());
        }
    };

    report.push(pass(
        "body.utf8",
        "Response body is valid UTF-8",
        format!("{} bytes", text.len()),
    ));

    match kind {
        ResponseKind::Json => {
            report.response_mode = Some("json".into());
            inspect_json(&text, expected_id, &args.protocol, report);
        }
        ResponseKind::Sse => {
            report.response_mode = Some("sse".into());
            inspect_sse(&text, expected_id, &args.protocol, report);
        }
        ResponseKind::Unknown => {
            if serde_json::from_str::<Value>(&text).is_ok() {
                report.push(info(
                    "body.fallback",
                    "Body looks like JSON despite unsupported Content-Type",
                    "MCPDoctor parsed the body for additional diagnostics only.",
                ));
                report.response_mode = Some("json-fallback".into());
                inspect_json(&text, expected_id, &args.protocol, report);
            } else {
                report.push(fail(
                    "body.parse",
                    "Unable to interpret response body",
                    preview(&text),
                    "Return a JSON object or a request-scoped SSE stream containing JSON-RPC messages.",
                ));
            }
        }
    }

    Ok(())
}

fn read_limited_body(
    response: &mut Response,
    max_body_bytes: u64,
    report: &mut Report,
) -> Result<Option<Vec<u8>>> {
    let mut body = Vec::new();
    response
        .take(max_body_bytes.saturating_add(1))
        .read_to_end(&mut body)
        .context("failed to read response body")?;

    if body.len() as u64 > max_body_bytes {
        report.push(fail(
            "body.size",
            "Response body exceeded inspection limit",
            format!("more than {max_body_bytes} bytes"),
            "Reduce the discovery response size or raise --max-body-bytes intentionally for diagnosis.",
        ));
        return Ok(None);
    }

    report.push(pass(
        "body.size",
        "Response body is within inspection limit",
        format!("{} / {} bytes", body.len(), max_body_bytes),
    ));
    Ok(Some(body))
}

fn inspect_json(text: &str, expected_id: &Value, protocol: &str, report: &mut Report) {
    match serde_json::from_str::<Value>(text) {
        Ok(value) => {
            report.push(pass(
                "json.parse",
                "Response is valid JSON",
                "The response body could be parsed as one JSON value.",
            ));
            check_jsonrpc_message(&value, expected_id, protocol, report);
        }
        Err(error) => report.push(fail(
            "json.parse",
            "Response is not valid JSON",
            error.to_string(),
            "Return one valid JSON-RPC response object for application/json responses.",
        )),
    }
}

fn inspect_sse(text: &str, expected_id: &Value, protocol: &str, report: &mut Report) {
    let events = parse_sse_data_events(text);
    if events.is_empty() {
        report.push(fail(
            "sse.data",
            "SSE stream contains no data events",
            "No non-empty data: payloads were found.",
            "Emit JSON-RPC messages in SSE data fields and finish with the response for this request.",
        ));
        return;
    }

    report.push(pass(
        "sse.data",
        "SSE data events observed",
        format!("{} data event(s)", events.len()),
    ));

    let mut parsed = Vec::new();
    let mut invalid = 0usize;
    for data in &events {
        match serde_json::from_str::<Value>(data) {
            Ok(value) => parsed.push(value),
            Err(_) => invalid += 1,
        }
    }

    if invalid > 0 {
        report.push(fail(
            "sse.json",
            "One or more SSE data events are not valid JSON",
            format!("{invalid} invalid event(s) out of {}", events.len()),
            "Each MCP SSE data event must carry a complete JSON-RPC message encoded as JSON.",
        ));
    } else {
        report.push(pass(
            "sse.json",
            "All SSE data events contain valid JSON",
            format!("{} JSON message(s)", parsed.len()),
        ));
    }

    let response = parsed.iter().rev().find(|value| {
        value.get("id") == Some(expected_id)
            && (value.get("result").is_some() || value.get("error").is_some())
    });

    match response {
        Some(value) => {
            report.push(pass(
                "sse.final_response",
                "Correlated final JSON-RPC response found",
                "The request-scoped SSE stream contains a response matching the discovery request id.",
            ));
            check_jsonrpc_message(value, expected_id, protocol, report);
        }
        None => report.push(fail(
            "sse.final_response",
            "No correlated final JSON-RPC response found",
            format!(
                "Expected id {}",
                serde_json::to_string(expected_id).unwrap_or_default()
            ),
            "Finish the request-scoped SSE stream with the JSON-RPC response for the original request id.",
        )),
    }
}

fn parse_sse_data_events(text: &str) -> Vec<String> {
    let normalized = text.replace("\r\n", "\n");
    let mut events = Vec::new();
    let mut data_lines = Vec::new();

    for line in normalized.split('\n') {
        if line.is_empty() {
            flush_sse_event(&mut data_lines, &mut events);
            continue;
        }

        if line.starts_with(':') {
            continue;
        }

        if let Some(rest) = line.strip_prefix("data:") {
            data_lines.push(rest.strip_prefix(' ').unwrap_or(rest).to_string());
        }
    }
    flush_sse_event(&mut data_lines, &mut events);
    events
}

fn flush_sse_event(data_lines: &mut Vec<String>, events: &mut Vec<String>) {
    if !data_lines.is_empty() {
        events.push(data_lines.join("\n"));
        data_lines.clear();
    }
}

fn preview(text: &str) -> String {
    const MAX: usize = 240;
    let mut chars = text.chars();
    let head: String = chars.by_ref().take(MAX).collect();
    if chars.next().is_some() {
        format!("{head}…")
    } else {
        head
    }
}

#[cfg(test)]
mod tests {
    use super::parse_sse_data_events;

    #[test]
    fn parses_multiline_sse_data() {
        let events =
            parse_sse_data_events("event: message\ndata: {\"a\":\ndata: 1}\n\n");
        assert_eq!(events, vec!["{\"a\":\n1}"]);
    }

    #[test]
    fn ignores_comments_and_empty_events() {
        let events =
            parse_sse_data_events(": keepalive\n\ndata: {\"ok\":true}\n\n");
        assert_eq!(events, vec!["{\"ok\":true}"]);
    }
}
