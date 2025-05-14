/// Main file
mod absolute_path;
mod app_log;
mod args;
mod database;
mod interactive;
mod progressbar;
mod project_config;
mod project_variables;
mod stm32_device;
mod template;
mod template_config;
mod template_filters;
mod template_variables;
mod user_parsed_input;
mod utils;

use app_log::log_env_init;
use args::*;
use interactive::LIST_SEP;
use liquid::ValueView;
use stm32_device::chip_pn::get_chip_pn;
use template::{create_liquid_object, set_project_variables};
use template_config::TemplateConfig;
use template_config::{Config, CONFIG_FILE_NAME};
use template_variables::project_name::get_project_name;
use template_variables::project_name::get_project_type;
use template_variables::project_name::ProjectType;
use template_variables::ProjectDir;
use user_parsed_input::UserParsedInput;

use anyhow::{bail, Result};
use console::style;
use indexmap::IndexMap;
use liquid_core::model::map::Entry;
use liquid_core::Object;
use log::{error, info};
use std::vec;
use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
};

const DEFAULT_TEMPLATE: &str = "https://github.com/AtlasHW/stm32bs-template-default";

fn main() -> Result<()> {
    log_env_init();
    let args = resolve_args();
    if !args.template_path.have_any_path() {
        if let Ok(config_file) = project_config::check_config_file() {
            // check stm32bs project type
            // if the project is a bsp project, continue
            // else, rise an error
            let project_config = project_config::ProjectConfig::from_path(&config_file)?;
            let prj_type = project_config
                .project
                .as_ref()
                .and_then(|s| s.get("project_type"))
                .and_then(|s| s.as_str());
            if let Some(prj_type) = prj_type {
                if prj_type == "Project with BSP" {
                    info!("Project type: {}", prj_type);
                    bail!("The function is developing, it will come soon!");
                } else {
                    error!("{} is not supported! for the function, you can use generate a BSP project!",
                     prj_type);
                    bail!("Error: Project type is not supported!");
                }
            } else {
                bail!("Config file has been tampered!");
            }
        }
        // cannot find any project config file, to generate a new project
        else {
            let mut args = args;
            args.template_path.git = Some(DEFAULT_TEMPLATE.to_string());
            generate(args)?;
        }
    } else {
        // Create a new project from template
        generate(args)?;
    }
    Ok(())
}

/// To generate a cargo project for stm32
fn generate(args: AppArgs) -> Result<PathBuf> {
    // mash AppConfig and CLI arguments together into UserParsedInput
    let user_parsed_input = UserParsedInput::try_from_args(&args);
    // copy the template files into a temporary directory
    let temp_dir = template::get_source_template_into_temp(user_parsed_input.location())?;
    let template_dir = template::resolve_template_dir(&temp_dir)?;
    // read configuration in the template
    let mut config =
        Config::from_path(&locate_template_file(CONFIG_FILE_NAME, &template_dir).ok())?;
    //Initialize Databas
    database::db_init(template_dir.join("stm32bs.db"))?;
    //let pac_file = locate_template_file(PAC_INFO_FILE_NAME, &template_dir).unwrap();
    check_stm32bs_version(&config)?;
    let project_dir = expand_template(
        &template_dir,
        &mut config,
        &user_parsed_input,
    )?;
    info!(
        "✨ {} {} {}",
        style("Done!").bold().green(),
        style("New project created").bold(),
        style(&project_dir.display()).underlined()
    );

    Ok(project_dir)
}

fn locate_template_file(name: &str, template_folder: impl AsRef<Path>) -> Result<PathBuf> {
    let search_folder = template_folder.as_ref().to_path_buf();
    let file_path = search_folder.join::<&str>(name);
    if file_path.exists() {
        return Ok(file_path);
    } else {
        bail!("{} not found within template", file_path.to_str().unwrap());
    }
}

