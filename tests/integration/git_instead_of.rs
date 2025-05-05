use crate::helpers::prelude::*;

#[test]
fn should_read_the_instead_of_config_and_rewrite_an_git_at_url_to_https() {
    let gitconfig_dir = tempdir()
        .file(
            ".gitconfig",
            indoc! { r#"
                [url "https://github.com/"]
                insteadOf = "git@github.com:"
            "# },
        )
        .build();

    let target = tempdir().build();

    binary()
        .arg_gitconfig(gitconfig_dir.path().join(".gitconfig"))
        .arg_git("git@github.com:AtlasHW/stm32bs-template-default")
        .arg_name("foobar-project")
        .arg_chip("STM32F103C8")
        .arg_type("empty")
        .current_dir(target.path())
        .env("RUST_LOG", "debug")
        .assert()
        .success()
        .stdout(
            predicates::str::contains("Done!").from_utf8().and(
                predicates::str::contains(
                    "gitconfig 'insteadOf' lead to this url: https://github.com/AtlasHW/stm32bs-template-default",
                )
                .from_utf8(),
            ),
        );

    let cargo_toml = target.read("foobar-project/Cargo.toml");
    assert!(cargo_toml.contains("foobar-project"));
}

#[test]
fn should_read_the_instead_of_config_and_rewrite_an_ssh_url_to_https() {
    let gitconfig_dir = tempdir()
        .file(
            ".gitconfig",
            indoc! { r#"
                [url "https://github.com/"]
                insteadOf = "ssh://git@github.com/"
            "# },
        )
        .build();

    let target = tempdir().build();

    binary()
        .arg_gitconfig(gitconfig_dir.path().join(".gitconfig"))
        .arg_git("ssh://git@github.com/AtlasHW/stm32bs-template-default")
        .arg_name("foobar-project")
        .arg_chip("STM32G071CB")
        .arg_type("empty")
        .current_dir(target.path())
        .env("RUST_LOG", "debug")
        .assert()
        .success()
        .stdout(
            predicates::str::contains("Done!").from_utf8().and(
                predicates::str::contains(
                    "gitconfig 'insteadOf' lead to this url: https://github.com/AtlasHW/stm32bs-template-default",
                )
                .from_utf8(),
            ),
        );

    let cargo_toml = target.read("foobar-project/Cargo.toml");
    assert!(cargo_toml.contains("foobar-project"));
}
