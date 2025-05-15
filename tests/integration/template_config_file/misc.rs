use crate::helpers::prelude::*;

#[test]
fn it_always_removes_config_file() {
    let template = tempdir().with_default_manifest().init_git().build();

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

    assert!(!dir.exists("foobar-project/stm32bs.toml"));
}

//https://github.com/ashleygwilliams/cargo-generate/issues/181
#[test]
fn it_doesnt_warn_on_config_with_no_ignore() {
    let template = tempdir()
        .with_default_manifest()
        .file(
            "cargo-generate.toml",
            r#"[template]
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
        .stdout(predicates::str::contains("neither").count(0).from_utf8())
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(!dir.exists("foobar-project/cargo-generate.toml"));
}

#[test]
fn a_template_can_specify_to_be_generated_into_cwd() -> anyhow::Result<()> {
    let template = tempdir()
        .with_default_manifest()
        .file(
            "cargo-generate.toml",
            indoc! {r#"
                [template]
                init = true
                "#},
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

    assert!(dir.exists("foobar-project/Cargo.toml"));
    assert!(!dir.path().join("foobar-project/.git").exists());
    Ok(())
}
