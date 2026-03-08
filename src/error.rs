use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepoError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Git error in '{path}': {message}")]
    Git { path: String, message: String },

    #[error("Unknown server alias: {0}")]
    UnknownAlias(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectStatus {
    Clean,
    Changes,
    Uninitialized,
    Unknown,
    Ahead,
    Behind,
    OutOfSync,
}

impl std::fmt::Display for ProjectStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ProjectStatus::Clean => "CLEAN",
            ProjectStatus::Changes => "CHANGES",
            ProjectStatus::Uninitialized => "UNINITIALIZED",
            ProjectStatus::Unknown => "UNKNOWN",
            ProjectStatus::Ahead => "AHEAD",
            ProjectStatus::Behind => "BEHIND",
            ProjectStatus::OutOfSync => "OUT_OF_SYNC",
        };
        f.pad(s)
    }
}
