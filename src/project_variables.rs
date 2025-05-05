use anyhow::Result;
use console::style;
use indexmap::IndexMap;
use liquid_core::model::map::Entry;
use liquid_core::Object;
use liquid_core::{Value, ValueView};
use log::info;
use regex::Regex;
use thiserror::Error;

use crate::interactive::LIST_SEP;
use crate::template_config::{Config, TemplateSlotsTable};

#[derive(Debug, Clone)]
pub struct TemplateSlots {
    pub(crate) var_name: String,
    pub(crate) var_info: VarInfo,
    pub(crate) prompt: String,
}

/// Information needed to prompt for a typed value
/// Editor will never have choices
#[derive(Debug, Clone)]
pub enum VarInfo {
    MultiSelect {
        entry: MSEntry,
    },
    Select {
        choices: Vec<String>,
        default: Option<String>,
    },
    Bool {
        default: Option<bool>,
    },
    String {
        regex: Option<Regex>,
    },
    Text {
        regex: Option<Regex>,
    },
    Integer {
        range: Option<(i32, i32)>,
    },
}

#[derive(Debug, Clone)]
pub struct MSEntry {
    pub(crate) default: Option<Vec<String>>,
    pub(crate) choices: Vec<String>,
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
    #[error(
        "invalid type for variable `{var_name}`: `{value}` possible values are `bool`, `string`, `text` and `editor`"
    )]
    InvalidVariableType { var_name: String, value: String },
    #[error("{var_type} type does not support `choices` field")]
    UnsupportedChoices { var_type: String },
    #[error("missing `choices` field on `{var_name}`")]
    MissingChoices { var_name: String },
    #[error("bool type does not support `regex` field")]
    RegexOnBool { var_name: String },
    #[error("regex of `{var_name}` is not a valid regex. {error}")]
    InvalidRegex {
        var_name: String,
        regex: String,
        error: regex::Error,
    },
    #[error("placeholder `{var_name}` is not valid as you can't override `project-name`, `crate_name`, `crate_type`, `authors` and `os-arch`")]
    InvalidPlaceholderName { var_name: String },
}

// #[derive(Debug, Clone, PartialEq)]
// enum SupportedVarValue {
//     Bool(bool),
//     String(String),
//     Array(Vec<String>),
// }

