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

// --- server add ---

#[test]
fn server_add_writes_to_config() {
    let ws = TestWorkspace::new_initialized();
    ws.cmd()
        .args(["server", "add"])
        .write_stdin("origin\nssh://git@example.com\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Server added."));
    let config = std::fs::read_to_string(ws.path().join(".repo.json")).unwrap();
    assert!(config.contains("origin"));
    assert!(config.contains("ssh://git@example.com"));
}

// --- server remove ---

#[test]
fn server_remove_updates_config() {
    let ws = TestWorkspace::new_initialized();
    ws.write_local_config(r#"{"servers":[{"alias":"origin","server":"ssh://git@example.com"}]}"#);
    ws.cmd()
        .args(["server", "remove", "origin"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Server 'origin' removed."));
    let config = std::fs::read_to_string(ws.path().join(".repo.json")).unwrap();
    assert!(!config.contains("origin"));
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

#[test]
fn project_list_shows_multiple_projects() {
    let ws = TestWorkspace::new_initialized();
    ws.write_projects_config(
        r#"{"projects":[
            {"name":"alpha","git_server_alias":"origin","git_path":"/alpha.git","path":"alpha"},
            {"name":"beta","git_server_alias":"backup","git_path":"/beta.git","path":"beta"}
        ]}"#,
    );
    ws.cmd()
        .args(["project", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("alpha"))
        .stdout(predicate::str::contains("beta"));
}

// --- status ---

#[test]
fn status_unknown_for_missing_project_dir() {
    let ws = TestWorkspace::new_initialized();
    ws.write_projects_config(
        r#"{"projects":[{"name":"myproject","git_server_alias":"origin","git_path":"/foo.git","path":"myproject"}]}"#,
    );
    // "myproject/" directory does not exist
    ws.cmd()
        .args(["status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("myproject"))
        .stdout(predicate::str::contains("UNKNOWN"));
}

#[test]
fn status_clean_for_synced_git_repo() {
    let ws = TestWorkspace::new_initialized();
    ws.make_clean_repo("myproject");
    ws.write_projects_config(
        r#"{"projects":[{"name":"myproject","git_server_alias":"origin","git_path":"/foo.git","path":"myproject"}]}"#,
    );
    ws.cmd()
        .args(["status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("myproject"))
        .stdout(predicate::str::contains("CLEAN"));
}