fn expand_template(
    template_dir: &Path,
    config: &mut Config,
    user_parsed_input: &UserParsedInput,
) -> Result<PathBuf> {
    // create a liquid object with the template variables
    let mut liquid_object = create_liquid_object(user_parsed_input)?;

    let project_name = get_project_name(user_parsed_input);

    // build a supported chip info list
    let chip_pn = get_chip_pn(user_parsed_input)?;
    let chip_info = database::resource::get_resource(&chip_pn)?;
    if user_parsed_input.is_verbose() {
        info!("{:?}", chip_info);
    }

    // This files must be included in the each project:
    // Cargo.toml
    // src/main.rs
    // build.rs
    // .cargo/config.toml
    // memory.x
    let mut include_files: Vec<String> = vec![
        "Cargo.toml".to_string(),
        "src/main.rs".to_string(),
        "build.rs".to_string(),
        ".cargo/config.toml".to_string(),
        "memory.x".to_string(),
    ];

    let project_type = get_project_type(user_parsed_input, config)?;

    match &project_type {
        ProjectType::BSPProject => {}
        ProjectType::EmptyProject => {}
        ProjectType::DemoProject(demo_name) => {
            let demo_file = template_dir
                .join("demo")
                .join((&demo_name).to_string() + ".rs");
            // Copy the demo file to the main.rs
            if demo_file.exists() {
                std::fs::copy(&demo_file, template_dir.join("src").join("main.rs"))?;
            } else {
                bail!("Demo file not found: {}", demo_file.display());
            }
            // expand the variable in the demo file
        }
    };
    let destination = ProjectDir::try_from((&project_name, user_parsed_input))?;
    destination.create(user_parsed_input.overwrite())?;
    set_project_variables(&mut liquid_object, &chip_info, &project_name, &project_type)?;

    info!(
        "🔧 {}",
        style(format!("Destination: {destination} ..."))
            .bold()
            .yellow()
    );
    info!(
        "🔧 {}",
        style(format!("project-name: {project_name} ..."))
            .bold()
            .yellow()
    );
    project_variables::show_project_variables_with_value(&liquid_object, config);

    info!("🔧 {}", style("Generating template ...").bold().yellow());

    // evaluate config for placeholders and and any that are undefined
    fill_placeholders_and_merge_conditionals(
        config,
        &mut liquid_object,
        user_parsed_input.template_values(),
    )?;
    if let ProjectType::DemoProject(demo_name) = &project_type {
        fill_demo_variables(
            config,
            &mut liquid_object,
            user_parsed_input.template_values(),
            demo_name.clone(),
        )?;
    }

    add_missing_provided_values(&mut liquid_object, user_parsed_input.template_values())?;

    // walk/evaluate the template
    let template_config = config.template.take().unwrap_or_default();

    template_config::replenish_include_file(
        template_dir,
        &mut include_files,
        &template_config.include,
    )?;
    template::walk_dir(&include_files, template_dir, &mut liquid_object)?;

    // copy the template files into the project directory
    for filename in &include_files {
        let src_path = template_dir.join(filename);
        let dst_path = destination.as_ref().join(filename);
        if !dst_path.exists() {
            std::fs::create_dir_all(dst_path.parent().unwrap())?;
        }
        std::fs::copy(src_path, dst_path)?;
    }

    // write the project config file
    project_config::write_project_config_file(
        &destination,
        project_type.clone(),
        // BSP settings, for BSP project, not implemented yet
    )?;
    //    config.template.replace(template_config);
    Ok(destination.as_ref().to_owned())
}

/// Try to add all provided `template_values` to the `liquid_object`.
///
/// ## Note:
/// Values for which a placeholder exists, should already be filled by `fill_project_variables`
pub(crate) fn add_missing_provided_values(
    liquid_object: &mut Object,
    template_values: &HashMap<String, toml::Value>,
) -> Result<(), anyhow::Error> {
    template_values.iter().try_for_each(|(k, v)| {
        if liquid_object.contains_key(k.as_str()) {
            return Ok(());
        }
        // we have a value without a slot in the liquid object.
        // try to create the slot from the provided value
        let value = match v {
            toml::Value::String(content) => liquid_core::Value::Scalar(content.clone().into()),
            toml::Value::Boolean(content) => liquid_core::Value::Scalar((*content).into()),
            _ => anyhow::bail!(style(
                "⛔ Unsupported value type. Only Strings and Booleans are supported."
            )
            .bold()
            .red(),),
        };
        liquid_object.insert(k.clone().into(), value);
        Ok(())
    })?;
    Ok(())
}

/// Turn things into strings that can be turned into strings
/// Tables are not allowed and will be ignored
/// arrays are allowed but will be flattened like so
/// \[\[\[\[a,b\],\[\[c\]\]\],\[\[\[d\]\]\]\]\] => "a,b,c,d"
fn extract_toml_string(value: &toml::Value) -> Option<String> {
    match value {
        toml::Value::String(s) => Some(s.clone()),
        toml::Value::Integer(s) => Some(s.to_string()),
        toml::Value::Float(s) => Some(s.to_string()),
        toml::Value::Boolean(s) => Some(s.to_string()),
        toml::Value::Datetime(s) => Some(s.to_string()),
        toml::Value::Array(s) => Some(
            s.iter()
                .filter_map(extract_toml_string)
                .collect::<Vec<String>>()
                .join(LIST_SEP),
        ),
        toml::Value::Table(_) => None,
    }
}

