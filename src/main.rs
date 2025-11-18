use anyhow::Result;
use clap::{Arg, Command};
use colored::Colorize;

mod cli;
mod config;
mod error;
mod hooks;
mod icons;
mod kaomoji;
mod platform;
mod state;
mod statusline;
mod theme;
mod types;
mod version;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        // Check if it's our custom error type that already has nice formatting
        if let Some(personality_err) = e.downcast_ref::<error::PersonalityError>() {
            eprintln!("{personality_err}");
        } else {
            // Fallback for other anyhow errors
            eprintln!("\u{f057} {}: {}", "Error".red().bold(), e);
        }
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let matches = Command::new("claude-code-personalities")
        .version(version::CURRENT_VERSION)
        .about("Dynamic text-face personalities for Claude Code's statusline")
        .subcommand(
            Command::new("init")
                .about("Initialize Claude Code settings for personalities")
                .arg(
                    Arg::new("non_interactive")
                        .long("non-interactive")
                        .help("Skip all prompts (for automated setup)")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("backup")
                        .long("backup")
                        .help("Create backup of existing settings")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(Command::new("update").about("Update to the latest version"))
        .subcommand(Command::new("uninstall").about("Remove Claude Code Personalities"))
        .subcommand(Command::new("status").about("Check installation status"))
        .subcommand(Command::new("check-update").about("Check for available updates"))
        .subcommand(
            Command::new("config")
                .about("Configure Claude Code Personalities display options")
                .subcommand(
                    Command::new("display").about("Configure what appears in the statusline"),
                )
                .subcommand(Command::new("theme").about("Change color theme"))
                .subcommand(Command::new("reset").about("Reset all settings to defaults")),
        )
        .arg(
            Arg::new("statusline")
                .long("statusline")
                .help("Run in statusline mode (called by Claude Code)")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("hook")
                .long("hook")
                .help("Run in hook mode")
                .value_name("TYPE")
                .value_parser(["pre-tool", "post-tool", "prompt-submit", "session-end"]),
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
            Some(("init", sub_matches)) => {
                let non_interactive = sub_matches.get_flag("non_interactive");
                let backup = sub_matches.get_flag("backup");
                cli::init(non_interactive, backup).await
            }
            Some(("update", _)) => cli::update().await,
            Some(("uninstall", _)) => cli::uninstall().await,
            Some(("status", _)) => cli::status().await,
            Some(("check-update", _)) => cli::check_update().await,
            Some(("config", sub_matches)) => {
                let subcommand = sub_matches.subcommand().map(|(name, _)| name);
                cli::config::handle_config_command(subcommand).await
            }
            _ => cli::help(),
        }
    }
}
