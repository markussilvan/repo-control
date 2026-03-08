use std::io::{self, BufRead, Write};

use crate::config::{ConfigManager, Server};

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
