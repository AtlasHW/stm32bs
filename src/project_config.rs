use anyhow::{bail, Result};
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::path::PathBuf;

use crate::ProjectType;

pub const PROJECT_CONFIG_FILE_NAME: &str = ".stm32bs.toml";

#[derive(Deserialize, Serialize, Debug, PartialEq, Default, Clone)]
pub struct ProjectConfig {
    pub project: Option<HashMap<String, toml::Value>>,
    pub peripheral: Option<HashMap<String, toml::Value>>,
    pub pinmap: Option<HashMap<String, toml::Value>>,
    pub driver: Option<HashMap<String, toml::Value>>,
    pub middleware: Option<HashMap<String, toml::Value>>,
}

impl TryFrom<String> for ProjectConfig {
    type Error = toml::de::Error;

    fn try_from(contents: String) -> Result<Self, Self::Error> {
        let config: Self = toml::from_str(&contents)?;
        Ok(config)
    }
}

impl ProjectConfig {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let config = match fs::read_to_string(path) {
            Ok(contents) => Self::try_from(contents)?,
            Err(e) => match e.kind() {
                ErrorKind::NotFound => Self::default(),
                _ => anyhow::bail!(e),
            },
        };
        Ok(config)
    }
}

pub fn check_config_file() -> Result<PathBuf> {
    let mut search_path = env::current_dir()?;
    loop {
        let config_file = search_path.join(PROJECT_CONFIG_FILE_NAME);
        if config_file.exists() {
            return Ok(config_file);
        }
        if let Some(path) = search_path.parent() {
            search_path = path.to_path_buf();
        } else {
            break;
        }
    }
    bail!("Project config file not found!");
}

pub fn write_project_config_file(
    project_path: impl AsRef<Path>,
    project_type: ProjectType,
    // BSP settings
) -> Result<()> {
    let mut config = ProjectConfig::default();
    config.project = Some(HashMap::from([(
        "project_type".to_string(),
        toml::Value::String(project_type.to_string()),
    )]));
    let config_file = project_path.as_ref().join(PROJECT_CONFIG_FILE_NAME);
    let toml_string = toml::to_string(&config)?;
    fs::write(config_file, toml_string)?;
    Ok(())
}
