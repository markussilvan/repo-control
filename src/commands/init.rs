use tracing::info;

use crate::config::ConfigManager;
use crate::git;

pub fn run(config: &mut ConfigManager) {
    if !config.projects_config_exists() {
        println!("Project configuration file doesn't exist - creating");
        if let Err(e) = config.create_projects_config() {
            println!("Creating project configuration file failed: {}", e);
            return;
        }
    } else {
        println!("Project configuration file exists");
    }

    if !config.local_config_exists() {
        println!("Local configuration file doesn't exist - creating");
        if let Err(e) = config.create_local_config() {
            println!("Creating local configuration file failed: {}", e);
            return;
        }
    } else {
        println!("Local configuration file exists");
    }

    if let Err(e) = config.read_local_config() {
        println!("Reading local configuration failed: {}", e);
        return;
    }
    if let Err(e) = config.read_projects_config() {
        println!("Reading project configuration failed: {}", e);
        return;
    }

    let projects = config
        .projects_config
        .as_ref()
        .map(|pc| pc.projects.clone())
        .unwrap_or_default();

    for project in &projects {
        if project.is_root() {
            info!("Skipping root project: '{}'", project.name);
            continue;
        }

        let project_dir = config.project_dir(project);
        if project_dir.exists() {
            println!("Project '{}' already exists", project.name);
            continue;
        }

        let url = match config.get_git_url(project) {
            Ok(u) => u,
            Err(e) => {
                println!("Cloning '{}' failed. Configuration error: {}", project.name, e);
                return;
            }
        };

        info!("Cloning project '{}' from '{}'", project.name, url);
        if let Err(e) = git::clone(&url, &project.path, &config.project_root) {
            println!("Error cloning project '{}': {}", project.name, e);
            return;
        }
    }

    println!("Done");
}
