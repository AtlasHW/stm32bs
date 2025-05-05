mod helpers;

// test modules go here
mod basics;
mod demo;
mod git;
mod git_instead_of;
#[cfg(e2e_tests_with_ssh_key)]
mod git_over_ssh;
mod project_name;
mod template_config_file;
mod template_filters;
