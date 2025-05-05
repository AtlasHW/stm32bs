use crate::helpers::prelude::*;

#[test]
fn it_substitutes_date() {
    let template = tempdir()
        .with_default_manifest()
        .file(
            "Cargo.toml",
            r#"[package]
name = "{{project-name}}"
description = "A wonderful project Copyright {{ "2018-10-04 18:18:45 +0200" | date: "%Y" }}"
version = "0.1.0"
"#,
        )
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
        .read("foobar-project/Cargo.toml")
        .contains("Copyright 2018"));
}

#[test]
fn it_errors_on_invalid_template() {
    let template = tempdir()
        .with_default_manifest()
        .file(
            "Cargo.toml",
            r#"[package]
name = "{{project}<>M>*(&^)-name}}"
description = "A wonderful project Copyright {{ "2018-10-04 18:18:45 +0200" | date: "%Y" }}"
version = "0.1.0"
"#,
        )
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
        .failure()
        .stderr(predicates::str::contains("Error processing file").from_utf8());
}

#[test]
fn it_applies_filters() {
    let template = tempdir()
        .with_default_manifest()
        .file(
            "stm32bs.toml",
            indoc! {r#"
                [template]
                description = "A wonderful project"
                version = ">=0.0.3"
                include = ["filters.txt"]
            "#},
        )
        .file(
            "filters.txt",
            r#"kebab_case = {{"some text" | kebab_case}}
lower_camel_case = {{"some text" | lower_camel_case}}
pascal_case = {{"some text" | pascal_case}}
shouty_kebab_case = {{"some text" | shouty_kebab_case}}
shouty_snake_case = {{"some text" | shouty_snake_case}}
snake_case = {{"some text" | snake_case}}
title_case = {{"some text" | title_case}}
upper_camel_case = {{"some text" | upper_camel_case}}
without_suffix = {{input1 | split: "_" | first}}
"#,
        )
        .init_git()
        .build();
    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_chip("STM32C011D6")
        .arg_type("empty")
        .arg_branch("main")
        .arg("--define")
        .arg("input1=input1_suffix")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    let cargo_toml = dir.read("foobar-project/filters.txt");
    assert!(cargo_toml.contains("kebab_case = some-text"));
    assert!(cargo_toml.contains("lower_camel_case = someText"));
    assert!(cargo_toml.contains("pascal_case = SomeText"));
    assert!(cargo_toml.contains("shouty_kebab_case = SOME-TEXT"));
    assert!(cargo_toml.contains("shouty_snake_case = SOME_TEXT"));
    assert!(cargo_toml.contains("snake_case = some_text"));
    assert!(cargo_toml.contains("title_case = Some Text"));
    assert!(cargo_toml.contains("upper_camel_case = SomeText"));
    assert!(cargo_toml.contains("without_suffix = input1"));
}
