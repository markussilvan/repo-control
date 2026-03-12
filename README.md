# Repo Control

A CLI tool for managing multiple Git repositories at once. Define a workspace
with a list of projects and Git servers, then clone, inspect, and update all
repos with a single command.

## How It Works

Two configuration files are needed. A `.repo.json` for your local
configuration and `projects.json` to define your projects.

Place a `.repo.json` file in your workspace root to mark it and define used
Git servers. Add a `projects.json` to list the repositories you want to manage.
Run `repo` from anywhere inside the workspace.

## Configuration

**.repo.json**, defines used Git servers (machine-local, not checked in):

```json
{
  "servers": [
    { "alias": "github", "server": "ssh://git@github.com" },
    { "alias": "homelab", "server": "https://homelab.local" }
  ]
}
```

**projects.json**, defines the repositories to manage:

```json
{
  "projects": [
    {
      "name": "projects",
      "git_server_alias": "homelab",
      "git_path": "/myorg/projects.git",
      "path": ""
    },
    {
      "name": "shared-lib",
      "git_server_alias": "github",
      "git_path": "/team/shared-lib.git",
      "path": "team/shared-lib" },
    {
      "name": "my-app",
      "git_server_alias": "homelab",
      "git_path": "/myorg/my-app.git",
      "path": "my-app" },
    {
      "name": "my-other-project",
      "git_server_alias": "homelab",
      "git_path": "/myorg/my-other-project.git",
      "path": "my-other-project"
    }
  ]
}
```

The `path` field is relative to the workspace root. A project with
`path: ""` or `path: "."` is treated as the root project and skipped
during clone/init. The master project can contain, for example, the
`projects.json` and a `.gitignore` among other things.

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

Manages the list of projects in `projects.json`.

```
repo project list              # List all configured projects
repo project add               # Add a new project interactively
repo project remove [path]     # Remove a project by local path (prompted if omitted)
```

### `repo server`

Manages Git server aliases in `.repo.json`.

```
repo server list               # List all configured server aliases
repo server add                # Add a new server alias interactively
repo server remove [alias]     # Remove a server alias (prompted if omitted)
repo server edit [alias]       # Edit a server alias and/or URL (prompted if omitted)
```

`repo server edit` updates the alias and/or URL of an existing server. Press
Enter to keep the current value for either field. If the server URL changed,
you will be asked whether to update the `origin` remote in all affected local
repositories automatically.
