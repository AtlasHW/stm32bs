use anyhow::Result;
use console::style;
use indexmap::IndexMap;
use liquid_core::model::map::Entry;
use liquid_core::Object;
use liquid_core::{Value, ValueView};
use log::info;
use regex::Regex;
use thiserror::Error;

use crate::{
    config::{Config, TemplateSlotsTable},
    interactive::LIST_SEP,
};

#[derive(Debug)]
pub struct TemplateSlots {
    pub(crate) var_name: String,
    pub(crate) var_info: VarInfo,
    pub(crate) prompt: String,
}

/// Information needed to prompt for a typed value
/// Editor will never have choices
#[derive(Debug, Clone)]
pub enum VarInfo {
    Array { entry: Box<ArrayEntry> },
    Bool { default: Option<bool> },
    String { entry: Box<StringEntry> },
}

#[derive(Debug, Clone)]
pub struct ArrayEntry {
    pub(crate) default: Option<Vec<String>>,
    pub(crate) choices: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct StringEntry {
    pub(crate) default: Option<String>,
    pub(crate) kind: StringKind,
    pub(crate) regex: Option<Regex>,
}

#[derive(Debug, Clone)]
pub enum StringKind {
    Choices(Vec<String>),
    String,
    Text,
}

#[derive(Error, Debug, PartialEq)]
pub enum ConversionError {
    #[error("parameter `{parameter}` of placeholder `{var_name}` should be a `{correct_type}`")]
    WrongTypeParameter {
        var_name: String,
        parameter: String,
        correct_type: String,
    },
    #[error("placeholder `{var_name}` should be a table")]
    InvalidPlaceholderFormat { var_name: String },
    #[error("missing prompt question for `{var_name}`")]
    MissingPrompt { var_name: String },
    #[error("choices array empty for `{var_name}`")]
    EmptyChoices { var_name: String },
    #[error("default is `{default}`, but is not a valid value in choices array `{choices:?}` for `{var_name}`")]
    InvalidDefault {
        var_name: String,
        default: String,
        choices: Vec<String>,
    },
    #[error(
        "invalid type for variable `{var_name}`: `{value}` possible values are `bool`, `string`, `text` and `editor`"
    )]
    InvalidVariableType { var_name: String, value: String },
    #[error("{var_type} type does not support `choices` field")]
    UnsupportedChoices { var_type: String },
    #[error("bool type does not support `regex` field")]
    RegexOnBool { var_name: String },
    #[error("field `{field}` of variable `{var_name}` does not match configured regex")]
    RegexDoesntMatchField { var_name: String, field: String },
    #[error("regex of `{var_name}` is not a valid regex. {error}")]
    InvalidRegex {
        var_name: String,
        regex: String,
        error: regex::Error,
    },
    #[error("placeholder `{var_name}` is not valid as you can't override `project-name`, `crate_name`, `crate_type`, `authors` and `os-arch`")]
    InvalidPlaceholderName { var_name: String },
}

