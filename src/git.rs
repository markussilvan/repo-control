use std::path::Path;
use std::process::Command;

use tracing::{debug, info, warn};

use crate::error::{ProjectStatus, RepoError};

pub fn run_git(dir: &Path, args: &[&str]) -> Result<Vec<u8>, RepoError> {
    debug!("Running git {:?} in {}", args, dir.display());
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .map_err(|e| RepoError::Git {
            path: dir.display().to_string(),
            message: format!("Failed to spawn git: {}", e),
        })?;

    if output.status.success() {
        Ok(output.stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(RepoError::Git {
            path: dir.display().to_string(),
            message: stderr,
        })
    }
}

pub fn clone(url: &str, dest_name: &str, working_dir: &Path) -> Result<(), RepoError> {
    info!("Cloning '{}' into '{}'", url, dest_name);
    let output = Command::new("git")
        .args(["clone", url, dest_name])
        .current_dir(working_dir)
        .output()
        .map_err(|e| RepoError::Git {
            path: working_dir.display().to_string(),
            message: format!("Failed to spawn git clone: {}", e),
        })?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(RepoError::Git {
            path: working_dir.display().to_string(),
            message: stderr,
        })
    }
}

pub fn check_project_status(dir: &Path) -> ProjectStatus {
    // Step 1: check for working-tree changes
    let stdout = match run_git(dir, &["status", "--porcelain"]) {
        Err(RepoError::Git { .. }) => return ProjectStatus::Unknown,
        Err(_) => return ProjectStatus::Uninitialized,
        Ok(out) => out,
    };

    if !stdout.is_empty() {
        return ProjectStatus::Changes;
    }

    // Step 2: check sync with upstream
    match run_git(
        dir,
        &["rev-list", "--left-right", "--count", "@...HEAD@{upstream}"],
    ) {
        Ok(out) => {
            let s = String::from_utf8_lossy(&out);
            let parts: Vec<&str> = s.split_whitespace().collect();
            if parts.len() == 2 {
                let ahead: u64 = parts[0].parse().unwrap_or(0);
                let behind: u64 = parts[1].parse().unwrap_or(0);
                match (ahead, behind) {
                    (a, b) if a != 0 && b != 0 => ProjectStatus::OutOfSync,
                    (a, _) if a != 0 => ProjectStatus::Ahead,
                    (_, b) if b != 0 => ProjectStatus::Behind,
                    _ => ProjectStatus::Clean,
                }
            } else {
                warn!("Unexpected rev-list output: {:?}", s);
                ProjectStatus::Unknown
            }
        }
        Err(e) => {
            warn!("rev-list failed for {}: {}", dir.display(), e);
            ProjectStatus::Unknown
        }
    }
}
