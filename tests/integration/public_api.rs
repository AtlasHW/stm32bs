use crate::helpers::prelude::*;

use crate::{generate, GenerateArgs, TemplatePath};

#[test]
fn it_allows_generate_call_with_public_args_and_returns_the_generated_path() {
    let cwd_before = std::env::current_dir().unwrap();

    let template = tempdir().init_default_template().init_git().build();

    let dir = tempdir().build().root.into_path();

    let args_exposed: GenerateArgs = GenerateArgs {
        template_path: TemplatePath {
            auto_path: None,
            git: Some(format!("{}", template.path().display())),
            branch: Some(String::from("main")),
            tag: None,
            revision: None,
            path: None,
            subfolder: None,
        },
        name: Some(String::from("foobar_project")),
        force: true,
        verbose: true,
        template_values_file: None,
        silent: false,
        continue_on_error: false,
        quiet: false,
        bin: true,
        lib: false,
        ssh_identity: None,
        gitconfig: None,
        define: vec![],
        destination: Some(dir.clone()),
        allow_commands: false,
        overwrite: false,
        skip_submodules: false,
    };

    assert_eq!(
        generate(args_exposed).expect("cannot generate project"),
        dir.join("foobar_project")
    );

    assert!(
        std::fs::read_to_string(dir.join("foobar_project").join("Cargo.toml"))
            .expect("cannot read file")
            .contains("foobar_project")
    );

    let cwd_after = std::env::current_dir().unwrap();
    assert!(cwd_after == cwd_before);
}
