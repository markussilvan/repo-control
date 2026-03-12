use std::fs;
use std::io::{self, BufRead, Write};

use crate::config::{ConfigManager, Project};
use crate::git;

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
        name: name.clone(),
        git_server_alias: server_alias,
        git_path,
        path: local_path.clone(),
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

    let (autoignore, autocommit) = config
        .local_config
        .as_ref()
        .map(|lc| (lc.autoignore, lc.autocommit))
        .unwrap_or((false, false));

    if autoignore {
        let gitignore_path = config.project_root.join(".gitignore");
        let existing = fs::read_to_string(&gitignore_path).unwrap_or_default();
        let already_present = existing.lines().any(|l| l == local_path);
        if !already_present {
            let mut content = existing;
            if !content.is_empty() && !content.ends_with('\n') {
                content.push('\n');
            }
            content.push_str(&local_path);
            content.push('\n');
            if let Err(e) = fs::write(&gitignore_path, &content) {
                eprintln!("Warning: failed to update .gitignore: {}", e);
            }
        }
    }

    if autocommit {
        let mut files = vec!["projects.json"];
        if autoignore {
            files.push(".gitignore");
        }
        let mut add_args = vec!["add"];
        add_args.extend_from_slice(&files);
        if let Err(e) = git::run_git(&config.project_root, &add_args) {
            eprintln!("Warning: git add failed: {}", e);
        } else {
            let msg = format!("repo: add project {}", name);
            if let Err(e) = git::run_git(&config.project_root, &["commit", "-m", &msg]) {
                eprintln!("Warning: git commit failed: {}", e);
            }
        }
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

    let autocommit = config
        .local_config
        .as_ref()
        .map(|lc| lc.autocommit)
        .unwrap_or(false);

    if autocommit {
        if let Err(e) = git::run_git(&config.project_root, &["add", "projects.json"]) {
            eprintln!("Warning: git add failed: {}", e);
        } else {
            let msg = format!("repo: remove project {}", target_path);
            if let Err(e) = git::run_git(&config.project_root, &["commit", "-m", &msg]) {
                eprintln!("Warning: git commit failed: {}", e);
            }
        }
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
