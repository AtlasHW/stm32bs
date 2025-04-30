use crate::project_variables::MSEntry;
use crate::project_variables::{TemplateSlots, VarInfo};

use anyhow::{Ok, Result};
use console::style;
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect, Select};
use liquid_core::Value;
use log::warn;
use regex::Regex;
use std::{
    io::{stdin, Read},
    ops::Index,
};

pub const LIST_SEP: &str = ",";

pub fn name() -> Result<String> {
    let valid_ident = regex::Regex::new(r"^([a-zA-Z][a-zA-Z0-9_-]+)$")?;
    let project_var = TemplateSlots {
        var_name: "crate_name".into(),
        prompt: "🤷 Project Name".into(),
        var_info: VarInfo::String {
            regex: Some(valid_ident),
        },
    };
    prompt_and_check_variable(&project_var)
}

pub fn chip_pn() -> Result<String> {
    let valid_ident = regex::Regex::new(r"^([Ss][Tt][Mm][3][2][a-zA-Z0-9]{4,11})$")?;
    let project_var = TemplateSlots {
        var_name: "chip_pn".into(),
        prompt: "🤷 Chip Part Number (eg. stm32g071cbt6)".into(),
        var_info: VarInfo::String {
            regex: Some(valid_ident),
        },
    };
    prompt_and_check_variable(&project_var)
}

pub fn select(choices: &Vec<&str>, prompt: &str, default: Option<String>) -> Result<String> {
    handle_choice_input(
        &choices.iter().map(|s| s.to_string()).collect(),
        &default,
        &prompt.to_string(),
    )
}

pub fn user_question(prompt: &String, qtype: usize) -> Result<String> {
    match qtype {
        0 => Input::<String>::new()
            .with_prompt(prompt)
            .interact()
            .map_err(Into::<anyhow::Error>::into),
        1 => {
            println!("{} (press Ctrl+d to stop reading)", prompt);
            let mut buffer = String::new();
            stdin().read_to_string(&mut buffer)?;
            Ok(buffer)
        },
        _ => {
            unreachable!("StringKind::Choices should be handled in the parent")
        }
    }
}

pub fn prompt_and_check_variable(variable: &TemplateSlots) -> Result<String> {
    match &variable.var_info {
        VarInfo::Bool { default } => handle_bool_input(&variable.prompt, default),
        VarInfo::Integer { range } => {
            handle_integer_input(&variable.var_name, range, &variable.prompt)
        }
        VarInfo::String { regex } => {
            handle_string_input(&variable.var_name, regex, &variable.prompt)
        }
        VarInfo::Text { regex } => handle_text_input(&variable.var_name, regex, &variable.prompt),
        VarInfo::Select { choices ,default} => handle_choice_input(choices, default, &variable.prompt),
        VarInfo::MultiSelect { entry } => handle_multi_select_input(entry, &variable.prompt),
    }
}

pub fn variable(variable: &TemplateSlots) -> Result<Value> {
    let user_entry = prompt_and_check_variable(variable)?;
    match &variable.var_info {
        VarInfo::Bool { .. } => {
            let as_bool = user_entry.parse::<bool>()?;
            Ok(Value::Scalar(as_bool.into()))
        },
        VarInfo::Integer { .. } => {
            let as_int = user_entry.trim().parse::<i32>()?;
            Ok(Value::Scalar(as_int.into()))
        },
        VarInfo::String { .. } => Ok(Value::Scalar(user_entry.into())),
        VarInfo::Text { .. } => Ok(Value::Scalar(user_entry.into())),
        VarInfo::Select { .. } => Ok(Value::Scalar(user_entry.into())),
        VarInfo::MultiSelect { .. } => {
            let items = if user_entry.is_empty() {
                Vec::new()
            } else {
                user_entry
                    .split(LIST_SEP)
                    .map(|s| Value::Scalar(s.to_string().into()))
                    .collect()
            };
            Ok(Value::Array(items))
        }
    }
}

