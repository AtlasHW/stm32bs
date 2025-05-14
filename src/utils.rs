use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use auth_git2::GitAuthenticator;
use console::style;
use git2::Config;
use git2::FetchOptions;
use git2::ProxyOptions;
use log::info;
use log::warn;
use regex::Regex;
use remove_dir_all::remove_dir_all;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;

use tempfile::TempDir;

/// Make a temperary
pub fn tmp_dir() -> std::io::Result<tempfile::TempDir> {
    tempfile::Builder::new().prefix("stm32bs").tempdir()
}

/// deals with `~/` and `$HOME/` prefixes
pub fn canonicalize_path(p: impl AsRef<Path>) -> Result<PathBuf> {
    let p = p.as_ref();
    let p = if p.starts_with("~/") {
        home()?.join(p.strip_prefix("~/")?)
    } else if p.starts_with("$HOME/") {
        home()?.join(p.strip_prefix("$HOME/")?)
    } else {
        p.to_path_buf()
    };

    p.canonicalize()
        .with_context(|| format!("path does not exist: {}", p.display()))
}

/// home path wrapper
pub fn home() -> Result<PathBuf> {
    home::home_dir().context("$HOME was not set")
}

/// clone git repository into temp using libgit2
pub fn clone_git_template_into_temp(
    git_url: &str,
    branch: Option<&str>,
    tag: Option<&str>,
    revision: Option<&str>,
    identity: Option<&Path>,
    gitconfig: Option<&Path>,
    skip_submodules: bool,
) -> anyhow::Result<TempDir> {
    let git_clone_dir = tmp_dir()?;
    let mut url = git_url.to_string();

    #[cfg(windows)]
    let authenticator = GitAuthenticator::default().try_ssh_agent(true);
    #[cfg(not(windows))]
    let mut authenticator = GitAuthenticator::default()
        .try_ssh_agent(true)
        .add_default_ssh_keys()
        .prompt_ssh_key_password(true)
        .try_password_prompt(3);

    if let Some(identity_path) = identity {
        let identity_path = canonicalize_path(identity_path)?;
        log::info!(
            "{} `{}` {}",
            style("Using private key:").bold(),
            style(format_args!("{}", identity_path.display()))
                .bold()
                .yellow(),
            style("for git-ssh checkout").bold()
        );
        authenticator = authenticator
            .add_ssh_key_from_file(identity_path, None)
            .try_password_prompt(3)
            .prompt_ssh_key_password(true);
    }
    if let Some(gitconfig_path) = gitconfig {
        if !gitconfig_path.exists() {
            bail!(
                "Cannot find the git config file {}",
                gitconfig_path.to_str().unwrap()
            );
        }
    }
    let gitconfig = gitconfig
        .map(|p| p.to_owned())
        .unwrap_or_else(|| home().unwrap().join(".gitconfig"));
    let gitconfig = if gitconfig.exists() {
        let gitconfig_content = Config::open(gitconfig.as_path())?;
        let gitconfig_content2 = Config::open(gitconfig.as_path())?;
        let mut entries = gitconfig_content2.entries(None).unwrap();
        // Match git config file item
        // [url "https://github.com/"]
        // insteadOf = gh:
        while let Some(entry) = entries.next() {
            let entry = entry.unwrap();
            let re = Regex::new("url.(.+).insteadof").unwrap();
            let cap = re.captures(entry.name().unwrap());
            if let Some(item) = cap {
                let insteadof_value = entry.value().unwrap();
                let insteadof_url = item.get(1).unwrap().as_str();
                if url.starts_with(&insteadof_value) {
                    url = insteadof_url.to_owned() + url.strip_prefix(&insteadof_value).unwrap();
                    info!("🔧 gitconfig 'insteadOf' lead to this url: {}", url);
                }
            }
        }
        gitconfig_content
    } else {
        git2::Config::open_default().unwrap()
    };

    let mut tag_or_revision = None;
    if let Some(tag) = tag {
        tag_or_revision = Some(tag.to_owned());
    }
    if let Some(revision) = revision {
        tag_or_revision = Some(revision.to_owned());
    }

    let mut fetch_options = FetchOptions::new();
    let mut callbacks = git2::RemoteCallbacks::new();

    callbacks.credentials(authenticator.credentials(&gitconfig));
    fetch_options.remote_callbacks(callbacks);

    let url = url.clone();

    let is_ssh_repo = url.starts_with("ssh}://") || url.starts_with("git@");
    let is_http_repo = url.starts_with("http://") || url.starts_with("https://");

    if is_http_repo {
        let mut proxy_options = ProxyOptions::new();
        proxy_options.auto();

        fetch_options.proxy_options(proxy_options);
        fetch_options.depth(1);
    }

    if is_ssh_repo || is_http_repo {
        fetch_options.download_tags(git2::AutotagOption::All);
    }

    let mut builder = git2::build::RepoBuilder::new();
    if let Some(branch) = branch {
        builder.branch(branch);
    }
    builder.fetch_options(fetch_options);

    let repository = builder
        .clone(&url, &git_clone_dir.path())
        .context("Please check if the Git user / repository exists.")?;

    if let Some(tag_or_revision) = tag_or_revision {
        let (object, reference) = repository.revparse_ext(tag_or_revision.as_str())?;
        repository.checkout_tree(&object, None)?;
        reference.map_or_else(
            || repository.set_head_detached(object.id()),
            |gref| repository.set_head(gref.name().unwrap()),
        )?
    }

    if skip_submodules {
        return Ok(git_clone_dir);
    }

    let config = repository.config()?;

    for mut sub in repository.submodules()? {
        let mut proxy_options = ProxyOptions::new();
        proxy_options.auto();

        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(authenticator.credentials(&config));

        let mut fetch_options = FetchOptions::new();
        fetch_options.proxy_options(proxy_options);
        fetch_options.remote_callbacks(callbacks);

        let mut update_options = git2::SubmoduleUpdateOptions::new();
        update_options.fetch(fetch_options);
        sub.update(true, Some(&mut update_options))?;
    }

    Ok(git_clone_dir)
}

