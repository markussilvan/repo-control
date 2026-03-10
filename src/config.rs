use std::fs;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::error::RepoError;

#[cfg(unix)]
fn device_id(path: &Path) -> Option<u64> {
    use std::os::unix::fs::MetadataExt;
    Some(fs::metadata(path).ok()?.dev())
}

#[cfg(windows)]
fn device_id(_path: &Path) -> Option<u64> {
    // volume_serial_number() requires an unstable feature (rust-lang/rust#63010).
    // Returning a constant disables cross-volume boundary detection, but traversal
    // still terminates correctly when parent == path at the filesystem root.
    Some(0)
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Server {
    pub alias: String,
    pub server: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Project {
    pub name: String,
    pub git_server_alias: String,
    pub git_path: String,
    pub path: String,
}

impl Project {
    pub fn is_root(&self) -> bool {
        self.path.is_empty() || self.path == "."
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LocalConfig {
    pub servers: Vec<Server>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectsConfig {
    pub projects: Vec<Project>,
}

fn load_yaml<T: DeserializeOwned>(yaml_str: &str) -> Result<T, RepoError> {
    let value: serde_yaml::Value = serde_yaml::from_str(yaml_str)?;
    let inner = match value {
        serde_yaml::Value::Tagged(t) => t.value,
        other => other,
    };
    serde_yaml::from_value(inner).map_err(RepoError::Yaml)
}

pub struct ConfigManager {
    pub project_root: PathBuf,
    pub local_config: Option<LocalConfig>,
    pub projects_config: Option<ProjectsConfig>,
}

impl ConfigManager {
    const LOCAL_CONFIG: &'static str = ".repo.yaml";
    const PROJECTS_CONFIG: &'static str = "projects.yaml";

    pub fn find_root(start: &Path) -> Option<PathBuf> {
        let start = start.canonicalize().ok()?;
        let start_dev = device_id(&start)?;

        let mut path = start.clone();
        loop {
            if path.join(Self::LOCAL_CONFIG).is_file() {
                return Some(path);
            }

            let parent = path.parent()?;

            // Stop at device boundary
            let parent_dev = device_id(parent)?;
            if parent_dev != start_dev {
                return None;
            }

            if parent == path {
                return None;
            }

            path = parent.to_path_buf();
        }
    }

    pub fn new() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let project_root = Self::find_root(&cwd).unwrap_or(cwd);
        ConfigManager {
            project_root,
            local_config: None,
            projects_config: None,
        }
    }

    pub fn local_config_exists(&self) -> bool {
        self.project_root.join(Self::LOCAL_CONFIG).is_file()
    }

    pub fn projects_config_exists(&self) -> bool {
        self.project_root.join(Self::PROJECTS_CONFIG).is_file()
    }

    pub fn read_local_config(&mut self) -> Result<(), RepoError> {
        let path = self.project_root.join(Self::LOCAL_CONFIG);
        debug!("Loading local config from {}", path.display());
        let contents = fs::read_to_string(&path)?;
        self.local_config = Some(load_yaml(&contents)?);
        Ok(())
    }

    pub fn read_projects_config(&mut self) -> Result<(), RepoError> {
        let path = self.project_root.join(Self::PROJECTS_CONFIG);
        debug!("Loading projects config from {}", path.display());
        let contents = fs::read_to_string(&path)?;
        self.projects_config = Some(load_yaml(&contents)?);
        Ok(())
    }

    pub fn create_local_config(&self) -> Result<(), RepoError> {
        let path = self.project_root.join(Self::LOCAL_CONFIG);
        fs::write(&path, "servers: []\n")?;
        Ok(())
    }

    pub fn create_projects_config(&self) -> Result<(), RepoError> {
        let path = self.project_root.join(Self::PROJECTS_CONFIG);
        fs::write(&path, "projects: []\n")?;
        Ok(())
    }

    pub fn save_local_config(&self) -> Result<(), RepoError> {
        let path = self.project_root.join(Self::LOCAL_CONFIG);
        let lc = self
            .local_config
            .as_ref()
            .ok_or_else(|| RepoError::Config("Local config not loaded".into()))?;
        let yaml = serde_yaml::to_string(lc)?;
        fs::write(&path, yaml)?;
        Ok(())
    }

    pub fn save_projects_config(&self) -> Result<(), RepoError> {
        let path = self.project_root.join(Self::PROJECTS_CONFIG);
        let pc = self
            .projects_config
            .as_ref()
            .ok_or_else(|| RepoError::Config("Projects config not loaded".into()))?;
        let yaml = serde_yaml::to_string(pc)?;
        fs::write(&path, yaml)?;
        Ok(())
    }

    pub fn get_server_url(&self, alias: &str) -> Result<String, RepoError> {
        let lc = self
            .local_config
            .as_ref()
            .ok_or_else(|| RepoError::Config("Local config not loaded".into()))?;
        lc.servers
            .iter()
            .find(|s| s.alias == alias)
            .map(|s| s.server.clone())
            .ok_or_else(|| RepoError::UnknownAlias(alias.to_string()))
    }

    pub fn get_git_url(&self, project: &Project) -> Result<String, RepoError> {
        let server = self.get_server_url(&project.git_server_alias)?;
        Ok(format!("{}{}", server, project.git_path))
    }

    pub fn project_dir(&self, project: &Project) -> PathBuf {
        self.project_root.join(&project.path)
    }
}