// Evaluate the configuration, adding defined placeholder variables to the liquid object.
fn fill_placeholders_and_merge_conditionals(
    config: &mut Config,
    liquid_object: &mut Object,
    template_values: &HashMap<String, toml::Value>,
) -> Result<()> {
    let mut conditionals = config.conditional.take().unwrap_or_default();
    loop {
        // keep evaluating for placeholder variables as long new ones are added.
        project_variables::fill_project_variables(liquid_object, config, |slot| {
            let provided_value = template_values
                .get(&slot.var_name)
                .and_then(extract_toml_string);
            if let Some(define_value) =
                project_variables::check_input_project_variables(slot, provided_value)
            {
                return Ok(define_value);
            }
            interactive::variable(slot)
        })?;

        let placeholders_changed = conditionals
            .iter_mut()
            // filter each conditional config block by trueness of the expression, given the known variables
            .filter_map(|(key, cfg)| {
                if liquid_object.contains_key(key.as_str()) {
                    let value = liquid_object
                        .get(key.as_str())
                        .unwrap()
                        .as_scalar()
                        .unwrap()
                        .to_bool();
                    match value {
                        Some(t) => {
                            if t {
                                Some(cfg)
                            } else {
                                None
                            }
                        }
                        None => None,
                    }
                } else {
                    None
                }
            })
            .map(|conditional_template_cfg| {
                // append the conditional blocks configuration, returning true if any placeholders were added
                let template_cfg = config.template.get_or_insert_with(TemplateConfig::default);
                if let Some(mut extras) = conditional_template_cfg.include.take() {
                    template_cfg
                        .include
                        .get_or_insert_with(Vec::default)
                        .append(&mut extras);
                }
                if let Some(extra_placeholders) = conditional_template_cfg.placeholders.take() {
                    match config.placeholders.as_mut() {
                        Some(placeholders) => {
                            for (k, v) in extra_placeholders.0 {
                                placeholders.0.insert(k, v);
                            }
                        }
                        None => {
                            config.placeholders = Some(extra_placeholders);
                        }
                    };
                    return true;
                }
                false
            })
            .fold(false, |acc, placeholders_changed| {
                acc | placeholders_changed
            });

        if !placeholders_changed {
            break;
        }
    }

    Ok(())
}

fn fill_demo_variables(
    config: &mut Config,
    liquid_object: &mut Object,
    template_values: &HashMap<String, toml::Value>,
    demo_name: String,
) -> Result<()> {
    let template_slots = config
        .demo
        .as_ref()
        .and_then(|s| s.get(demo_name.as_str()))
        .map(project_variables::map_to_template_slots)
        .unwrap_or_else(|| Ok(IndexMap::new()))?;

    for (&key, slot) in template_slots.iter() {
        match liquid_object.entry(key.to_string()) {
            Entry::Occupied(_) => {
                // we already have the value from the config file
            }
            Entry::Vacant(entry) => {
                // we don't have the file from the config but we can ask for it
                let value = {
                    let provided_value = template_values
                        .get(&slot.var_name)
                        .and_then(extract_toml_string);
                    if let Some(define_value) =
                        project_variables::check_input_project_variables(slot, provided_value)
                    {
                        define_value
                    } else {
                        interactive::variable(slot)?
                    }
                };
                entry.insert(value);
            }
        }
    }
    project_variables::fill_project_variables(liquid_object, config, |slot| {
        let provided_value = template_values
            .get(&slot.var_name)
            .and_then(extract_toml_string);
        if let Some(define_value) =
            project_variables::check_input_project_variables(slot, provided_value)
        {
            return Ok(define_value);
        }
        interactive::variable(slot)
    })?;
    Ok(())
}

fn check_stm32bs_version(template_config: &Config) -> Result<(), anyhow::Error> {
    if let Config {
        template:
            Some(template_config::TemplateConfig {
                cargo_generate_version: Some(requirement),
                ..
            }),
        ..
    } = template_config
    {
        let version = semver::Version::parse(env!("CARGO_PKG_VERSION"))?;
        if !requirement.matches(&version) {
            bail!(
                "⛔ {} {} {} {}",
                style("Required stm32bs version not met. Required:")
                    .bold()
                    .red(),
                style(requirement).yellow(),
                style(" was:").bold().red(),
                style(version).yellow(),
            );
        }
    }
    Ok(())
}