/// remove context of repository by removing `.git` from filesystem
pub fn remove_history(project_dir: &Path) -> Result<()> {
    let git_dir = project_dir.join(".git");
    if git_dir.exists() && git_dir.is_dir() {
        let mut attempt = 0_u8;

        loop {
            attempt += 1;
            if let Err(e) = remove_dir_all(&git_dir) {
                if attempt == 5 {
                    bail!("{}", e.to_string());
                }

                if e.to_string().contains("The process cannot access the file because it is being used by another process.") {
                    let wait_for = Duration::from_secs(5);
                    warn!("Git history cleanup failed with a windows process blocking error. [Retry in {:?}]", wait_for);
                    sleep(wait_for);
                } else {
                    bail!("{}", e.to_string());
                }
            } else {
                return Ok(());
            }
        }
    } else {
        //FIXME should we assume this is expected by caller?
        // panic!("tmp panic");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::canonicalize_path;
    use std::path::PathBuf;

    #[test]
    fn should_canonicalize() {
        #[cfg(target_os = "macos")]
        {
            assert!(canonicalize_path(PathBuf::from("../"))
                .unwrap()
                .starts_with("/Users/"));

            assert!(canonicalize_path(PathBuf::from("$HOME/"))
                .unwrap()
                .starts_with("/Users/"));
        }
        #[cfg(target_os = "linux")]
        assert_eq!(
            canonicalize_path(PathBuf::from("../")).ok(),
            std::env::current_dir()
                .unwrap()
                .parent()
                .map(|p| p.to_path_buf())
        );
        assert!(canonicalize_path(PathBuf::from("$HOME/"))
            .unwrap()
            .starts_with("/home/"));
        assert_eq!(
            PathBuf::from("../").canonicalize().unwrap(),
            std::env::current_dir()
                .unwrap()
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap()
        );
        #[cfg(windows)]
        assert!(canonicalize_path(PathBuf::from("../"))
            .unwrap()
            // not a bug, a feature:
            // https://stackoverflow.com/questions/41233684/why-does-my-canonicalized-path-get-prefixed-with
            .to_str()
            .unwrap()
            .starts_with("\\\\?\\"));
    }
}
