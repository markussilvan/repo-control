use clap::{Parser, Subcommand};

use crate::error::ProjectStatus;

#[derive(Parser)]
#[command(name = "repo", about = "Git workarea management tool", version = env!("CARGO_PKG_VERSION"), disable_version_flag = true)]
pub struct Cli {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    _version: (),
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize the repository
    Init,
    /// Show combined project status
    Status,
    /// Fetch from remotes of all repositories
    Fetch,
    /// Update all repositories
    Update,
    /// Manage configured projects
    Project {
        #[command(subcommand)]
        subcommand: ProjectCommands,
    },
    /// Manage configured server aliases
    Server {
        #[command(subcommand)]
        subcommand: ServerCommands,
    },
}

#[derive(Subcommand)]
pub enum ProjectCommands {
    /// Add a new project interactively
    Add,
    /// Remove a project by local path
    Remove {
        /// Local path of the project to remove (prompted if omitted)
        path: Option<String>,
    },
    /// List all configured projects
    List,
}

#[derive(Subcommand)]
pub enum ServerCommands {
    /// List all configured server aliases
    List,
    /// Add a new server alias interactively
    Add,
    /// Remove a server alias
    Remove {
        /// Alias of the server to remove (prompted if omitted)
        alias: Option<String>,
    },
    /// Edit an existing server alias and/or URL interactively
    Edit {
        /// Alias of the server to edit (prompted if omitted)
        alias: Option<String>,
    },
}

pub fn show_server_table(servers: &[(String, String)]) {
    println!("┌{}┬{}┐", "─".repeat(32), "─".repeat(52));
    println!("│ {:<30} │ {:<50} │", "Alias", "URL");
    println!("├{}┼{}┤", "─".repeat(32), "─".repeat(52));
    for (alias, url) in servers {
        println!("│ {:<30} │ {:<50} │", alias, url);
    }
    println!("└{}┴{}┘", "─".repeat(32), "─".repeat(52));
}

pub fn show_projects_table(projects: &[(String, String, String, String)]) {
    println!("┌{}┬{}┐", "─".repeat(13), "─".repeat(52));
    for (i, (name, server, git_path, local_path)) in projects.iter().enumerate() {
        if i > 0 {
            println!("├{}┼{}┤", "─".repeat(13), "─".repeat(52));
        }
        println!("│ {:<11} │ {:<50} │", "Name", name);
        println!("│ {:<11} │ {:<50} │", "Server", server);
        println!("│ {:<11} │ {:<50} │", "Git path", git_path);
        println!("│ {:<11} │ {:<50} │", "Local path", local_path);
    }
    println!("└{}┴{}┘", "─".repeat(13), "─".repeat(52));
}

pub fn show_status_table(statuses: &[(String, ProjectStatus)]) {
    println!("┌{}┬{}┐", "─".repeat(42), "─".repeat(22));
    println!("│ {:<40} │ {:<20} │", "Project", "Status");
    println!("├{}┼{}┤", "─".repeat(42), "─".repeat(22));
    for (name, status) in statuses {
        println!("│ {:<40} │ {:<20} │", name, status);
    }
    println!("└{}┴{}┘", "─".repeat(42), "─".repeat(22));
}
