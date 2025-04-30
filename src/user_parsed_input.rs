//! Input from user but after parse

use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
};

use crate::absolute_path::AbsolutePathExt;
use console::style;

use crate::AppArgs;
use log::warn;

// Contains parsed information from user.
#[derive(Debug)]
pub struct UserParsedInput {
    name: Option<String>,
    chip_pn: Option<String>,

    // from where clone or copy template?
    template_location: TemplateLocation,

    destination: PathBuf,

    // all values that user defined through:
    // 1. environment variables
    // 2. configuration file
    // 3. cli arguments --define
    template_values: HashMap<String, toml::Value>,

    overwrite: bool,
    verbose: bool,
    //TODO:
    // 1. This structure should be used instead of args
    // 2. This struct can contains internally args and app_config to not confuse
    //    other developer with parsing configuration and args by themself
}

impl UserParsedInput {
    /// Try create `UserParsedInput` reading in order \[`AppConfig`\] and \[`Args`\]
    ///
    /// # Panics
    /// This function assume that Args and AppConfig are verified earlier and are logically correct
    /// For example if both `--git` and `--path` are set this function will panic
    pub fn try_from_args(args: &AppArgs) -> Self {
        let destination = args
            .destination
            .as_ref()
            .map(|p| {
                p.as_absolute()
                    .expect("cannot get the absolute path of the destination folder")
                    .to_path_buf()
            })
            .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| ".".into()));

        let mut default_values = HashMap::new();
        for item in args.define.iter() {
            if let Some((k, v)) = item.split_once('=') {
                default_values.insert(k.to_string(), toml::Value::String(v.to_string()));
            }
        }

        let ssh_identity = None;

        // --git
        if let Some(git_url) = args.template_path.git() {
            let git_user_in = GitUserInput::new(
                git_url,
                args.template_path.branch(),
                args.template_path.tag(),
                args.template_path.revision(),
                ssh_identity,
                args.gitconfig.clone(),
                args.skip_submodules,
            );
            return Self {
                name: args.name.clone(),
                chip_pn: args.chip_pn.clone(),
                template_location: git_user_in.into(),
                template_values: default_values,
                overwrite: args.overwrite,
                verbose: args.verbose,
                destination,
            };
        }

        // --path
        if let Some(path) = args.template_path.path() {
            return Self {
                name: args.name.clone(),
                chip_pn: args.chip_pn.clone(),
                template_location: path.as_ref().into(),
                template_values: default_values,
                overwrite: args.overwrite,
                verbose: args.verbose,
                destination,
            };
        }

        // If auto path is inputed, to check git short, local path and git full path
        let any_path = args.template_path.any_path();

        // there is no specified favorite in configuration
        // this part try to guess what user wanted in order:
        // 1. look for abbreviations like gh:, gl: etc.
        let temp_location = abbreviated_git_url_to_full_remote(any_path).map(|git_url| {
            let git_user_in = GitUserInput::with_git_url_and_args(&git_url, args);
            TemplateLocation::from(git_user_in)
        });

        // 2. check if template directory exist
        let temp_location =
            temp_location.or_else(|| local_path(any_path).map(TemplateLocation::from));

        // 3. assume user wanted use --git
        let temp_location = temp_location.unwrap_or_else(|| {
            let git_user_in = GitUserInput::new(
                &any_path,
                args.template_path.branch(),
                args.template_path.tag(),
                args.template_path.revision(),
                ssh_identity,
                args.gitconfig.clone(),
                args.skip_submodules,
            );
            TemplateLocation::from(git_user_in)
        });

        // Print information what happened to user
        let location_msg = match &temp_location {
            TemplateLocation::Git(git_user_input) => {
                format!("git repository: {}", style(git_user_input.url()).bold())
            }
            TemplateLocation::Path(path) => {
                format!("local path: {}", style(path.display()).bold())
            }
        };
        warn!(
            "Auto path `{}` detected, trying it as a {}",
            style(&any_path).bold(),
            location_msg
        );

        Self {
            name: args.name.clone(),
            chip_pn: args.chip_pn.clone(),
            template_location: temp_location,
            template_values: default_values,
            overwrite: args.overwrite,
            verbose: args.verbose,
            destination,
        }
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn chip_pn(&self) -> Option<&str> {
        self.chip_pn.as_deref()
    }

    pub const fn location(&self) -> &TemplateLocation {
        &self.template_location
    }

    pub const fn template_values(&self) -> &HashMap<String, toml::Value> {
        &self.template_values
    }

    pub const fn overwrite(&self) -> bool {
        self.overwrite
    }

    pub const fn is_verbose(&self) -> bool {
        self.verbose
    }

    pub fn destination(&self) -> &Path {
        self.destination.as_path()
    }
}

