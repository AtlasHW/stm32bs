use crate::helpers::prelude::*;

#[test]
fn it_can_use_a_plain_folder() {
    let template = tempdir().with_default_manifest().build();

    let dir = tempdir().build();

    binary()
        .arg_name("foobar-project")
        .arg_chip("STM32G071CBT6TR")
        .arg_type("empty")
        .arg(template.path())
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(
            predicates::str::contains("Done!")
                .and(predicates::str::contains(format!(
                    "Auto path `{}` detected, trying it as a local path",
                    template.path().display()
                )))
                .from_utf8(),
        );

    assert_eq!(dir.exists("foobar-project"), true);
    assert_eq!(dir.exists("foobar-project/.git"), false);
    assert_eq!(dir.exists("foobar-project/Cargo.toml"), true);
    assert_eq!(dir.exists("foobar-project/memory.x"), true);
    assert_eq!(dir.exists("foobar-project/build.rs"), true);
    assert_eq!(dir.exists("foobar-project/src/main.rs"), true);
    assert_eq!(dir.exists("foobar-project/build.rs"), true);
    assert_eq!(dir.exists("foobar-project/.cargo/config.toml"), true);
    assert_eq!(dir.exists("foobar-project/stm32bs.toml"), false);
    assert_eq!(dir.exists("foobar-project/.stm32bs.toml"), true);
}

#[test]
fn it_can_use_a_specified_path() {
    let template = tempdir().with_default_manifest().build();

    let dir = tempdir().build();

    binary()
        .arg_name("foobar-project")
        .arg_chip("STM32G071CBT6TR")
        .arg_type("empty")
        .arg_path(template.path())
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert_eq!(dir.exists("foobar-project"), true);
    assert_eq!(dir.exists("foobar-project/.git"), false);
    assert_eq!(dir.exists("foobar-project/Cargo.toml"), true);
    assert_eq!(dir.exists("foobar-project/memory.x"), true);
    assert_eq!(dir.exists("foobar-project/build.rs"), true);
    assert_eq!(dir.exists("foobar-project/src/main.rs"), true);
    assert_eq!(dir.exists("foobar-project/build.rs"), true);
    assert_eq!(dir.exists("foobar-project/.cargo/config.toml"), true);
    assert_eq!(dir.exists("foobar-project/stm32bs.toml"), false);
    assert_eq!(dir.exists("foobar-project/.stm32bs.toml"), true);
}

#[test]
fn it_substitutes_lowcase_chip_pn() {
    let template = tempdir().init_default_template().build();

    let dir = tempdir().build();

    binary()
        .arg_chip("stm32g071cbt6tr")
        .arg_type("empty")
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_branch("main")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("foobar-project/Cargo.toml")
        .contains("foobar-project"));
}

#[test]
fn it_substitutes_authors_and_username() {
    let template = tempdir()
        .with_default_manifest()
        .file(
            "Cargo.toml",
            r#"[package]
name = "{{project-name}}"
authors = "{{authors}}"
description = "A wonderful project by {{username}}"
version = "0.1.0"
"#,
        )
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_chip("STM32G071CBT6TR")
        .arg_type("empty")
        .arg_name("foobar-project")
        .arg_branch("main")
        .current_dir(dir.path())
        .env("CARGO_EMAIL", "Email")
        .env("CARGO_NAME", "Author")
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("foobar-project/Cargo.toml")
        .contains(r#"authors = "Author <Email>""#));
    assert!(dir
        .read("foobar-project/Cargo.toml")
        .contains(r#"description = "A wonderful project by Author""#));
}

#[test]
fn it_substitutes_os_arch() {
    let template = tempdir()
        .with_default_manifest()
        .file(
            "stm32bs.toml",
            r#"
[template]
include = ["some-file"]
description = "A wonderful project"
version = ">=0.0.3"
"#,
        )
        .file("some-file", r#"{{os-arch}}"#)
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_chip("STM32G071CBT6TR")
        .arg_type("empty")
        .arg_name("foobar-project")
        .arg_branch("main")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir.read("foobar-project/some-file").contains(&format!(
        "{}-{}",
        env::consts::OS,
        env::consts::ARCH
    )));
}

#[test]
fn it_can_render_pac_name() {
    let template = tempdir()
        .with_default_manifest()
        .file(
            "src/main.rs",
            r#"
extern crate {{pac_name}};
"#,
        )
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_branch("main")
        .arg_chip("STM32G071CBT6TR")
        .arg_type("empty")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    let file = dir.read("foobar-project/src/main.rs");
    assert!(file.contains("stm32g0"));
}

#[test]
fn short_commands_work() {
    let template = tempdir().init_default_template().build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_chip("STM32G071CBT6TR")
        .arg_type("empty")
        .arg_branch("main")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("foobar-project/Cargo.toml")
        .contains("foobar-project"));
}

#[test]
fn it_can_generate_inside_existing_repository() -> anyhow::Result<()> {
    let template = tempdir().init_default_template().build();
    let dir = tempdir().build();
    binary()
        .arg_git(template.path())
        .arg_name("outer")
        .arg_chip("STM32G071CBT6TR")
        .arg_type("empty")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());
    assert!(dir.read("outer/Cargo.toml").contains("outer"));
    let outer_project_dir = dir.path().join("outer");

    binary()
        .arg_git(template.path())
        .arg_name("inner")
        .arg_chip("STM32G071CBT6TR")
        .arg_type("empty")
        .current_dir(&outer_project_dir)
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());
    assert!(dir.read("outer/inner/Cargo.toml").contains("inner"));
    Ok(())
}

