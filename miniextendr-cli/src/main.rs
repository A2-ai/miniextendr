mod bridge;
mod cli;
mod commands;
mod output;
mod project;

use std::path::Path;
use std::process::ExitCode;

use anyhow::Result;
use clap::{CommandFactory, Parser};

use cli::{Cli, Command};
use project::ProjectContext;

fn main() -> ExitCode {
    let cli = Cli::parse();

    // Handle completions before project discovery (no project needed)
    if let Command::Completions { shell } = &cli.command {
        clap_complete::generate(
            *shell,
            &mut Cli::command(),
            "miniextendr",
            &mut std::io::stdout(),
        );
        return ExitCode::SUCCESS;
    }

    match run(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            if !cli.quiet {
                eprintln!("Error: {e:#}");
            }
            ExitCode::FAILURE
        }
    }
}

fn run(cli: &Cli) -> Result<()> {
    let ctx = ProjectContext::discover(Path::new(&cli.path))?;
    commands::dispatch(&cli.command, &ctx, cli.quiet, cli.json)
}
