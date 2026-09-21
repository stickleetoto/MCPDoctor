mod checks;
mod cli;
mod probe;
mod report;

use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;

use crate::{
    cli::{Cli, Command},
    probe::run_http_discovery_probe,
    report::print_human,
};

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("mcp-doctor: {error:#}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<ExitCode> {
    let cli = Cli::parse();

    match cli.command {
        Command::Check(args) => {
            let report = run_http_discovery_probe(&args)?;
            let failed =
                report.has_failures() || (args.fail_on_warn && report.has_warnings());

            if args.json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print_human(&report);
            }

            Ok(if failed {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            })
        }
    }
}
