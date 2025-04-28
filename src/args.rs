use std::path::PathBuf;

use clap::{Args, Parser};
use std::env;

/// Styles from <https://github.com/rust-lang/cargo/blob/master/src/cargo/util/style.rs>
mod style {
    use anstyle::*;
    use clap::builder::Styles;

    const HEADER: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
    const USAGE: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
    const LITERAL: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
    const PLACEHOLDER: Style = AnsiColor::Cyan.on_default();
    const ERROR: Style = AnsiColor::Red.on_default().effects(Effects::BOLD);
    const VALID: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
    const INVALID: Style = AnsiColor::Yellow.on_default().effects(Effects::BOLD);

    pub const STYLES: Styles = {
        Styles::styled()
            .header(HEADER)
            .usage(USAGE)
            .literal(LITERAL)
            .placeholder(PLACEHOLDER)
            .error(ERROR)
            .valid(VALID)
            .invalid(INVALID)
            .error(ERROR)
    };
}

mod heading {
    pub const GIT_PARAMETERS: &str = "Git Parameters";
    pub const TEMPLATE_SELECTION: &str = "Template Selection";
    pub const OUTPUT_PARAMETERS: &str = "Output Parameters";
}

#[derive(Parser)]
#[command(
    name = "cargo stm32bs",
    bin_name = "cargo",
    arg_required_else_help(true),
    version,
    about,
    next_line_help(false),
    styles(style::STYLES)
)]
pub enum Cli {
    #[command(name = "stm32bs", visible_alias = "stbs")]
    ParseArgs(AppArgs),
}

#[derive(Clone, Debug, Args)]
#[command(arg_required_else_help(false), version, about)]
pub struct AppArgs {
    #[command(flatten)]
    pub template_path: TemplatePath,

    /// Directory to create / project name; if the name isn't in kebab-case, it will be converted
    #[arg(long, short, value_parser, help_heading = heading::OUTPUT_PARAMETERS)]
    pub name: Option<String>,

    /// The chip part number to use for the project. This is used to generate the correct
    #[arg(long="chip", short, value_parser, help_heading = heading::OUTPUT_PARAMETERS)]
    pub chip_pn: Option<String>,

    /// Enables more verbose output.
    #[arg(long, short, action)]
    pub verbose: bool,

    /// Pass template values through a file. Values should be in the format `key=value`, one per
    /// line
    #[arg(long="values-file", value_parser, alias="template-values-file", value_name="FILE", help_heading = heading::OUTPUT_PARAMETERS)]
    pub template_values_file: Option<String>,

    /// If silent mode is set all variables will be extracted from the template_values_file. If a
    /// value is missing the project generation will fail
    #[arg(long, short, requires("name"), action)]
    pub silent: bool,

    /// Use a different ssh identity
    #[arg(short = 'i', long = "identity", value_parser, value_name="IDENTITY", help_heading = heading::GIT_PARAMETERS)]
    pub ssh_identity: Option<PathBuf>,

    /// Use a different gitconfig file, if omitted the usual $HOME/.gitconfig will be used
    #[arg(long = "gitconfig", value_parser, value_name="GITCONFIG_FILE", help_heading = heading::GIT_PARAMETERS)]
    pub gitconfig: Option<PathBuf>,

    /// Define a value for use during template expansion. E.g `--define foo=bar`
    #[arg(long, short, number_of_values = 1, value_parser, help_heading = heading::OUTPUT_PARAMETERS)]
    pub define: Vec<String>,

    /// Generate the template directly at the given path.
    #[arg(long, value_parser, value_name="PATH", help_heading = heading::OUTPUT_PARAMETERS)]
    pub destination: Option<PathBuf>,

    /// Allows running system commands without being prompted. Warning: Setting this flag will
    /// enable the template to run arbitrary system commands without user confirmation. Use at your
    /// own risk and be sure to review the template code beforehand.
    #[arg(short, long, action, help_heading = heading::OUTPUT_PARAMETERS)]
    pub allow_commands: bool,

