/// Main file
mod absolute_path;
mod app_log;
mod args;
mod config;
mod git;
mod hooks;
mod interactive;
mod progressbar;
mod project_variables;
mod stm32_device;
mod template;
mod template_filters;
mod template_variables;
mod user_parsed_input;

use args::*;
use app_log::log_env_init;
use config::TemplateConfig;
use config::{Config, CONFIG_FILE_NAME};
use hooks::evaluate_script;
use hooks::{context::RhaiHooksContext, execute_hooks};

use interactive::LIST_SEP;
use liquid::ValueView;
use liquid_core::Value;
use project_variables::{StringKind, VarInfo};
//use serde::de;
use stm32_device::chip_info::ChipInfo;
use stm32_device::chip_info::ChipStatus;
use stm32_device::chip_pn::get_chip_pn;
use stm32_device::device_list::{DeviceList, PRODUCT_LIST_FILE_NAME, PAC_INFO_FILE_NAME};
use template::{create_liquid_object, set_project_variables};
use template_variables::project_name::get_project_name;
use template_variables::ProjectDir;
use user_parsed_input::UserParsedInput;

use anyhow::{bail, Result};
use console::style;
use liquid_core::Object;
use log::{error, info, warn};
use std::vec;
use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
};

fn main() -> Result<()> {
    log_env_init();
    let args = resolve_args();
    if !args.template_path.have_any_path() {
        // Manage the project
        error!("Project manage function is processing!");
    } else {
        // Create a new project from template
        generate(args)?;
    }
    Ok(())
}