#[derive(Debug, Clone, PartialEq)]
enum SupportedVarValue {
    Bool(bool),
    String(String),
    Array(Vec<String>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SupportedVarType {
    Bool,
    String,
    Text,
    Array,
}

const RESERVED_NAMES: [&str; 6] = [
    "authors",
    "os-arch",
    "project-name",
    "crate_name",
    "crate_type",
    "within_cargo_project",
];

pub fn show_project_variables_with_value(template_object: &Object, config: &Config) {
    let template_slots = config
        .placeholders
        .as_ref()
        .map(try_into_template_slots)
        .unwrap_or_else(|| Ok(IndexMap::new()))
        .unwrap_or_default();

    template_slots
        .iter()
        .filter(|(k, _)| template_object.contains_key(**k))
        .for_each(|(k, v)| {
            let name = v.var_name.as_str();
            let value = template_object.get(*k).unwrap().to_kstr().to_string();
            info!(
                "🔧 {} (placeholder provided by cli argument)",
                style(format!("{name}: {value:?}")).bold(),
            )
        });
}

/// For each defined placeholder, try to add it with value as a variable to the template_object.
pub fn fill_project_variables(
    template_object: &mut Object,
    config: &Config,
    value_provider: impl Fn(&TemplateSlots) -> Result<Value>,
) -> Result<()> {
    let template_slots = config
        .placeholders
        .as_ref()
        .map(try_into_template_slots)
        .unwrap_or_else(|| Ok(IndexMap::new()))?;

    for (&key, slot) in template_slots.iter() {
        match template_object.entry(key.to_string()) {
            Entry::Occupied(_) => {
                // we already have the value from the config file
            }
            Entry::Vacant(entry) => {
                // we don't have the file from the config but we can ask for it
                let value = value_provider(slot)?;
                entry.insert(value);
            }
        }
    }
    Ok(())
}

fn try_into_template_slots(
    TemplateSlotsTable(table): &TemplateSlotsTable,
) -> Result<IndexMap<&str, TemplateSlots>, ConversionError> {
    let mut slots = IndexMap::with_capacity(table.len());
    for (key, values) in table.iter() {
        slots.insert(key.as_str(), try_key_value_into_slot(key, values)?);
    }
    Ok(slots)
}

fn try_key_value_into_slot(
    key: &str,
    values: &toml::Value,
) -> Result<TemplateSlots, ConversionError> {
    if RESERVED_NAMES.contains(&key) {
        return Err(ConversionError::InvalidPlaceholderName {
            var_name: key.to_string(),
        });
    }

    let table = values
        .as_table()
        .ok_or(ConversionError::InvalidPlaceholderFormat {
            var_name: key.to_string(),
        })?;

    let var_type = extract_type(key, table.get("type"))?;
    let regex = extract_regex(key, var_type, table.get("regex"))?;
    let prompt = extract_prompt(key, table.get("prompt"))?;
    let choices = extract_choices(key, var_type, regex.as_ref(), table.get("choices"))?;
    let default_choice = extract_default(
        key,
        var_type,
        regex.as_ref(),
        table.get("default"),
        choices.as_ref(),
    )?;

    let var_info = match var_type {
        SupportedVarType::Bool => VarInfo::Bool {
            default: if let Some(SupportedVarValue::Bool(value)) = default_choice {
                Some(value)
            } else {
                None
            },
        },
        SupportedVarType::String => VarInfo::String {
            entry: Box::new(StringEntry {
                default: if let Some(SupportedVarValue::String(value)) = default_choice {
                    Some(value)
                } else {
                    None
                },
                kind: choices.map_or(StringKind::String, StringKind::Choices),
                regex,
            }),
        },
        SupportedVarType::Array => VarInfo::Array {
            entry: Box::new(ArrayEntry {
                default: if let Some(SupportedVarValue::Array(value)) = default_choice {
                    Some(value)
                } else {
                    None
                },
                choices: choices.unwrap_or_default(),
            }),
        },
        SupportedVarType::Text => VarInfo::String {
            entry: Box::new(StringEntry {
                default: if let Some(SupportedVarValue::String(value)) = default_choice {
                    Some(value)
                } else {
                    None
                },
                kind: StringKind::Text,
                regex,
            }),
        },
    };
    Ok(TemplateSlots {
        var_name: key.to_string(),
        var_info,
        prompt: format!("🤷 {}", style(&prompt).bold()),
    })
}

fn extract_regex(
    var_name: &str,
    var_type: SupportedVarType,
    table_entry: Option<&toml::Value>,
) -> Result<Option<Regex>, ConversionError> {
    match (var_type, table_entry) {
        (SupportedVarType::Bool, Some(_)) => Err(ConversionError::RegexOnBool {
            var_name: var_name.into(),
        }),
        (SupportedVarType::String | SupportedVarType::Text, Some(toml::Value::String(value))) => {
            match Regex::new(value) {
                Ok(regex) => Ok(Some(regex)),
                Err(e) => Err(ConversionError::InvalidRegex {
                    var_name: var_name.into(),
                    regex: value.clone(),
                    error: e,
                }),
            }
        }
        (SupportedVarType::String | SupportedVarType::Text | SupportedVarType::Array, Some(_)) => {
            Err(ConversionError::WrongTypeParameter {
                var_name: var_name.into(),
                parameter: "regex".to_string(),
                correct_type: "String".to_string(),
            })
        }
        (_, None) => Ok(None),
    }
}

fn extract_type(
    var_name: &str,
    table_entry: Option<&toml::Value>,
) -> Result<SupportedVarType, ConversionError> {
    match table_entry {
        None => Ok(SupportedVarType::String),
        Some(toml::Value::String(value)) if value == "string" => Ok(SupportedVarType::String),
        Some(toml::Value::String(value)) if value == "text" => Ok(SupportedVarType::Text),
        Some(toml::Value::String(value)) if value == "bool" => Ok(SupportedVarType::Bool),
        Some(toml::Value::String(value)) if value == "array" => Ok(SupportedVarType::Array),
        Some(toml::Value::String(value)) => Err(ConversionError::InvalidVariableType {
            var_name: var_name.into(),
            value: value.clone(),
        }),
        Some(_) => Err(ConversionError::WrongTypeParameter {
            var_name: var_name.into(),
            parameter: "type".to_string(),
            correct_type: "String".to_string(),
        }),
    }
}

fn extract_prompt(
    var_name: &str,
    table_entry: Option<&toml::Value>,
) -> Result<String, ConversionError> {
    match table_entry {
        Some(toml::Value::String(value)) => Ok(value.clone()),
        Some(_) => Err(ConversionError::WrongTypeParameter {
            var_name: var_name.into(),
            parameter: "prompt".into(),
            correct_type: "String".into(),
        }),
        None => Err(ConversionError::MissingPrompt {
            var_name: var_name.into(),
        }),
    }
}

fn extract_default(
    var_name: &str,
    var_type: SupportedVarType,
    regex: Option<&Regex>,
    table_entry: Option<&toml::Value>,
    choices: Option<&Vec<String>>,
) -> Result<Option<SupportedVarValue>, ConversionError> {
    match (table_entry, choices, var_type) {
        // no default set
        (None, _, _) => Ok(None),
        // default set without choices
        (Some(toml::Value::Boolean(value)), _, SupportedVarType::Bool) => {
            Ok(Some(SupportedVarValue::Bool(*value)))
        }
        (
            Some(toml::Value::String(value)),
            None,
            SupportedVarType::String | SupportedVarType::Text,
        ) => {
            if let Some(reg) = regex {
                if !reg.is_match(value) {
                    return Err(ConversionError::RegexDoesntMatchField {
                        var_name: var_name.into(),
                        field: "default".to_string(),
                    });
                }
            }
            Ok(Some(SupportedVarValue::String(value.clone())))
        }

        // default and choices set
        // No need to check bool because it always has a choices vec with two values
        (
            Some(toml::Value::String(value)),
            Some(choices),
            SupportedVarType::String | SupportedVarType::Text,
        ) => {
            if !choices.contains(value) {
                Err(ConversionError::InvalidDefault {
                    var_name: var_name.into(),
                    default: value.clone(),
                    choices: choices.clone(),
                })
            } else {
                if let Some(reg) = regex {
                    if !reg.is_match(value) {
                        return Err(ConversionError::RegexDoesntMatchField {
                            var_name: var_name.into(),
                            field: "default".to_string(),
                        });
                    }
                }
                Ok(Some(SupportedVarValue::String(value.clone())))
            }
        }
        (Some(toml::Value::Array(defaults)), Some(choices), SupportedVarType::Array) => {
            let default_string_array: Vec<String> = defaults
                .iter()
                .filter(|f| !(f.is_table() && f.is_array()))
                .map(|f| f.as_str().unwrap_or_default().to_string())
                .collect();
            if default_string_array.iter().all(|v| choices.contains(v)) {
                Ok(Some(SupportedVarValue::Array(default_string_array.clone())))
            } else {
                Err(ConversionError::InvalidDefault {
                    var_name: var_name.into(),
                    default: default_string_array.join(LIST_SEP),
                    choices: choices.clone(),
                })
            }
        }

        // Wrong type of variables
        (Some(_), _, type_name) => Err(ConversionError::WrongTypeParameter {
            var_name: var_name.into(),
            parameter: "default".to_string(),
            correct_type: match type_name {
                SupportedVarType::Bool => "bool".to_string(),
                SupportedVarType::String => "string".to_string(),
                SupportedVarType::Text => "text".to_string(),
                SupportedVarType::Array => "array".to_string(),
            },
        }),
    }
}

fn extract_choices(
    var_name: &str,
    var_type: SupportedVarType,
    regex: Option<&Regex>,
    table_entry: Option<&toml::Value>,
) -> Result<Option<Vec<String>>, ConversionError> {
    match (table_entry, var_type) {
        (None, SupportedVarType::Bool | SupportedVarType::Text | SupportedVarType::Array) => {
            Ok(None)
        }
        (Some(_), SupportedVarType::Bool | SupportedVarType::Text) => {
            Err(ConversionError::UnsupportedChoices {
                var_type: format!("{var_type:?}"),
            })
        }
        (Some(toml::Value::Array(arr)), SupportedVarType::String) if arr.is_empty() => {
            Err(ConversionError::EmptyChoices {
                var_name: var_name.into(),
            })
        }
        (Some(toml::Value::Array(arr)), SupportedVarType::Array) => {
            let converted = arr
                .iter()
                .map(|entry| match entry {
                    toml::Value::String(s) => Ok(s.clone()),
                    _ => Err(()),
                })
                .collect::<Vec<_>>();
            if converted.iter().any(|v| v.is_err()) {
                return Err(ConversionError::WrongTypeParameter {
                    var_name: var_name.into(),
                    parameter: "choices".to_string(),
                    correct_type: "String Array".to_string(),
                });
            }

            let strings = converted
                .iter()
                .cloned()
                .map(|v| v.unwrap())
                .collect::<Vec<_>>();
            Ok(Some(strings))
        }
        (Some(_), SupportedVarType::Array) => Err(ConversionError::WrongTypeParameter {
            var_name: var_name.into(),
            parameter: "choices".to_string(),
            correct_type: "String Array".to_string(),
        }),
        (Some(toml::Value::Array(arr)), SupportedVarType::String) => {
            // Checks if very entry in the array is a String
            let converted = arr
                .iter()
                .map(|entry| match entry {
                    toml::Value::String(s) => Ok(s.clone()),
                    _ => Err(()),
                })
                .collect::<Vec<_>>();
            if converted.iter().any(|v| v.is_err()) {
                return Err(ConversionError::WrongTypeParameter {
                    var_name: var_name.into(),
                    parameter: "choices".to_string(),
                    correct_type: "String Array".to_string(),
                });
            }

            let strings = converted
                .iter()
                .cloned()
                .map(|v| v.unwrap())
                .collect::<Vec<_>>();
            // check if regex matches every choice
            if let Some(reg) = regex {
                if strings.iter().any(|v| !reg.is_match(v)) {
                    return Err(ConversionError::RegexDoesntMatchField {
                        var_name: var_name.into(),
                        field: "choices".to_string(),
                    });
                }
            }

            Ok(Some(strings))
        }
        (Some(_), SupportedVarType::String) => Err(ConversionError::WrongTypeParameter {
            var_name: var_name.into(),
            parameter: "choices".to_string(),
            correct_type: "String Array".to_string(),
        }),
        (None, SupportedVarType::String) => Ok(None),
    }
}