#[derive(Debug, Clone, Copy, PartialEq)]
enum SupportedVarType {
    Bool,
    Integer,
    Select,
    String,
    Text,
    MultiSelect,
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

pub fn check_input_project_variables(
    slot: &TemplateSlots,
    provided_value: Option<String>,
) -> Option<Value> {
    let value = match provided_value {
        Some(value) => value,
        None => return None,
    };
    match slot.clone().var_info {
        VarInfo::Bool { .. } => {
            let as_bool = value.parse::<bool>();
            if as_bool.is_ok() {
                return Some(Value::Scalar(as_bool.unwrap().into()));
            }
        }
        VarInfo::Integer { range } => {
            if let Ok(as_int) = value.trim().parse::<i32>() {
                if let Some((min, max)) = range {
                    if as_int >= min && as_int <= max {
                        return Some(Value::Scalar(as_int.into()));
                    }
                } else {
                    return Some(Value::Scalar(as_int.into()));
                }
            }
        }
        VarInfo::String { regex, .. } => {
            if regex.is_none() {
                return Some(Value::Scalar(value.into()));
            } else {
                let regex = regex.as_ref().unwrap();
                if regex.is_match(&value) {
                    return Some(Value::Scalar(value.into()));
                }
            }
        }
        VarInfo::Text { regex, .. } => {
            if regex.is_none() {
                return Some(Value::Scalar(value.into()));
            } else {
                let regex = regex.as_ref().unwrap();
                if regex.is_match(&value) {
                    return Some(Value::Scalar(value.into()));
                }
            }
        }
        VarInfo::Select { choices, .. } => {
            if choices.contains(&value) {
                return Some(Value::Scalar(value.into()));
            }
        }
        VarInfo::MultiSelect { entry } => {
            let choices_defaults = if value.is_empty() {
                Vec::new()
            } else {
                value
                    .split(LIST_SEP)
                    .map(|s| Value::Scalar(s.to_string().into()))
                    .collect()
            };
            if choices_defaults
                .iter()
                .all(|v| entry.choices.contains(&v.to_kstr().to_string()))
            {
                return Some(Value::Array(choices_defaults));
            }
        }
    }
    None
}

pub fn map_to_template_slots(
    table: &IndexMap<String, toml::Value>,
) -> Result<IndexMap<&str, TemplateSlots>, ConversionError> {
    let mut slots = IndexMap::with_capacity(table.len());
    for (key, values) in table.iter() {
        slots.insert(key.as_str(), try_key_value_into_slot(key, values)?);
    }
    Ok(slots)
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
    let range = extract_range(key, var_type, table.get("range"))?;
    let prompt = extract_prompt(key, table.get("prompt"))?;
    let choices = extract_choices(key, var_type, table.get("choices"))?;

    let var_info = match var_type {
        SupportedVarType::Bool => VarInfo::Bool {
            default: if let Some(toml::Value::Boolean(value)) = table.get("default") {
                Some(*value)
            } else {
                None
            },
        },
        SupportedVarType::Integer => VarInfo::Integer { range },
        SupportedVarType::String => VarInfo::String { regex },
        SupportedVarType::Select => VarInfo::Select {
            choices: choices.unwrap_or_default(),
            default: if let Some(toml::Value::String(value)) = table.get("default") {
                Some(value.to_string())
            } else {
                None
            },
        },
        SupportedVarType::MultiSelect => VarInfo::MultiSelect {
            entry: MSEntry {
                default: if let Some(toml::Value::Array(value)) = table.get("default") {
                    let default_string_array: Vec<String> = value
                        .iter()
                        .filter(|f| !(f.is_table() && f.is_array()))
                        .map(|f| f.as_str().unwrap_or_default().to_string())
                        .collect();
                    Some(default_string_array)
                } else {
                    None
                },
                choices: choices.unwrap_or_default(),
            },
        },
        SupportedVarType::Text => VarInfo::Text { regex },
    };
    Ok(TemplateSlots {
        var_name: key.to_string(),
        var_info,
        prompt: format!("🤷 {}", style(&prompt).bold()),
    })
}

fn extract_range(
    var_name: &str,
    var_type: SupportedVarType,
    table_entry: Option<&toml::Value>,
) -> Result<Option<(i32, i32)>, ConversionError> {
    match (var_type, table_entry) {
        (SupportedVarType::Integer, Some(toml::Value::Array(value))) => {
            if value.len() != 2 {
                return Err(ConversionError::WrongTypeParameter {
                    var_name: var_name.into(),
                    parameter: "range".to_string(),
                    correct_type: "Integer Array".to_string(),
                });
            }
            if let (Some(r1), Some(r2)) = (value[0].as_integer(), value[1].as_integer()) {
                if r1 > r2 {
                    return Ok(Some((r2 as i32, r1 as i32)));
                } else {
                    return Ok(Some((r1 as i32, r2 as i32)));
                }
            }
            return Err(ConversionError::WrongTypeParameter {
                var_name: var_name.into(),
                parameter: "range".to_string(),
                correct_type: "Integer Array".to_string(),
            });
        }
        (_, Some(_)) => Err(ConversionError::WrongTypeParameter {
            var_name: var_name.into(),
            parameter: "regex".to_string(),
            correct_type: "String".to_string(),
        }),
        (_, None) => Ok(None),
    }
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
        (_, Some(_)) => Err(ConversionError::WrongTypeParameter {
            var_name: var_name.into(),
            parameter: "regex".to_string(),
            correct_type: "String".to_string(),
        }),
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
        Some(toml::Value::String(value)) if value == "integer" => Ok(SupportedVarType::Integer),
        Some(toml::Value::String(value)) if value == "text" => Ok(SupportedVarType::Text),
        Some(toml::Value::String(value)) if value == "bool" => Ok(SupportedVarType::Bool),
        Some(toml::Value::String(value)) if value == "select" => Ok(SupportedVarType::Select),
        Some(toml::Value::String(value)) if value == "multiselect" => {
            Ok(SupportedVarType::MultiSelect)
        }
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

fn extract_choices(
    var_name: &str,
    var_type: SupportedVarType,
    table_entry: Option<&toml::Value>,
) -> Result<Option<Vec<String>>, ConversionError> {
    match (table_entry, var_type) {
        (
            Some(toml::Value::Array(arr)),
            SupportedVarType::Select | SupportedVarType::MultiSelect,
        ) => {
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
        (Some(_), SupportedVarType::Select | SupportedVarType::MultiSelect) => {
            Err(ConversionError::WrongTypeParameter {
                var_name: var_name.into(),
                parameter: "choices".to_string(),
                correct_type: "String Array".to_string(),
            })
        }
        (None, SupportedVarType::Select | SupportedVarType::MultiSelect) => {
            Err(ConversionError::MissingChoices {
                var_name: var_name.into(),
            })
        }
        (Some(_), _) => Err(ConversionError::UnsupportedChoices {
            var_type: format!("{var_type:?}"),
        }),
        (_, _) => Ok(None),
    }
}
