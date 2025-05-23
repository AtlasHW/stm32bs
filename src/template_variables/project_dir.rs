use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use anyhow::bail;
use console::style;

//use crate::template_variables::project_name::sanitize_project_name;
use crate::user_parsed_input::UserParsedInput;
use log::warn;

/// Stores user inputted name and provides convenience methods
/// for handling casing.
#[derive(Debug, PartialEq)]
pub struct ProjectDir(PathBuf);

impl AsRef<Path> for ProjectDir {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

impl Display for ProjectDir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.display().fmt(f)
    }
}

impl TryFrom<(&String, &UserParsedInput)> for ProjectDir {
    type Error = anyhow::Error;

    fn try_from(
        (project_name_input, user_parsed_input): (&String, &UserParsedInput),
    ) -> Result<Self, Self::Error> {
        let base_path = user_parsed_input.destination();
        let name = project_name_input.to_string();
        let project_dir = base_path.join(name);
        Ok(Self(project_dir))
    }
}

impl ProjectDir {
    pub fn create(&self, overwrite: bool) -> anyhow::Result<()> {
        let path = self.0.as_path();
        if path.exists() & overwrite {
            std::fs::remove_dir_all(path)?;
            warn!(
                "{}",
                style(format!("Overwrite existing directory: {}", path.display()))
                    .bold()
                    .yellow()
            );
        }
        if path.exists() {
            bail!(
                "⛔ {}",
                style("Target directory already exists, aborting!")
                    .bold()
                    .blue()
            );
        }
        std::fs::create_dir(&self.0)?;
        Ok(())
    }
}
