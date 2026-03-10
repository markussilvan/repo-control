use tracing::info;

use crate::cli::show_status_table;
use crate::config::ConfigManager;
use crate::git::check_project_status;

pub fn run(config: &ConfigManager) {
    let projects = match config.projects_config.as_ref() {
        Some(pc) => &pc.projects,
        None => {
            eprintln!("Projects config not loaded");
            return;
        }
    };

    let statuses: Vec<(String, _)> = projects
        .iter()
        .map(|p| {
            let dir = config.project_dir(p);
            let status = check_project_status(&dir);
            info!("Project '{}' status: {}", p.name, status);
            (p.name.clone(), status)
        })
        .collect();

    show_status_table(&statuses);
}
