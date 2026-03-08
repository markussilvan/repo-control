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

    for project in projects {
        println!("Fetching project '{}'", project.name);
        let dir = config.project_dir(project);
        if let Err(e) = run_git(&dir, &["fetch"]) {
            println!("Fetching project '{}' failed: {}", project.name, e);
            return;
        }
    }
}
