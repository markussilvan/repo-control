use std::io::{self, BufRead, Write};

use crate::config::{ConfigManager, Server};
use crate::git;

pub fn run_list(config: &ConfigManager) {
    let servers = match config.local_config.as_ref() {
        Some(lc) => &lc.servers,
        None => {
            eprintln!("Local config not loaded");
            return;
        }
    };
    if servers.is_empty() {
        println!("No servers configured.");
        return;
    }
    println!("{:<30} {}", "Alias", "URL");
    println!("{}", "-".repeat(70));
    for s in servers {
        println!("{:<30} {}", s.alias, s.server);
    }
}

pub fn run_add(config: &mut ConfigManager) {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let alias = prompt(&stdin, &stdout, "Server alias: ");
    let url = prompt(&stdin, &stdout, "Server URL: ");

    let lc = match config.local_config.as_mut() {
        Some(lc) => lc,
        None => {
            eprintln!("Local config not loaded");
            return;
        }
    };
    if lc.servers.iter().any(|s| s.alias == alias) {
        eprintln!("Server alias '{}' already exists.", alias);
        return;
    }
    lc.servers.push(Server { alias, server: url });

    if let Err(e) = config.save_local_config() {
        eprintln!("Failed to save local config: {}", e);
        return;
    }
    println!("Server added.");
}

pub fn run_remove(config: &mut ConfigManager, alias: Option<String>) {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let target = match alias {
        Some(a) => a,
        None => prompt(&stdin, &stdout, "Alias of server to remove: "),
    };

    let lc = match config.local_config.as_mut() {
        Some(lc) => lc,
        None => {
            eprintln!("Local config not loaded");
            return;
        }
    };
    let original_len = lc.servers.len();
    lc.servers.retain(|s| s.alias != target);
    if lc.servers.len() == original_len {
        eprintln!("No server found with alias '{}'", target);
        return;
    }

    if let Err(e) = config.save_local_config() {
        eprintln!("Failed to save local config: {}", e);
        return;
    }
    println!("Server '{}' removed.", target);
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

fn prompt_or_keep(stdin: &io::Stdin, stdout: &io::Stdout, message: &str, current: &str) -> String {
    let input = prompt(stdin, stdout, &format!("{} [{}]: ", message, current));
    if input.is_empty() {
        current.to_string()
    } else {
        input
    }
}

pub fn run_edit(config: &mut ConfigManager, alias: Option<String>) {
    let stdin = io::stdin();
    let stdout = io::stdout();

    let target = match alias {
        Some(a) => a,
        None => prompt(&stdin, &stdout, "Alias of server to edit: "),
    };

    let (current_alias, current_url) = {
        let lc = match config.local_config.as_ref() {
            Some(lc) => lc,
            None => {
                eprintln!("Local config not loaded");
                return;
            }
        };
        match lc.servers.iter().find(|s| s.alias == target) {
            Some(s) => (s.alias.clone(), s.server.clone()),
            None => {
                eprintln!("No server found with alias '{}'", target);
                return;
            }
        }
    };

    let new_alias = prompt_or_keep(&stdin, &stdout, "New alias", &current_alias);
    let new_url = prompt_or_keep(&stdin, &stdout, "New URL", &current_url);

    if new_alias == current_alias && new_url == current_url {
        println!("No changes made.");
        return;
    }

    let alias_changed = new_alias != current_alias;

    {
        let lc = match config.local_config.as_mut() {
            Some(lc) => lc,
            None => {
                eprintln!("Local config not loaded");
                return;
            }
        };
        if alias_changed && lc.servers.iter().any(|s| s.alias == new_alias) {
            eprintln!("Server alias '{}' already exists.", new_alias);
            return;
        }
        if let Some(s) = lc.servers.iter_mut().find(|s| s.alias == target) {
            s.alias = new_alias.clone();
            s.server = new_url.clone();
        }
    }

    if alias_changed {
        if let Some(pc) = config.projects_config.as_mut() {
            for p in pc.projects.iter_mut() {
                if p.git_server_alias == current_alias {
                    p.git_server_alias = new_alias.clone();
                }
            }
        }
        if let Err(e) = config.save_projects_config() {
            eprintln!("Failed to save projects config: {}", e);
            return;
        }
    }

    if let Err(e) = config.save_local_config() {
        eprintln!("Failed to save local config: {}", e);
        return;
    }

    println!("Server updated.");

    if new_url == current_url {
        return;
    }

    let affected_projects: Vec<_> = config
        .projects_config
        .as_ref()
        .map(|pc| {
            pc.projects
                .iter()
                .filter(|p| p.git_server_alias == new_alias)
                .filter(|p| config.project_dir(p).is_dir())
                .cloned()
                .collect()
        })
        .unwrap_or_default();

    if affected_projects.is_empty() {
        return;
    }

    let answer = prompt(
        &stdin,
        &stdout,
        "Update git remote URLs for affected projects? [y/N]: ",
    );
    if !answer.eq_ignore_ascii_case("y") {
        return;
    }

    for p in &affected_projects {
        let dir = config.project_dir(p);
        let old_url = match git::run_git(&dir, &["config", "--get", "remote.origin.url"]) {
            Ok(out) => String::from_utf8_lossy(&out).trim().to_string(),
            Err(_) => String::from("<unknown>"),
        };
        let new_remote_url = format!("{}{}", new_url, p.git_path);
        println!("  {}: {} -> {}", p.name, old_url, new_remote_url);
        if let Err(e) = git::run_git(&dir, &["remote", "set-url", "origin", &new_remote_url]) {
            eprintln!("  Warning: failed to update remote for '{}': {}", p.name, e);
        }
    }
}
