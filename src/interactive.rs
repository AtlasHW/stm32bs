use crate::project_variables::{ArrayEntry, StringEntry, StringKind, TemplateSlots, VarInfo};
use anyhow::{Ok, Result};
use console::style;
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect, Select};
use liquid_core::Value;
use log::warn;
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
            entry: Box::new(StringEntry {
                default: None,
                kind: StringKind::String,
                regex: Some(valid_ident),
            }),
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
            entry: Box::new(StringEntry {
                default: None,
                kind: StringKind::String,
                regex: Some(valid_ident),
            }),
        },
    };
    prompt_and_check_variable(&project_var)
}

pub fn select(choices: &Vec<&str>, prompt: &str, default: usize) -> Result<usize> {
    if default >= choices.len() - 1 {
        return Err(anyhow::anyhow!("Default index out of bounds"));
    }
    let chosen = Select::with_theme(&ColorfulTheme::default())
        .items(choices)
        .with_prompt(prompt)
        .default(default)
        .interact()?;

    Ok(chosen)
}

pub fn user_question(
    prompt: &String,
    default: &Option<String>,
    kind: &StringKind,
) -> Result<String> {
    match kind {
        StringKind::String => {
            let mut i = Input::<String>::new().with_prompt(prompt);
            if let Some(s) = default {
                i = i.default(s.to_owned());
            }
            i.interact().map_err(Into::<anyhow::Error>::into)
        }
        StringKind::Text => {
            println!("{} (press Ctrl+d to stop reading)", prompt);
            let mut buffer = String::new();
            stdin().read_to_string(&mut buffer)?;
            Ok(buffer)
        }
        StringKind::Choices(_) => {
            unreachable!("StringKind::Choices should be handled in the parent")
        }
    }
}

pub fn prompt_and_check_variable(variable: &TemplateSlots) -> Result<String> {
    match &variable.var_info {
        VarInfo::Bool { default } => handle_bool_input(&variable.prompt, default),
        VarInfo::String { entry } => match &entry.kind {
            StringKind::Choices(choices) => handle_choice_input(choices, entry, &variable.prompt),
            StringKind::String | StringKind::Text => {
                handle_string_input(&variable.var_name, entry, &variable.prompt)
            }
        },
        VarInfo::Array { entry } => handle_multi_select_input(entry, &variable.prompt),
    }
}

pub fn variable(variable: &TemplateSlots) -> Result<Value> {
    let user_entry = prompt_and_check_variable(variable)?;
    match &variable.var_info {
        VarInfo::Bool { .. } => {
            let as_bool = user_entry.parse::<bool>()?;
            Ok(Value::Scalar(as_bool.into()))
        }
        VarInfo::String { .. } => Ok(Value::Scalar(user_entry.into())),
        VarInfo::Array { .. } => {
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

fn handle_string_input(var_name: &str, entry: &StringEntry, prompt: &String) -> Result<String> {
    match &entry.regex {
        Some(regex) => loop {
            let user_entry = user_question(&prompt, &entry.default, &entry.kind)?;
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
        None => Ok(user_question(&prompt, &entry.default, &entry.kind)?),
    }
}

fn handle_choice_input(
    choices: &Vec<String>,
    entry: &StringEntry,
    prompt: &String,
) -> Result<String> {
    let default = entry
        .default
        .as_ref()
        .map_or(0, |default| choices.binary_search(default).unwrap_or(0));

    let chosen = Select::with_theme(&ColorfulTheme::default())
        .items(choices)
        .with_prompt(prompt)
        .default(default)
        .interact()?;

    Ok(choices.index(chosen).to_string())
}

fn handle_multi_select_input(entry: &ArrayEntry, prompt: &String) -> Result<String> {
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
        let default = 5; // Out of bounds

        let result = select(&choices, &prompt.as_str(), default);
        assert!(result.is_err());
    }
}