    /// Allow the template to overwrite existing files in the destination.
    #[arg(short, long, action, help_heading = heading::OUTPUT_PARAMETERS)]
    pub overwrite: bool,

    /// Skip downloading git submodules (if there are any)
    #[arg(long, action, help_heading = heading::GIT_PARAMETERS)]
    pub skip_submodules: bool,
}

impl Default for AppArgs {
    fn default() -> Self {
        Self {
            template_path: TemplatePath::default(),
            name: None,
            chip_pn: None,
            verbose: false,
            template_values_file: None,
            silent: false,
            ssh_identity: None,
            gitconfig: None,
            define: Vec::default(),
            destination: None,
            allow_commands: false,
            overwrite: false,
            skip_submodules: false,
        }
    }
}

#[derive(Default, Debug, Clone, Args)]
pub struct TemplatePath {
    /// Auto attempt to use as `--git` or --path. If it is specified explicitly,
    /// use as subfolder.
    #[arg()]
    pub auto_path: Option<String>,

    /// Git repository to clone template from. Can be a URL (like
    /// `https://github.com/rust-cli/cli-template`), a path (relative or absolute), or an
    /// `owner/repo` abbreviated GitHub URL (like `rust-cli/cli-template`).
    ///
    /// Note that cargo generate will first attempt to interpret the `owner/repo` form as a
    /// relative path and only try a GitHub URL if the local path doesn't exist.
    #[arg(short, long, help_heading = heading::TEMPLATE_SELECTION)]
    pub git: Option<String>,

    /// Branch to use when installing from git
    #[arg(short, long, conflicts_with_all = ["revision", "tag"], help_heading = heading::GIT_PARAMETERS)]
    pub branch: Option<String>,

    /// Tag to use when installing from git
    #[arg(short, long, conflicts_with_all = ["revision", "branch"], help_heading = heading::GIT_PARAMETERS)]
    pub tag: Option<String>,

    /// Git revision to use when installing from git (e.g. a commit hash)
    #[arg(short, long, conflicts_with_all = ["tag", "branch"], alias = "rev", help_heading = heading::GIT_PARAMETERS)]
    pub revision: Option<String>,

    /// Local path to copy the template from. Can not be specified together with --git.
    #[arg(short, long, help_heading = heading::TEMPLATE_SELECTION)]
    pub path: Option<String>,
}

impl TemplatePath {
    /// # Panics
    /// Will panic if no path to a template has been set at all,
    /// which is never if Clap is initialized properly.
    pub fn any_path(&self) -> &str {
        self.git
            .as_ref()
            .or(self.path.as_ref())
            .or(self.auto_path.as_ref())
            .unwrap()
    }

    /// Check exist an auto path or git path or local path
    pub fn have_any_path(&self) -> bool {
        if self.auto_path != None {
            return true;
        }
        if self.git != None {
            return true;
        }
        if self.path != None {
            return true;
        }
        false
    }

    pub const fn git(&self) -> Option<&(impl AsRef<str> + '_)> {
        self.git.as_ref()
    }

    pub const fn branch(&self) -> Option<&(impl AsRef<str> + '_)> {
        self.branch.as_ref()
    }

    pub const fn tag(&self) -> Option<&(impl AsRef<str> + '_)> {
        self.tag.as_ref()
    }

    pub const fn revision(&self) -> Option<&(impl AsRef<str> + '_)> {
        self.revision.as_ref()
    }

    pub const fn path(&self) -> Option<&(impl AsRef<str> + '_)> {
        self.path.as_ref()
    }

    pub const fn auto_path(&self) -> Option<&(impl AsRef<str> + '_)> {
        self.auto_path.as_ref()
    }
}

/// To get the arguments list from terminal
/// Return : work arguments
pub fn resolve_args() -> AppArgs {
    let args = env::args();
    let Cli::ParseArgs(args) = Cli::parse_from(args);
    args
}

#[cfg(test)]
mod cli_tests {
    use super::*;

    #[test]
    fn test_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert()
    }
}
