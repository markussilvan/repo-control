use std::io::{self, BufRead, Write};

use crate::config::{ConfigManager, Project};

pub fn run_list(config: &ConfigManager) {
    let projects = match config.projects_config.as_ref() {
        Some(pc) => &pc.projects,
        None => {
            eprintln!("Projects config not loaded");
            return;
        }
    };
    if projects.is_empty() {
        println!("No projects configured.");
        return;
    }
    println!(
        "{:<40} {:<20} {:<40} {}",
        "Name", "Server Alias", "Git Path", "Local Path"
    );
    println!("{}", "-".repeat(110));
    for p in projects {
        println!(
            "{:<40} {:<20} {:<40} {}",
            p.name, p.git_server_alias, p.git_path, p.path
        );
    }
}

pub fn run_add(config: &mut ConfigManager) {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let name = prompt(&stdin, &stdout, "Project name: ");
    let server_alias = prompt(&stdin, &stdout, "Server alias: ");
    let git_path = prompt(&stdin, &stdout, "Git path: ");
    let local_path = prompt(&stdin, &stdout, "Local path: ");

    if let Err(e) = config.get_server_url(&server_alias) {
        eprintln!("Invalid server alias '{}': {}", server_alias, e);
        return;
    }

    let project = Project {
        name,
        git_server_alias: server_alias,
        git_path,
        path: local_path,
    };
    let pc = match config.projects_config.as_mut() {
        Some(pc) => pc,
        None => {
            eprintln!("Projects config not loaded");
            return;
        }
    };
    pc.projects.push(project);

    if let Err(e) = config.save_projects_config() {
        eprintln!("Failed to save projects config: {}", e);
        return;
    }
    println!("Project added.");
}

pub fn run_remove(config: &mut ConfigManager, path: Option<String>) {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let target_path = match path {
        Some(p) => p,
        None => prompt(&stdin, &stdout, "Local path of project to remove: "),
    };

    let pc = match config.projects_config.as_mut() {
        Some(pc) => pc,
        None => {
            eprintln!("Projects config not loaded");
            return;
        }
    };
    let original_len = pc.projects.len();
    pc.projects.retain(|p| p.path != target_path);
    if pc.projects.len() == original_len {
        eprintln!("No project found with local path '{}'", target_path);
        return;
    }

    if let Err(e) = config.save_projects_config() {
        eprintln!("Failed to save projects config: {}", e);
        return;
    }
    println!("Project with path '{}' removed.", target_path);
}

fn prompt(stdin: &io::Stdin, stdout: &io::Stdout, message: &str) -> String {
    let mut out = stdout.lock();
    write!(out, "{}", message).unwrap();
    out.flush().unwrap();
    drop(out);
    let mut line = String::new();
    stdin.lock().read_line(&mut line).unwrap_or(0);
    line.trim().to_string()
}
