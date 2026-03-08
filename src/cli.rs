use clap::{Parser, Subcommand};

use crate::error::ProjectStatus;

#[derive(Parser)]
#[command(name = "repo", about = "Git workarea management tool")]
pub struct Cli {
    #[arg(short, long, global = true)]
    pub debug: bool,

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
}

pub fn show_status_table(statuses: &[(String, ProjectStatus)]) {
    println!("+{}+{}+", "-".repeat(42), "-".repeat(22));
    println!("| {:<40} | {:<20} |", "Project", "Status");
    println!("+{}+{}+", "-".repeat(42), "-".repeat(22));
    for (name, status) in statuses {
        println!("| {:<40} | {:<20} |", name, status);
    }
    println!("+{}+{}+", "-".repeat(42), "-".repeat(22));
}
