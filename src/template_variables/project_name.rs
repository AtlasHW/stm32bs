use heck::{ToKebabCase, ToSnakeCase};

use crate::{interactive, user_parsed_input::UserParsedInput};

pub fn get_project_name(user_parsed_input: &UserParsedInput) -> String {
    let name = user_parsed_input.name();
    match name {
        Some(name) => name.to_string(),
        None => interactive::name().unwrap(),
    }
}

pub fn sanitize_project_name(name: &str) -> String {
    let snake_case_project_name = name.to_snake_case();
    if snake_case_project_name == name {
        snake_case_project_name
    } else {
        name.to_kebab_case()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ProjectType {
    BSPProject,
    EmptyProject,
    DemoProject,    
}