/// favorite can be in form with abbreviation what means that input is git repository
/// if so, the 3rd character would be a semicolon
pub fn abbreviated_git_url_to_full_remote(git: impl AsRef<str>) -> Option<String> {
    let git = git.as_ref();
    if git.len() >= 3 {
        match &git[..3] {
            "gl:" => Some(format!("https://gitlab.com/{}.git", &git[3..])),
            "bb:" => Some(format!("https://bitbucket.org/{}.git", &git[3..])),
            "gh:" => Some(format!("https://github.com/{}.git", &git[3..])),
            "sr:" => Some(format!("https://git.sr.ht/~{}", &git[3..])),
            _ => None,
        }
    } else {
        None
    }
}

pub fn local_path(fav: &str) -> Option<PathBuf> {
    let path = PathBuf::from(fav);
    (path.exists() && path.is_dir()).then_some(path)
}

// Template should be cloned with git
#[derive(Debug)]
pub struct GitUserInput {
    url: String,
    branch: Option<String>,
    tag: Option<String>,
    revision: Option<String>,
    identity: Option<PathBuf>,
    gitconfig: Option<PathBuf>,
    pub skip_submodules: bool,
}

impl GitUserInput {
    #[allow(clippy::too_many_arguments)]
    fn new(
        url: &impl AsRef<str>,
        branch: Option<&impl AsRef<str>>,
        tag: Option<&impl AsRef<str>>,
        revision: Option<&impl AsRef<str>>,
        identity: Option<PathBuf>,
        gitconfig: Option<PathBuf>,
        skip_submodules: bool,
    ) -> Self {
        Self {
            url: url.as_ref().to_owned(),
            branch: branch.map(|s| s.as_ref().to_owned()),
            tag: tag.map(|s| s.as_ref().to_owned()),
            revision: revision.map(|s| s.as_ref().to_owned()),
            identity,
            gitconfig,
            skip_submodules,
        }
    }

    // when git was used as abbreviation but other flags still could be passed
    fn with_git_url_and_args(url: &impl AsRef<str>, args: &AppArgs) -> Self {
        Self::new(
            url,
            args.template_path.branch(),
            args.template_path.tag(),
            args.template_path.revision(),
            args.ssh_identity.clone(),
            args.gitconfig.clone(),
            args.skip_submodules,
        )
    }

    pub fn url(&self) -> &str {
        self.url.as_ref()
    }

    pub fn branch(&self) -> Option<&str> {
        self.branch.as_deref()
    }

    pub fn tag(&self) -> Option<&str> {
        self.tag.as_deref()
    }

    pub fn revision(&self) -> Option<&str> {
        self.revision.as_deref()
    }

    pub fn identity(&self) -> Option<&Path> {
        self.identity.as_deref()
    }

    pub fn gitconfig(&self) -> Option<&Path> {
        self.gitconfig.as_deref()
    }
}

// Distinguish between plain copy and clone
#[derive(Debug)]
pub enum TemplateLocation {
    Git(GitUserInput),
    Path(PathBuf),
}

impl From<GitUserInput> for TemplateLocation {
    fn from(source: GitUserInput) -> Self {
        Self::Git(source)
    }
}

impl<T> From<T> for TemplateLocation
where
    T: AsRef<Path>,
{
    fn from(source: T) -> Self {
        Self::Path(PathBuf::from(source.as_ref()))
    }
}
