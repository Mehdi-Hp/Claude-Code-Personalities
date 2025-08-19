use clap::{Arg, Command};
use anyhow::Result;
use colored::*;

mod cli;
mod statusline;
mod hooks;
mod state;
mod config;
mod types;
mod error;
mod animation;

// Module imports handled per-function

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        // Check if it's our custom error type that already has nice formatting
        if let Some(personality_err) = e.downcast_ref::<error::PersonalityError>() {
            eprintln!("{}", personality_err);
        } else {
            // Fallback for other anyhow errors - using Nerd Font error icon
            eprintln!("\u{f057} {}: {}", "Error".red().bold(), e);
        }
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let matches = Command::new("claude-code-personalities")
        .version("0.1.0")
        .about("Dynamic text-face personalities for Claude Code's statusline")
        .subcommand(
            Command::new("install")
                .about("Install Claude Code Personalities")
        )
        .subcommand(
            Command::new("update")
                .about("Update to the latest version")
        )
        .subcommand(
            Command::new("uninstall")
                .about("Remove Claude Code Personalities")
        )
        .subcommand(
            Command::new("status")
                .about("Check installation status")
        )
        .subcommand(
            Command::new("check-update")
                .about("Check for available updates")
        )
        .subcommand(
            Command::new("config")
                .about("Configure Claude Code Personalities display options")
        )
        .arg(
            Arg::new("statusline")
                .long("statusline")
                .help("Run in statusline mode (called by Claude Code)")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("hook")
                .long("hook")
                .help("Run in hook mode")
                .value_name("TYPE")
                .value_parser(["pre-tool", "post-tool", "prompt-submit", "session-end"])
        )
        .get_matches();

    // Handle different modes
    if matches.get_flag("statusline") {
        statusline::run_statusline().await
    } else if let Some(hook_type) = matches.get_one::<String>("hook") {
        hooks::run_hook(hook_type).await
    } else {
        // CLI commands
        match matches.subcommand() {
            Some(("install", _)) => cli::install().await,
            Some(("update", _)) => cli::update().await,
            Some(("uninstall", _)) => cli::uninstall().await,
            Some(("status", _)) => cli::status().await,
            Some(("check-update", _)) => cli::check_update().await,
            Some(("config", _)) => cli::configure().await,
            _ => cli::help(),
        }
    }
}