/// To generate a cargo project for stm32
pub fn generate(args: AppArgs) -> Result<PathBuf> {
    // mash AppConfig and CLI arguments together into UserParsedInput
    let user_parsed_input = UserParsedInput::try_from_args(&args);
    // copy the template files into a temporary directory
    let temp_dir = template::get_source_template_into_temp(user_parsed_input.location())?;
    let template_dir = template::resolve_template_dir(&temp_dir)?;

    // read configuration in the template
    let mut config =
        Config::from_path(&locate_template_file(CONFIG_FILE_NAME, &template_dir).ok())?;

    let device_list = DeviceList::try_from_path(
        locate_template_file(PRODUCT_LIST_FILE_NAME, &template_dir).unwrap(),
    )?;
    let pac_file = locate_template_file(PAC_INFO_FILE_NAME, &template_dir).unwrap();

    //+++++++++++++++++++++++++++++++++++
    println!("\r\nTemplate config:");
    println!("\ttemplate: {:?}", config.template);
    println!("\tplaceholders: {:?}", config.placeholders);
    println!("\thooks: {:?}", config.hooks);
    println!("\tconditional: {:?}", config.conditional);
    println!("\tdemo: {:?}", config.demo);
    println!("");
    //+++++++++++++++++++++++++++++++++++

    check_cargo_generate_version(&config)?;

    let project_dir =
        expand_template(&template_dir, &mut config, &device_list, &user_parsed_input, &pac_file,)?;


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
    devicelist: &DeviceList,
    user_parsed_input: &UserParsedInput,
    pac_file: &PathBuf,
) -> Result<PathBuf> {
    // create a liquid object with the template variables
    let mut liquid_object = create_liquid_object(user_parsed_input)?;
    let context = RhaiHooksContext {
        liquid_object: liquid_object.clone(),
        allow_commands: user_parsed_input.allow_commands(),
        silent: user_parsed_input.silent(),
        working_directory: template_dir.to_owned(),
        destination_directory: user_parsed_input.destination().to_owned(),
    };

    // run init hooks - these won't have access to `crate_name`/`within_cargo_project`
    // variables, as these are not set yet. Furthermore, if `project-name` is set, it is the raw
    // user input!
    // The init hooks are free to set `project-name` (but it will be validated before further
    // use).
    execute_hooks(&context, &config.get_init_hooks())?;

    let project_name = get_project_name(user_parsed_input);

    // build a supported chip info list
    let chip_pn = get_chip_pn(user_parsed_input, &devicelist).unwrap();
    let chip_info_raw = devicelist.devices.get(&chip_pn).unwrap();
    let chip_info = ChipInfo::try_from_string(chip_pn, chip_info_raw, pac_file).unwrap();
    if user_parsed_input.is_verbose() {
        if chip_info.status == ChipStatus::NRND {
            warn!(
                "{}",
                style(format!("Cation: The chip is NRND ( Not Recommended for New Designs )."))
                    .bold()
                    .red()
            );
        }
        info!("{:?}", chip_info);
    }
    // ++++++++++++++++++++++++++++++++++
    let project_type = interactive::select(
        &vec!["Project with BSP", "Empty Project", "Demo"],
        "🤷 Choose a project type",
        0,
    )?;

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
    match project_type {
        0 => {
            info!("Create a STM32 Project with BSP...");
            bail!("The function is not implemented yet!");
        }
        1 => {
            info!("Create a Empty STM32 Project...");
            // Select files should be included in the template
        }
        2 => {
            info!("Create a STM32 Demo project...");
            let demo_list = config.get_demo_list();
            let demo_list = demo_list.iter().map(|s|s.as_str()).collect::<Vec<&str>>();
            // chooce a demo for the project
            let demo = interactive::select(
                &demo_list,
                "🤷 Choose a demo",
                0,
            )?;
        }
        _ => {
            bail!("Invalid project type selected!");
        }
    }

    let destination = ProjectDir::try_from((&project_name, user_parsed_input))?;

    destination.create(user_parsed_input.overwrite())?;

    set_project_variables(&mut liquid_object, &chip_info, &project_name, project_type)?;

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

    println!("{}", style("Merged!").green());
    add_missing_provided_values(&mut liquid_object, user_parsed_input.template_values())?;

    let context = RhaiHooksContext {
        liquid_object: liquid_object.clone(),
        destination_directory: destination.as_ref().to_owned(),
        ..context
    };

    //+++++++++++++++++++++++++++++++++
    println!("liquid_object: ");
    for item in liquid_object.iter() {
        println!("\t{:15}:\t{:?}", item.clone().0, item.clone().1);
    }
    println!("");
    //+++++++++++++++++++++++++++++++++

    println!("{}", style("execute pre hooks...").red());
    // run pre-hooks
    execute_hooks(&context, &config.get_pre_hooks())?;

    // walk/evaluate the template
    let mut template_config = config.template.take().unwrap_or_default();
    println!("template_config: {template_config:?}"); //+++++++++++++++++++++++++++++++++

    include_files.append(template_config.include.as_mut().unwrap_or(&mut vec![]));
    template::walk_dir(
        &include_files,
        &user_parsed_input,
        template_dir,
        &mut liquid_object,
    )?;
    // run post-hooks
    execute_hooks(&context, &config.get_post_hooks())?;

    // copy the template files into the project directory
    for filename in  &include_files {
        let src_path = template_dir.join(filename);
        let dst_path = destination.as_ref().join(filename);
        if !dst_path.exists() {
            std::fs::create_dir_all(dst_path.parent().unwrap())?;
        }
        std::fs::copy(src_path, dst_path)?;
    }

    config.template.replace(template_config);
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
            if provided_value.is_some() {
                let value = provided_value.unwrap();
                match &slot.var_info {
                    VarInfo::Bool { .. } => {
                        println!("value: {:?}", value);
                        let as_bool = value.parse::<bool>();
                        if as_bool.is_ok() {
                            return Ok(Value::Scalar(as_bool.unwrap().into()));
                        }
                    }
                    VarInfo::String { entry } => match &entry.kind {
                        StringKind::Choices(choices) => {
                            if choices.contains(&value) {
                                return Ok(Value::Scalar(value.into()));
                            }
                        }
                        StringKind::String | StringKind::Text => {
                            if entry.regex.is_none() {
                                return Ok(Value::Scalar(value.into()));
                            } else {
                                let regex = entry.regex.as_ref().unwrap();
                                if regex.is_match(&value) {
                                    return Ok(Value::Scalar(value.into()));
                                }
                            }
                        }
                    },
                    VarInfo::Array { entry } => {
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
                            return Ok(Value::Array(choices_defaults));
                        }
                    }
                }
            }
            interactive::variable(slot)
        })?;

        let placeholders_changed = conditionals
            .iter_mut()
            // filter each conditional config block by trueness of the expression, given the known variables
            .filter_map(|(key, cfg)| {
                evaluate_script::<bool>(liquid_object, key)
                    .ok()
                    .filter(|&r| r)
                    .map(|_| cfg)
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

fn check_cargo_generate_version(template_config: &Config) -> Result<(), anyhow::Error> {
    if let Config {
        template:
            Some(config::TemplateConfig {
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
