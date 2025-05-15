use crate::helpers::prelude::*;

#[test]
fn it_need_input_projectname() {
    let template = tempdir().with_default_manifest().init_git().build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        //.arg_name("foobar-project")
        .arg_chip("STM32G071CBT6TR")
        .arg_type("empty")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("not a terminal").from_utf8());
}

#[test]
fn it_can_fill_projectname() {
    let template = tempdir().with_default_manifest().init_git().build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar-project")
        .arg_chip("STM32G071CBT6TR")
        .arg_type("empty")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done").from_utf8());

    assert!(dir
        .read("foobar-project/Cargo.toml")
        .contains("foobar-project"));
}

#[test]
fn it_can_fill_projectname_with_illegal_char() {
    let template = tempdir().with_default_manifest().init_git().build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar&project")
        .arg_chip("STM32G071CBT6TR")
        .arg_type("empty")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done").from_utf8());

    println!("{}", dir.read("foobar-project/Cargo.toml"));
    assert!(dir
        .read("foobar-project/Cargo.toml")
        .contains("foobar-project"));
}

#[test]
fn it_can_fill_projectname_with_underline() {
    let template = tempdir().with_default_manifest().init_git().build();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("foobar_project")
        .arg_chip("STM32G071CBT6TR")
        .arg_type("empty")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done").from_utf8());

    println!("{}", dir.read("foobar-project/Cargo.toml"));
    assert!(dir
        .read("foobar-project/Cargo.toml")
        .contains("foobar-project"));
}

#[test]
fn it_can_fill_projectname_with_uppercase() {
    let template = create_template();

    let dir = tempdir().build();

    binary()
        .arg_git(template.path())
        .arg_name("FoobarProject")
        .arg_chip("STM32G071CBT6TR")
        .arg_type("empty")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(
            predicates::str::contains("Done")
                .from_utf8()
                .and(predicates::str::contains("Renaming project").from_utf8()),
        );

    println!("{}", dir.read("foobar-project/Cargo.toml"));
    assert!(dir
        .read("foobar-project/Cargo.toml")
        .contains("foobar-project"));
}

#[test]
// TODO: this test fails on linux, for mysterious reasons
#[cfg(not(target_os = "linux"))]
fn it_preserves_liquid_files_with_git() {
    assert_liquid_paths(Location::Git)
}

#[test]
// TODO: this test fails on linux, for mysterious reasons
#[cfg(not(target_os = "linux"))]
fn it_preserves_liquid_files_with_path() {
    assert_liquid_paths(Location::Path)
}

#[allow(dead_code)]
#[derive(PartialEq)]
enum Location {
    Git,
    Path,
}

#[allow(dead_code)]
fn assert_liquid_paths(location: Location) {
    let mut project_builder = tempdir()
        .file("README.md", "This file contents will be overwritten")
        .file("README.md.liquid", "This file contents will be preserved");

    if location == Location::Git {
        project_builder = project_builder.init_git();
    }

    let template = project_builder.build();

    let mut binary_command = binary();
    match location {
        Location::Git => {
            binary_command.arg_git(template.path());
        }
        Location::Path => {
            binary_command.arg_path(template.path());
        }
    }

    let target = tempdir().build();
    binary_command
        .arg_name("foobar-project")
        .current_dir(target.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Done!").from_utf8());

    assert!(
        target.exists("foobar-project/README.md"),
        "project should contain foobar-project/README.md"
    );
    assert_eq!(
        target.read("foobar-project/README.md"),
        "This file contents will be preserved",
        "project should keep .liquid file contents"
    );

    assert!(
        !target.exists("foobar-project/README.md.liquid"),
        "project should not contain foobar-project/README.md.liquid"
    );
}
