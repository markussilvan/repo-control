use std::fs::{File, OpenOptions};
use std::path::Path;
use std::process::Stdio;
use tempfile::TempDir;

pub struct TestWorkspace {
    pub dir: TempDir,
}

impl TestWorkspace {
    /// Empty temp dir — no config files.
    pub fn new() -> Self {
        Self { dir: tempfile::tempdir().unwrap() }
    }

    /// Temp dir pre-populated with empty config files (skips testing `init` itself).
    pub fn new_initialized() -> Self {
        let ws = Self::new();
        std::fs::write(ws.path().join(".repo.json"), "{\"servers\":[]}\n").unwrap();
        std::fs::write(ws.path().join("projects.json"), "{\"projects\":[]}\n").unwrap();
        ws
    }

    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    /// A `repo` Command already `current_dir`'d into the workspace.
    pub fn cmd(&self) -> assert_cmd::Command {
        let mut cmd = assert_cmd::Command::cargo_bin("repo").unwrap();
        cmd.current_dir(self.path());
        cmd
    }

    /// Write (or overwrite) .repo.json with the given content.
    pub fn write_local_config(&self, json: &str) {
        std::fs::write(self.path().join(".repo.json"), json).unwrap();
    }

    /// Write (or overwrite) projects.json with the given content.
    pub fn write_projects_config(&self, json: &str) {
        std::fs::write(self.path().join("projects.json"), json).unwrap();
    }

    /// Create a subdirectory `name/` containing a clean git repo synced with a
    /// local bare remote.  After this call `git status --porcelain` returns
    /// empty output and the upstream tracking ref is in sync, so the binary
    /// will report CLEAN for a project whose "path" is `name`.
    pub fn make_clean_repo(&self, name: &str) {
        let bare = self.path().join(format!("{}.git", name));
        let local = self.path().join(name);
        git_cmd(&["init", "--bare", "-b", "master", bare.to_str().unwrap()]);
        git_cmd(&["clone", bare.to_str().unwrap(), local.to_str().unwrap()]);
        git_cmd_in(&local, &["config", "user.email", "test@example.com"]);
        git_cmd_in(&local, &["config", "user.name", "Test"]);
        git_cmd_in(&local, &["commit", "--allow-empty", "-m", "init"]);
        git_cmd_in(&local, &["push", "-u", "origin", "HEAD"]);
    }
}

/// Opens (or creates) `target/test-git.log` in append mode.
/// All git helper output is redirected here instead of the terminal.
fn open_log() -> File {
    std::fs::create_dir_all("target").unwrap();
    OpenOptions::new()
        .create(true)
        .append(true)
        .open("target/test-git.log")
        .unwrap()
}

fn git_cmd(args: &[&str]) {
    let log = open_log();
    let status = std::process::Command::new("git")
        .args(args)
        .stdout(Stdio::from(log.try_clone().unwrap()))
        .stderr(Stdio::from(log))
        .status()
        .unwrap();
    assert!(status.success(), "git {:?} failed", args);
}

fn git_cmd_in(dir: &Path, args: &[&str]) {
    let log = open_log();
    let status = std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .stdout(Stdio::from(log.try_clone().unwrap()))
        .stderr(Stdio::from(log))
        .status()
        .unwrap();
    assert!(status.success(), "git {:?} in {:?} failed", args, dir);
}
