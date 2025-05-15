use crate::helpers::prelude::*;

#[test]
fn it_can_build_a_demo_project() {
    let template = tempdir()
        .with_default_manifest()
        .file(
            "stm32bs.toml",
            indoc! {r#"
                [template]
                description = "A wonderful project"
                version = ">=0.0.3"
                 [demo.'hello']
            "#},
        )
        .file("demo/hello.rs", "A test demo file: hello")
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_chip("STM32G071CBT6TR")
        .arg_demo("hello")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert_eq!(
        dir.read("foobar-project/src/main.rs").as_str(),
        "A test demo file: hello"
    );
}

#[test]
fn demo_project_can_include_placeholder() {
    let template = tempdir()
        .with_default_manifest()
        .file(
            "stm32bs.toml",
            indoc! {r#"
                [template]
                description = "A wonderful project"
                version = ">=0.0.3"
                 [demo.'hello']
                 port = { type="string", prompt="Port of GPIO is used to LED, eg. B", regex = "^[a-fA-F]$"}
            "#},
        )
        .file("demo/hello.rs", "A test demo file: hello")
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_chip("STM32G071CBT6TR")
        .arg_demo("hello")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("not a terminal").from_utf8()); //need input port
}

#[test]
fn demo_project_can_fill_placeholder() {
    let template = tempdir()
        .with_default_manifest()
        .file(
            "stm32bs.toml",
            indoc! {r#"
                [template]
                description = "A wonderful project"
                version = ">=0.0.3"
                 [demo.'hello']
                 port = { type="string", prompt="Port of GPIO is used to LED, eg. B", regex = "^[a-fA-F]$"}
            "#},
        )
        .file("demo/hello.rs", "hello demo include var port = {{port}}")
        .init_git()
        .build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_chip("STM32G071CBT6TR")
        .arg_demo("hello")
        .arg_define("port=B")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done").from_utf8()); //need input port

    assert_eq!(
        dir.read("foobar-project/src/main.rs").as_str(),
        "hello demo include var port = B"
    );
}
