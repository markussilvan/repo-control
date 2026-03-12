mod common;
use common::TestWorkspace;
use predicates::prelude::*;

// --- init ---

#[test]
fn init_creates_config_files() {
    let ws = TestWorkspace::new();
    ws.cmd().arg("init").assert().success();
    assert!(ws.path().join(".repo.json").exists());
    assert!(ws.path().join("projects.json").exists());
}

// --- guard: uninitialized workspace ---

#[test]
fn command_fails_without_init() {
    let ws = TestWorkspace::new();
    ws.cmd()
        .args(["server", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("repo init"));
}

// --- server list ---

#[test]
fn server_list_empty() {
    let ws = TestWorkspace::new_initialized();
    ws.cmd()
        .args(["server", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No servers configured."));
}

#[test]
fn server_list_shows_configured_servers() {
    let ws = TestWorkspace::new_initialized();
    ws.write_local_config(r#"{"servers":[{"alias":"origin","server":"ssh://git@example.com"}]}"#);
    ws.cmd()
        .args(["server", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("origin"))
        .stdout(predicate::str::contains("ssh://git@example.com"));
}

// --- project list ---

#[test]
fn project_list_empty() {
    let ws = TestWorkspace::new_initialized();
    ws.cmd()
        .args(["project", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No projects configured."));
}
