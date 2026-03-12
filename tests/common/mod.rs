use std::path::Path;
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
}
