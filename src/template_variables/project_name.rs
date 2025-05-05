use anyhow::bail;
use anyhow::Result;
use console::style;
use heck::ToKebabCase;
use log::info;
use log::warn;

use crate::template_config::Config;
use crate::{interactive, user_parsed_input::UserParsedInput};

pub fn get_project_name(user_parsed_input: &UserParsedInput) -> String {
    let name = user_parsed_input.name();
    match name {
        Some(name) => {
            if name != name.to_kebab_case() {
                warn!(
                    "{} `{}` {} `{}`{}",
                    style("Renaming project called").bold(),
                    style(name).bold().yellow(),
                    style("to").bold(),
                    style(&name.to_kebab_case()).bold().green(),
                    style("...").bold()
                );
            }
            name.to_kebab_case()
        }
        None => interactive::name().unwrap(),
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ProjectType {
    BSPProject,
    EmptyProject,
    DemoProject(String),
}

impl std::fmt::Display for ProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectType::BSPProject => write!(f, "Project with BSP"),
            ProjectType::EmptyProject => write!(f, "Empty Project"),
            ProjectType::DemoProject(_) => write!(f, "Demo"),
        }
    }
}

pub fn get_project_type(
    user_parsed_input: &UserParsedInput,
    template_config: &mut Config,
) -> Result<ProjectType> {
    let project_type = user_parsed_input.project_type();
    let demo_name = user_parsed_input.demo_name();
    if let Some(demo_name) = demo_name {
        return Ok(ProjectType::DemoProject(demo_name.to_string()));
    }
    let mut is_demo = false;
    if let Some(project_type_str) = project_type {
        match project_type_str {
            "bsp" => return Ok(ProjectType::BSPProject),
            "empty" => return Ok(ProjectType::EmptyProject),
            "demo" => is_demo = true,
            _ => {
                bail!("Invalid project type: {}", project_type_str);
            }
        };
    }
    let project_type_str = if is_demo {
        "Demo".to_string()
    } else {
        // Ask the user for the project type
        interactive::select(
            &vec!["Project with BSP", "Empty Project", "Demo"],
            "🤷 Choose a project type",
            None,
        )?
    };
    let project_type = match project_type_str.as_str() {
        "Project with BSP" => {
            info!("Create a STM32 Project with BSP...");
            bail!("The function is not implemented yet!");
            #[allow(unreachable_code)]
            ProjectType::BSPProject
        }
        "Empty Project" => {
            info!("Create a Empty STM32 Project...");
            ProjectType::EmptyProject
        }
        "Demo" => {
            info!("Create a STM32 Demo project...");
            let demo_list = template_config.get_demo_list();
            let demo_list = demo_list.iter().map(|s| s.as_str()).collect();
            // chooce a demo for the project
            let demo_name = interactive::select(&demo_list, "🤷 Choose a demo", None)?;
            ProjectType::DemoProject((&demo_name).clone())
        }
        _ => {
            bail!("Invalid project type selected!");
        }
    };
    Ok(project_type)
}
