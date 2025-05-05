use crate::helpers::prelude::*;

#[test]
fn it_only_processes_include_files_in_config() {
    let template = tempdir()
        .with_default_manifest()
        .file(
            "stm32bs.toml",
            indoc! {r#"
                [template]
                include = ["included"]
                description = "A wonderful project"
                version = ">=0.0.3"
            "#},
        )
        .file("included", "{{project-name}}")
        .file("excluded1", "{{should-not-process}}")
        .file("excluded2", "{{should-not-process}}")
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_branch("main")
        .arg_chip("STM32C011D6")
        .arg_type("empty")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(dir
        .read("foobar-project/included")
        .contains("foobar-project"));
    assert!(!dir.exists("foobar-project/excluded1"));
    assert!(!dir.exists("foobar-project/excluded2"));
}
