# Repo Control

A CLI tool for managing multiple Git repositories at once. Define a workspace
with a list of projects and Git servers, then clone, inspect, and update all
repos with a single command.

## How It Works

Two configuration files are needed. A `.repo.yaml` for your local
configuration and `projects.yaml` to define your projects.

Place a `.repo.yaml` file in your workspace root to mark it and define used
Git servers. Add a `projects.yaml` to list the repositories you want to manage.
Run `repo` from anywhere inside the workspace.

## Configuration

**.repo.yaml**, defines used Git servers (machine-local, not checked in):

```yaml
servers:
  - alias: github
    server: ssh://git@github.com
  - alias: homelab
    server: https://homelab.local
```

**projects.yaml**, defines the repositories to manage:

```yaml
projects:
  - name: projects
    git_server_alias: homelab
    git_path: /myorg/projects.git
    path: ''
  - name: shared-lib
    git_server_alias: github
    git_path: /team/shared-lib.git
    path: team/shared-lib
  - name: my-app
    git_server_alias: homelab
    git_path: /myorg/my-app.git
    path: my-app
  - name: my-other-project
    git_server_alias: homelab
    git_path: /myorg/my-other-project.git
    path: my-other-project
```

The `path` field is relative to the workspace root. A project with
`path: ""` or `path: "."` is treated as the root project and skipped
during clone/init. The master project can contain, for example, the
`projects.yaml` and a `.gitignore` among other things.

## Commands

### `repo init`

Creates config files if they don't exist, then clones any projects
that haven't been checked out yet.

```
repo init
```

### `repo status`

Shows the status of all configured repositories in a table.

```
repo status

+------------------------------------------+----------------------+
| Project                                  | Status               |
+------------------------------------------+----------------------+
| my-app                                   | CLEAN                |
| shared-lib                               | CHANGES              |
| old-service                              | BEHIND               |
+------------------------------------------+----------------------+
```

### `repo fetch`

Fetches from remotes for all repositories.

```
repo fetch
```

### `repo update`

Fetches from all remotes, then merges updates into each repository
(two-phase: fetch-all first, then merge-all).

```
repo update
```

### `repo project`

Manages the list of projects in `projects.yaml`.

```
repo project list              # List all configured projects
repo project add               # Add a new project interactively
repo project remove [path]     # Remove a project by local path (prompted if omitted)
```

### `repo server`

Manages Git server aliases in `.repo.yaml`.

```
repo server list               # List all configured server aliases
repo server add                # Add a new server alias interactively
repo server remove [alias]     # Remove a server alias (prompted if omitted)
```

## Global Flags

| Flag            | Description          |
|-----------------|----------------------|
| `-d`, `--debug` | Enable debug logging |
