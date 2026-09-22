use clap::{Args, Parser, Subcommand};

pub const DEFAULT_PROTOCOL: &str = "2026-07-28";

#[derive(Debug, Parser)]
#[command(
    name = "mcp-doctor",
    version,
    about = "Diagnose MCP compatibility problems and explain how to fix them"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Probe an MCP Streamable HTTP endpoint and report compatibility findings.
    Check(CheckArgs),
}

#[derive(Debug, Args)]
pub struct CheckArgs {
    /// MCP Streamable HTTP endpoint, for example http://127.0.0.1:3000/mcp
    pub target: String,

    /// Protocol revision to probe.
    #[arg(long, default_value = DEFAULT_PROTOCOL)]
    pub protocol: String,

    /// Emit machine-readable JSON.
    #[arg(long)]
    pub json: bool,

    /// Treat warnings as a failing process exit status.
    #[arg(long)]
    pub fail_on_warn: bool,

    /// Request timeout in seconds.
    #[arg(long, default_value_t = 10, value_parser = clap::value_parser!(u64).range(1..=300))]
    pub timeout: u64,

    /// Maximum response body size to inspect, in bytes.
    #[arg(long, default_value_t = 1_048_576, value_parser = clap::value_parser!(u64).range(1024..=67_108_864))]
    pub max_body_bytes: u64,
}