fn handle_integer_input(var_name: &str, range: &Option<(i32, i32)>, prompt: &String) -> Result<String> {
    match range {
        Some(range) => loop {
            let user_entry = Input::<i32>::new()
            .with_prompt(prompt)
            .interact()
            .map_err(Into::<anyhow::Error>::into)?;
            if range.0 <= user_entry && user_entry <= range.1 {
                break Ok(user_entry.to_string());
            }
            warn!(
                "{} \"{}\" {}",
                style("Sorry,").bold().red(),
                style(&user_entry).bold().yellow(),
                style(format!("is over the range {range:?} for {var_name}"))
                    .bold()
                    .red()
            );
        },
        None => Ok(Input::<i32>::new().with_prompt(prompt).interact()?.to_string()),
    }
}

fn handle_string_input(var_name: &str, regex: &Option<Regex>, prompt: &String) -> Result<String> {
    match regex {
        Some(regex) => loop {
            let user_entry = user_question(&prompt, 0)?;
            if regex.is_match(&user_entry) {
                break Ok(user_entry);
            }
            warn!(
                "{} \"{}\" {}",
                style("Sorry,").bold().red(),
                style(&user_entry).bold().yellow(),
                style(format!("is not a valid value for {var_name}"))
                    .bold()
                    .red()
            );
        },
        None => Ok(user_question(&prompt, 0)?),
    }
}

fn handle_text_input(var_name: &str, regex: &Option<Regex>, prompt: &String) -> Result<String> {
    match regex {
        Some(regex) => loop {
            let user_entry = user_question(&prompt, 1)?;
            if regex.is_match(&user_entry) {
                break Ok(user_entry);
            }

            warn!(
                "{} \"{}\" {}",
                style("Sorry,").bold().red(),
                style(&user_entry).bold().yellow(),
                style(format!("is not a valid value for {var_name}"))
                    .bold()
                    .red()
            );
        },
        None => Ok(user_question(&prompt, 1)?),
    }
}

fn handle_choice_input(
    choices: &Vec<String>,
    default: &Option<String>,
    prompt: &String,
) -> Result<String> {
    let default = default
        .as_ref()
        .map_or(0, |default| choices.binary_search(default).unwrap_or(0));

    let chosen = Select::with_theme(&ColorfulTheme::default())
        .items(choices)
        .with_prompt(prompt)
        .default(default)
        .interact()?;

    Ok(choices.index(chosen).to_string())
}

fn handle_multi_select_input(entry: &MSEntry, prompt: &String) -> Result<String> {
    let val = {
        let mut selected_by_default = Vec::<bool>::with_capacity(entry.choices.len());
        match &entry.default {
            // if no defaults are provided everything is disselected by default
            None => {
                selected_by_default.resize(entry.choices.len(), false);
            }
            Some(default_choices) => {
                for choice in &entry.choices {
                    selected_by_default.push(default_choices.contains(choice));
                }
            }
        };

        let choice_indices = MultiSelect::with_theme(&ColorfulTheme::default())
            .items(&entry.choices)
            .with_prompt(prompt)
            .defaults(&selected_by_default)
            .interact()?;

        choice_indices
            .iter()
            .filter_map(|idx| entry.choices.get(*idx))
            .cloned()
            .collect::<Vec<String>>()
            .join(LIST_SEP)
    };
    Ok(val)
}

fn handle_bool_input(prompt: &String, default: &Option<bool>) -> Result<String> {
    let choices = [false.to_string(), true.to_string()];
    let chosen = Select::with_theme(&ColorfulTheme::default())
        .items(&choices)
        .with_prompt(prompt)
        .default(usize::from(default.unwrap_or(false)))
        .interact()?;

    Ok(choices.index(chosen).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_out_of_bounds_default() {
        let choices = vec!["Option 1", "Option 2", "Option 3"];
        let prompt = "Select an option".to_string();
        let default = "Option 2".to_string(); // Out of bounds

        let result = select(&choices, &prompt.as_str(), default.into());
        assert!(result.is_err());
    }
}
