mod cli;
mod commands;
mod config;
mod error;
mod git;

use std::process;

use clap::Parser;
use log::LevelFilter;

use cli::{Cli, Commands};
use config::ConfigManager;

fn main() {
    let cli = Cli::parse();

    let level = if cli.debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Warn
    };
    env_logger::Builder::new().filter_level(level).init();

    let mut config = ConfigManager::new();

    match cli.command {
        Commands::Init => {
            commands::init::run(&mut config);
        }
        _ => {
            if !config.local_config_exists() || !config.projects_config_exists() {
                eprintln!("Repo not initialized. Run 'repo init' to initialize");
                process::exit(1);
            }

            if let Err(e) = config.read_local_config() {
                eprintln!("Failed to read local config: {}", e);
                process::exit(1);
            }
            if let Err(e) = config.read_projects_config() {
                eprintln!("Failed to read projects config: {}", e);
                process::exit(1);
            }

            match cli.command {
                Commands::Status => commands::status::run(&config),
                Commands::Fetch => commands::fetch::run(&config),
                Commands::Update => commands::update::run(&config),
                Commands::Init => unreachable!(),
            }
        }
    }
}
