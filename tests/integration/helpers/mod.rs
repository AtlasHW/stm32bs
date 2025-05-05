use crate::helpers::project::Project;
use crate::helpers::project_builder::tempdir;

pub mod arg_builder;
pub mod prelude;
pub mod project;
pub mod project_builder;

pub fn create_template() -> Project {
    tempdir().with_default_manifest().init_git().build()
}
