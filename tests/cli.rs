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

// --- version validation ---

#[test]
fn local_config_missing_version_is_rejected() {
    let ws = TestWorkspace::new_initialized();
    ws.write_local_config(r#"{"servers":[]}"#);
    ws.cmd()
        .args(["server", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("version"));
}

#[test]
fn local_config_future_version_is_rejected() {
    let ws = TestWorkspace::new_initialized();
    ws.write_local_config(r#"{"version":999,"servers":[]}"#);
    ws.cmd()
        .args(["server", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("version"));
}

#[test]
fn projects_config_missing_version_is_rejected() {
    let ws = TestWorkspace::new_initialized();
    ws.write_projects_config(r#"{"projects":[]}"#);
    ws.cmd()
        .args(["project", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("version"));
}

#[test]
fn projects_config_future_version_is_rejected() {
    let ws = TestWorkspace::new_initialized();
    ws.write_projects_config(r#"{"version":999,"projects":[]}"#);
    ws.cmd()
        .args(["project", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("version"));
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
    ws.write_local_config(r#"{"version":1,"servers":[{"alias":"origin","url":"ssh://git@example.com"}]}"#);
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

// --- server edit ---

#[test]
fn server_edit_updates_url() {
    let ws = TestWorkspace::new_initialized();
    ws.write_local_config(r#"{"version":1,"servers":[{"alias":"origin","url":"ssh://git@example.com"}]}"#);
    ws.cmd()
        .args(["server", "edit", "origin"])
        .write_stdin("\nssh://git@new.example.com\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Server updated."));
    let config = std::fs::read_to_string(ws.path().join(".repo.json")).unwrap();
    assert!(config.contains("ssh://git@new.example.com"));
}

#[test]
fn server_edit_updates_alias() {
    let ws = TestWorkspace::new_initialized();
    ws.write_local_config(r#"{"version":1,"servers":[{"alias":"origin","url":"ssh://git@example.com"}]}"#);
    ws.cmd()
        .args(["server", "edit", "origin"])
        .write_stdin("upstream\n\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Server updated."));
    let config = std::fs::read_to_string(ws.path().join(".repo.json")).unwrap();
    assert!(config.contains("upstream"));
    assert!(!config.contains("\"origin\""));
}

// --- server remove ---

#[test]
fn server_remove_updates_config() {
    let ws = TestWorkspace::new_initialized();
    ws.write_local_config(r#"{"version":1,"servers":[{"alias":"origin","url":"ssh://git@example.com"}]}"#);
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
        r#"{"version":1,"projects":[
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

// --- project add ---

#[test]
fn project_add_writes_to_config() {
    let ws = TestWorkspace::new_initialized();
    ws.write_local_config(r#"{"version":1,"servers":[{"alias":"origin","url":"ssh://git@example.com"}]}"#);
    ws.cmd()
        .args(["project", "add"])
        .write_stdin("myproject\norigin\n/myproject.git\nmyproject\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Project added."));
    let config = std::fs::read_to_string(ws.path().join("projects.json")).unwrap();
    assert!(config.contains("myproject"));
    assert!(config.contains("origin"));
    assert!(config.contains("/myproject.git"));
}

// --- project add: invalid alias ---

#[test]
fn project_add_rejects_unknown_server_alias() {
    let ws = TestWorkspace::new_initialized();
    ws.cmd()
        .args(["project", "add"])
        .write_stdin("myproject\nnonexistent\n/myproject.git\nmyproject\n")
        .assert()
        .success()
        .stderr(predicate::str::contains("Invalid server alias"));
    let config = std::fs::read_to_string(ws.path().join("projects.json")).unwrap();
    assert!(!config.contains("myproject"));
}

// --- project create ---

#[test]
fn project_create_makes_dir_and_config() {
    let ws = TestWorkspace::new_initialized();
    ws.write_local_config(r#"{"version":1,"servers":[{"alias":"origin","url":"ssh://git@example.com"}]}"#);
    ws.cmd()
        .args(["project", "create"])
        .write_stdin("myproject\norigin\n/myproject.git\nmyproject\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Project created."));
    assert!(ws.path().join("myproject").is_dir());
    assert!(ws.path().join("myproject/.git").is_dir());
    let config = std::fs::read_to_string(ws.path().join("projects.json")).unwrap();
    assert!(config.contains("myproject"));
}

#[test]
fn project_create_rejects_unknown_server_alias() {
    let ws = TestWorkspace::new_initialized();
    ws.cmd()
        .args(["project", "create"])
        .write_stdin("myproject\nnonexistent\n/myproject.git\nmyproject\n")
        .assert()
        .success()
        .stderr(predicate::str::contains("Invalid server alias"));
    assert!(!ws.path().join("myproject").exists());
}

#[test]
fn project_create_fails_if_dir_exists() {
    let ws = TestWorkspace::new_initialized();
    ws.write_local_config(r#"{"version":1,"servers":[{"alias":"origin","url":"ssh://git@example.com"}]}"#);
    std::fs::create_dir(ws.path().join("myproject")).unwrap();
    ws.cmd()
        .args(["project", "create"])
        .write_stdin("myproject\norigin\n/myproject.git\nmyproject\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

// --- project remove ---

#[test]
fn project_remove_updates_config() {
    let ws = TestWorkspace::new_initialized();
    ws.write_projects_config(
        r#"{"version":1,"projects":[{"name":"myproject","git_server_alias":"origin","git_path":"/myproject.git","path":"myproject"}]}"#,
    );
    ws.cmd()
        .args(["project", "remove", "myproject"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Project with path 'myproject' removed."));
    let config = std::fs::read_to_string(ws.path().join("projects.json")).unwrap();
    assert!(!config.contains("myproject"));
}

// --- status ---

#[test]
fn status_unknown_for_missing_project_dir() {
    let ws = TestWorkspace::new_initialized();
    ws.write_projects_config(
        r#"{"version":1,"projects":[{"name":"myproject","git_server_alias":"origin","git_path":"/foo.git","path":"myproject"}]}"#,
    );
    // "myproject/" directory does not exist
    ws.cmd()
        .args(["status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("myproject"))
        .stdout(predicate::str::contains("UNKNOWN"))
        .stdout(predicate::str::contains("0/1 projects cloned."));
}

#[test]
fn status_summary_mixed_cloned_and_missing() {
    let ws = TestWorkspace::new_initialized();
    ws.make_clean_repo("present");
    ws.write_projects_config(
        r#"{"version":1,"projects":[
            {"name":"present","git_server_alias":"origin","git_path":"/present.git","path":"present"},
            {"name":"missing","git_server_alias":"origin","git_path":"/missing.git","path":"missing"}
        ]}"#,
    );
    ws.cmd()
        .args(["status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1/2 projects cloned."))
        .stdout(predicate::str::contains("1/2 projects clean."));
}

#[test]
fn status_clean_for_synced_git_repo() {
    let ws = TestWorkspace::new_initialized();
    ws.make_clean_repo("myproject");
    ws.write_projects_config(
        r#"{"version":1,"projects":[{"name":"myproject","git_server_alias":"origin","git_path":"/foo.git","path":"myproject"}]}"#,
    );
    ws.cmd()
        .args(["status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("myproject"))
        .stdout(predicate::str::contains("CLEAN"))
        .stdout(predicate::str::contains("All 1 projects cloned and clean."));
}
