use crate::git::utils::home;
use anyhow::Context;
use anyhow::Result;
use gix_config::{File as GitConfigParser, Source};
use std::path::{Path, PathBuf};

pub fn find_gitconfig() -> Result<Option<PathBuf>> {
    let gitconfig = home().map(|home| home.join(".gitconfig"))?;
    if gitconfig.exists() {
        return Ok(Some(gitconfig));
    }

    Ok(None)
}

/// trades urls, to replace a given repo remote url with the right on based
/// on the `[url]` section in the `~/.gitconfig`
pub fn resolve_instead_url(
    remote: impl AsRef<str>,
    gitconfig: impl AsRef<Path>,
) -> Result<Option<String>> {
    let gitconfig = gitconfig.as_ref().to_path_buf();
    let remote = remote.as_ref().to_string();
    let config = GitConfigParser::from_path_no_includes(gitconfig, Source::User)
        .context("Cannot read or parse .gitconfig")?;
    let x = config.sections_by_name("url").and_then(|iter| {
        iter.map(|section| {
            let head = section.header();
            let body = section.body();
            let url = head.subsection_name();
            let instead_of = body
                .value("insteadOf")
                .map(|x| std::str::from_utf8(&x[..]).unwrap().to_owned());
            (instead_of, url)
        })
        .filter(|(old, new)| new.is_some() && old.is_some())
        .find_map(|(old, new)| {
            let old = old.unwrap();
            let new = new.unwrap().to_string();
            remote
                .starts_with(old.as_str())
                .then(|| remote.replace(old.as_str(), new.as_str()))
        })
    });

    Ok(x)
}