#[test]
fn it_can_generate_into_cwd() -> anyhow::Result<()> {
    let template = tempdir().init_default_template().build();
    let dir = tempdir().build();
    assert!(
        !dir.path().join(".git").exists(),
        "Pre-condition: there should not be a .git dir in CWD"
    );

    binary()
        .arg_git(template.path())
        .arg_name("my-proj")
        .arg_chip("STM32G071CBT6TR")
        .arg_type("empty")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());
    assert!(dir.read("my-proj/Cargo.toml").contains("my-proj"));

    assert!(
        !dir.path().join(".git").exists(),
        "Post-condition: there should not be a .git dir in CWD"
    );
    Ok(())
}

#[test]
fn it_can_generate_into_existing_git_dir() -> anyhow::Result<()> {
    let template = tempdir().init_default_template().build();
    let dir = tempdir().file(".git/config", "foobar").build();
    assert!(
        dir.path().join(".git").exists(),
        "Pre-condition: there is a .git dir in CWD"
    );

    binary()
        .arg_git(template.path())
        .arg_name("my-proj")
        .arg_chip("STM32G071CBT6TR")
        .arg_type("empty")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());
    assert!(dir.read("my-proj/Cargo.toml").contains("my-proj"));
    assert!(
        dir.read(".git/config").contains("foobar"),
        "Post-condition: .git/config is preserved"
    );
    Ok(())
}

#[test]
fn it_can_generate_at_given_path() -> anyhow::Result<()> {
    let template = tempdir().init_default_template().build();
    let dir = tempdir().build();
    let dest = dir.path().join("destination");
    fs::create_dir(&dest).expect("can create directory");
    binary()
        .arg_git(template.path())
        .arg_chip("STM32G071CBT6TR")
        .arg_type("empty")
        .arg_name("my-proj")
        .arg("--destination")
        .arg(&dest)
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());
    assert!(dir
        .read("destination/my-proj/Cargo.toml")
        .contains("my-proj"));
    Ok(())
}

#[test]
fn it_does_not_overwrite_existing_files() -> anyhow::Result<()> {
    let template = tempdir().init_default_template().build();
    let dir = tempdir().build();
    let _ = binary()
        .arg_git(template.path())
        .arg_name("my-proj")
        .arg_chip("STM32G071CBT6TR")
        .arg_type("empty")
        .current_dir(dir.path())
        .status();
    binary()
        .arg_git(template.path())
        .arg_name("overwritten-proj")
        .arg_chip("STM32G071CBT6TR")
        .arg_type("empty")
        .current_dir(dir.path())
        .assert()
        .success();
    assert!(dir.read("my-proj/Cargo.toml").contains("my-proj"));
    Ok(())
}

#[test]
fn it_can_overwrite_files() -> anyhow::Result<()> {
    let template = tempdir().init_default_template().build();
    let dir = tempdir().build();
    let _ = binary()
        .arg_git(template.path())
        .arg_name("my-proj")
        .arg_chip("STM32G071CBT6TR")
        .arg_type("empty")
        .current_dir(dir.path())
        .status();
    binary()
        .arg_git(template.path())
        .arg_name("overwritten-proj")
        .arg_chip("STM32G071CBT6TR")
        .arg_type("empty")
        .arg("--overwrite")
        .current_dir(dir.path())
        .assert()
        .success();
    assert!(dir
        .read("overwritten-proj/Cargo.toml")
        .contains("overwritten-proj"));
    Ok(())
}

#[test]
fn it_always_removes_genignore_file() {
    let template = tempdir()
        .with_default_manifest()
        .file(".genignore", r#"farts"#)
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_chip("STM32G071CBT6TR")
        .arg_type("empty")
        .arg_branch("main")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir.exists("foobar-project/.genignore").not());
}

