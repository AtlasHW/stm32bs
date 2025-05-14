use crate::helpers::prelude::*;

#[test]
fn it_allows_a_git_branch_to_be_specified() {
    let template = tempdir().init_default_template().branch("bak").build();
    let dir = tempdir().build();

    binary()
        .arg_branch("bak")
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_chip("STM32C011D6")
        .arg_type("empty")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("foobar-project/Cargo.toml")
        .contains("foobar-project"));
}

#[test]
fn it_allows_a_git_tag_to_be_specified() {
    let template = tempdir().init_default_template().tag("v1.0").build();
    let dir = tempdir().build();

    binary()
        .arg("--tag")
        .arg("v1.0")
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_chip("STM32C011D6")
        .arg_type("empty")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("foobar-project/Cargo.toml")
        .contains("foobar-project"));
}

#[test]
fn it_allows_a_git_revision_to_be_specified() {
    let template = tempdir().init_default_template().build();
    let commit_sha = template.commit_shas().first().unwrap().to_string();
    let dir = tempdir().build();

    binary()
        .arg_revision(commit_sha)
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_chip("STM32C011D6")
        .arg_type("empty")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("foobar-project/Cargo.toml")
        .contains("foobar-project"));
}

#[test]
fn it_removes_git_history_also_on_local_templates() {
    let template = tempdir().init_default_template().build();
    let dir = tempdir().build();

    binary()
        .arg_path(template.path())
        .arg_name("xyz")
        .arg_chip("STM32C011D6")
        .arg_type("empty")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    let target_path = dir.target_path("xyz");
    assert!(!target_path.join(".git").exists());
}
