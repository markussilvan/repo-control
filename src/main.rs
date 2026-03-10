mod cli;
mod commands;
mod config;
mod error;
mod git;

use std::process;

use clap::Parser;

use cli::{Cli, Commands, ProjectCommands, ServerCommands};
use config::ConfigManager;

fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    let mut config = ConfigManager::new();

    if let Commands::Init = cli.command {
        commands::init::run(&mut config);
        return;
    }

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
        Commands::Init => unreachable!(),
        Commands::Status => commands::status::run(&config),
        Commands::Fetch => commands::fetch::run(&config),
        Commands::Update => commands::update::run(&config),
        Commands::Project { subcommand } => match subcommand {
            ProjectCommands::List => commands::project::run_list(&config),
            ProjectCommands::Add => commands::project::run_add(&mut config),
            ProjectCommands::Remove { path } => commands::project::run_remove(&mut config, path),
        },
        Commands::Server { subcommand } => match subcommand {
            ServerCommands::List => commands::server::run_list(&config),
            ServerCommands::Add => commands::server::run_add(&mut config),
            ServerCommands::Remove { alias } => commands::server::run_remove(&mut config, alias),
        },
    }
}