#[test]
fn it_always_removes_cargo_ok_file() {
    let template = tempdir()
        .with_default_manifest()
        .file(
            "Cargo.toml",
            indoc! {r#"
                [package]
                name = "{{project-name}}"
                description = "A wonderful project"
                version = "0.1.0"
            "#},
        )
        .file(".genignore", r#"farts"#)
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_branch("main")
        .arg_chip("STM32G071CBT6TR")
        .arg_type("empty")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir.exists("foobar-project/.cargo-ok").not());
}

#[test]
fn it_removes_genignore_files_before_substitution() {
    let template = tempdir()
        .with_default_manifest()
        .file(
            "Cargo.toml",
            indoc! {r#"
                [package]
                name = "{{project-name}}"
                description = "A wonderful project"
                version = "0.1.0"
            "#},
        )
        .file(".cicd_workflow", "i contain a ${{ github }} var")
        .file(".genignore", r#".cicd_workflow"#)
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_branch("main")
        .arg_chip("STM32G071CBT6TR")
        .arg_type("empty")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir.exists("foobar-project/.cicd_workflow").not());
}

#[test]
fn it_does_not_remove_files_from_outside_project_dir() {
    let template = tempdir()
        .with_default_manifest()
        .file(
            "Cargo.toml",
            indoc! {r#"
                [package]
                name = "{{project-name}}"
                description = "A wonderful project"
                version = "0.1.0"
            "#},
        )
        .file(
            ".genignore",
            r#"../dangerous.todelete.cargogeneratetests
"#,
        )
        .init_git()
        .build();

    let dir = tempdir().build();

    let dangerous_file = template
        .path()
        .join("..")
        .join("dangerous.todelete.cargogeneratetests");

    fs::write(&dangerous_file, "YOU BETTER NOT").unwrap_or_else(|_| {
        panic!(
            "Could not write {}",
            dangerous_file.to_str().expect("Could not read path.")
        )
    });

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_branch("main")
        .arg_chip("STM32G071CBT6TR")
        .arg_type("empty")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(fs::metadata(&dangerous_file)
        .expect("should exist")
        .is_file());
    fs::remove_file(&dangerous_file).expect("failed to clean up test file");
}

#[test]
fn errant_ignore_entry_doesnt_affect_template_files() {
    let template = tempdir()
        .with_default_manifest()
        .file(
            "Cargo.toml",
            indoc! {r#"
                [package]
                name = "{{project-name}}"
                description = "A wonderful project"
                version = "0.1.0"
            "#},
        )
        .file(
            ".genignore",
            r#"../dangerous.todelete.cargogeneratetests
"#,
        )
        .file("./dangerous.todelete.cargogeneratetests", "IM FINE OK")
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_branch("main")
        .arg_chip("STM32G071CBT6TR")
        .arg_type("empty")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(fs::metadata(
        template
            .path()
            .join("dangerous.todelete.cargogeneratetests")
    )
    .unwrap()
    .is_file());
}

#[test]
fn it_loads_a_submodule() {
    let submodule = tempdir()
        .file("tREADME.rs", "*JUST A SUBMODULE*")
        .init_git()
        .build();

    let submodule_url = url::Url::from_file_path(submodule.path()).unwrap();
    let template = tempdir()
        .with_default_manifest()
        .file(
            "stm32bs.toml",
            indoc! {r#"
                [template]
                description = "A wonderful project"
                version = ">=0.0.3"
                include = ["submodule/*"]
            "#},
        )
        .file(
            "Cargo.toml",
            indoc! { r#"
                [package]
                name = "{{project-name}}"
                description = "A wonderful project"
                version = "0.1.0"
            "#},
        )
        .init_git()
        .add_submodule("./submodule/", submodule_url.as_str())
        .build();

    let dir = tempdir().build();
    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_chip("STM32G071CBT6TR")
        .arg_type("empty")
        .arg_branch("main")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("foobar-project/Cargo.toml")
        .contains("foobar-project"));
    assert!(dir
        .read("foobar-project/submodule/tREADME.rs")
        .contains("*JUST A SUBMODULE*"));
}

#[test]
fn it_allows_relative_paths() {
    let template = tempdir()
        .with_default_manifest()
        .file(
            "Cargo.toml",
            indoc! { r#"
                [package]
                name = "{{project-name}}"
                description = "A wonderful project"
                version = "0.1.0"
            "#},
        )
        .init_git()
        .build();

    let relative_path = {
        let mut relative_path = std::path::PathBuf::new();
        relative_path.push("../");
        relative_path.push(template.path().file_name().unwrap().to_str().unwrap());
        relative_path
    };

    let dir = tempdir().build();
    binary()
        .arg_git(relative_path)
        .arg_name("foobar-project")
        .arg_branch("main")
        .arg_chip("STM32G071CBT6TR")
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
fn it_doesnt_warn_with_neither_config_nor_ignore() {
    let template = tempdir().init_default_template().build();
    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_chip("STM32G071CBT6TR")
        .arg_type("empty")
        .arg_branch("main")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Removed:").count(0).from_utf8())
        .stdout(predicates::str::contains("neither").count(0).from_utf8())
        .stdout(predicates::str::contains("Done!").from_utf8());
}

#[test]
fn it_processes_dot_github_directory_files() {
    let template = tempdir()
        .with_default_manifest()
        .file(
            "stm32bs.toml",
            indoc! {r#"
                [template]
                description = "A wonderful project"
                version = ">=0.0.3"
                include = [".github/*"]
            "#},
        )
        .file(".github/foo.txt", "{{project-name}}")
        .init_git()
        .build();
    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_chip("STM32G071CBT6TR")
        .arg_type("empty")
        .arg_branch("main")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert_eq!(dir.read("foobar-project/.github/foo.txt"), "foobar-project");
}

#[test]
fn it_ignore_tags_inside_raw_block() {
    let raw_body = r#"{{badges}}
# {{crate}} {{project-name}}
{{readme}}
{{license}}
## Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
This project try follow rules:
* [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
* [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
_This README was generated with [cargo-readme](https://github.com/livioribeiro/cargo-readme) from [template](https://github.com/xoac/crates-io-lib-template)
"#;
    let raw_template = format!("{{% raw %}}{raw_body}{{% endraw %}}");
    let template = tempdir()
        .with_default_manifest()
        .file(
            "stm32bs.toml",
            indoc! {r#"
                [template]
                description = "A wonderful project"
                version = ">=0.0.3"
                include = ["README.tpl"]
            "#},
        )
        .file("README.tpl", raw_template)
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_branch("main")
        .arg_chip("STM32G071CBT6TR")
        .arg_type("empty")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    let template = dir.read("foobar-project/README.tpl");
    assert!(template.contains("{{badges}}"));
    assert!(template.contains("{{crate}}"));
    assert!(template.contains("{{project-name}}"));
    assert!(template.contains("{{readme}}"));
    assert!(template.contains("{{license}}"));
}

#[test]
fn it_dont_initializing_repository() {
    // Build and commit on branch named 'main'
    let template = tempdir().init_default_template().build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_chip("STM32G071CBT6TR")
        .arg_type("empty")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("foobar-project/Cargo.toml")
        .contains("foobar-project"));
    assert!(Repository::open(dir.path().join("foobar-project")).is_err());
}

#[test]
fn it_provides_crate_type_bin() {
    // Build and commit on branch named 'main'
    let template = tempdir()
        .with_default_manifest()
        .file(
            "Cargo.toml",
            r#"[package]
name = "{{project-name}}"
description = "this is a {{crate_type}}"
version = "0.1.0"
"#,
        )
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_chip("STM32G071CBT6TR")
        .arg_type("empty")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    let cargo_toml = dir.read("foobar-project/Cargo.toml");
    assert!(cargo_toml.contains("this is a bin"));
}

#[test]
fn it_skips_substitution_for_unknown_variables_in_cargo_toml() {
    let template = tempdir()
        .with_default_manifest()
        .file(
            "Cargo.toml",
            r#"[package]
name = "{{ project-name }}"
description = "{{ project-description }}"
description2 = "{{ project-some-other-thing }}"
version = "0.1.0"
"#,
        )
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_chip("STM32G071CBT6TR")
        .arg_type("empty")
        .arg_branch("main")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(
        dir.read("foobar-project/Cargo.toml")
            .contains("foobar-project"),
        "project-name was not substituted"
    );
    assert!(!dir
        .read("foobar-project/Cargo.toml")
        .contains("{{ project-description }}"));
    assert!(!dir
        .read("foobar-project/Cargo.toml")
        .contains("{{ project-some-other-thing }}"));
}

#[test]
fn error_message_for_invalid_repo_or_user() {
    let dir = tempdir().build();

    binary()
        .arg_git("sassman/cli-template-rs-xx")
        .arg_name("favorite-project")
        .arg_chip("STM32G071CBT6TR")
        .arg_type("empty")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(
            predicates::str::contains(r#"Error: Please check if the Git user / repository exists"#)
                .from_utf8(),
        );
}
