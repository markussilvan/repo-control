use crate::config::ConfigManager;
use crate::git::run_git;

pub fn run(config: &ConfigManager) {
    let projects = match config.projects_config.as_ref() {
        Some(pc) => &pc.projects,
        None => {
            eprintln!("Projects config not loaded");
            return;
        }
    };

    // Phase 1: fetch all
    for project in projects {
        println!("Fetching project '{}'", project.name);
        let dir = config.project_dir(project);
        if let Err(e) = run_git(&dir, &["fetch"]) {
            println!("Fetching project '{}' failed: {}", project.name, e);
            return;
        }
    }

    // Phase 2: merge all
    for project in projects {
        println!("Updating project '{}'", project.name);
        let dir = config.project_dir(project);
        if let Err(e) = run_git(&dir, &["merge"]) {
            println!("Updating project '{}' failed: {}", project.name, e);
            return;
        }
    }
}
