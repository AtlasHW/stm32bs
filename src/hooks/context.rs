use std::path::PathBuf;

use liquid_core::Object;

#[derive(Debug)]
pub struct RhaiHooksContext {
    pub liquid_object: Object,
    pub allow_commands: bool,
    pub silent: bool,
    pub working_directory: PathBuf,
    pub destination_directory: PathBuf,
}
