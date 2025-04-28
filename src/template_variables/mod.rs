mod authors;
mod project_dir;
pub mod project_name;

use indexmap::IndexMap;
use serde::Deserialize;

pub use authors::{get_authors, Authors};

pub use project_dir::ProjectDir;

#[derive(Deserialize, Debug, PartialEq)]
struct TemplateValuesToml {
    pub(crate) values: IndexMap<String, toml::Value>,
}
