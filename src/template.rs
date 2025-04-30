use anyhow::{bail, Context, Result};
use console::style;
use indicatif::ProgressBar;
use liquid::model::KString;
use liquid::{Parser, ParserBuilder};
use liquid_core::{Object, Value};
use std::collections::HashMap;
use std::env;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;
//use walkdir::{DirEntry, WalkDir};

use crate::config::locate_template_configs;
use crate::git;
use crate::git::tmp_dir;
use crate::interactive::prompt_and_check_variable;
use crate::progressbar;
use crate::progressbar::spinner;
use crate::project_variables::{TemplateSlots, VarInfo};
use crate::stm32_device::chip_info::{ChipInfo, HSI_DEFAULT};
use crate::stm32_device::chip_info::FREQ;
use crate::template_filters::*;
use crate::template_variables::{get_authors, /*get_os_arch,*/ Authors};
use crate::template_variables::project_name::ProjectType;
use crate::user_parsed_input::TemplateLocation;
use crate::user_parsed_input::UserParsedInput;

pub fn create_liquid_engine() -> Parser {
    ParserBuilder::with_stdlib()
        .filter(KebabCaseFilterParser)
        .filter(LowerCamelCaseFilterParser)
        .filter(PascalCaseFilterParser)
        .filter(ShoutyKebabCaseFilterParser)
        .filter(ShoutySnakeCaseFilterParser)
        .filter(SnakeCaseFilterParser)
        .filter(TitleCaseFilterParser)
        .filter(UpperCamelCaseFilterParser)
        .build()
        .expect("can't fail due to no partials support")
}

/// create liquid object for the template, and pre-fill it with all known variables
pub fn create_liquid_object(user_parsed_input: &UserParsedInput) -> Result<Object> {
    let authors: Authors = get_authors()?;
    let os_arch = format!("{}-{}", env::consts::OS, env::consts::ARCH);

    let mut liquid_object = Object::new();

    if let Some(name) = user_parsed_input.name() {
        liquid_object.insert("project-name".into(), Value::Scalar(name.to_owned().into()));
    }

    liquid_object.insert("crate_type".into(), Value::Scalar("bin".to_string().into()));
    liquid_object.insert("authors".into(), Value::Scalar(authors.author.into()));
    liquid_object.insert("username".into(), Value::Scalar(authors.username.into()));
    liquid_object.insert("os-arch".into(), Value::Scalar(os_arch.into()));

    Ok(liquid_object)
}

pub fn set_project_variables(
    liquid_object: &mut Object,
    chipinfo: &ChipInfo,
    project_name: &String,
    project_type: &ProjectType,
) -> Result<()> {
    liquid_object.insert(
        "project-name".into(),
        Value::Scalar(project_name.to_owned().into()),
    );
    liquid_object.insert(
        "target".into(),
        Value::Scalar(chipinfo.target.to_owned().into()),
    );
    liquid_object.insert(
        "pac_name".into(),
        Value::Scalar(chipinfo.pac.pac_name.to_owned().into()),
    );
    liquid_object.insert(
        "pac_ver".into(),
        Value::Scalar(chipinfo.pac.version.to_owned().into()),
    );
    liquid_object.insert(
        "pac_feature".into(),
        Value::Scalar(chipinfo.pac.features.to_owned().into()),
    );
    liquid_object.insert("flash_origin".into(), Value::Scalar("0x08000000".into()));
    liquid_object.insert(
        "flash_size".into(),
        Value::Scalar(chipinfo.flash.to_owned().into()),
    );
    liquid_object.insert("ram1_origin".into(), Value::Scalar("0x20000000".into()));
    liquid_object.insert(
        "ram1_size".into(),
        Value::Scalar(chipinfo.ram1.to_owned().into()),
    );
    liquid_object.insert("pn".into(), Value::Scalar(chipinfo.pn.to_owned().into()));

    let freq = match chipinfo.freq {
        FREQ::SINGLE(f) => f,
        FREQ::DUAL(f1, f2) => {
            if f1 > f2 {
                f2
            } else {
                f1
            }
        }
    };
    match project_type {
        ProjectType::BSPProject => {
            liquid_object.insert("frequency".into(), Value::Scalar(freq.into()));
        }
        ProjectType::DemoProject(_) => {
            let hsi_freq = HashMap::from(HSI_DEFAULT);
            let family = chipinfo.family.clone();
            let hsi_freq= hsi_freq.get(family.to_string().as_str()).unwrap();
            liquid_object.insert("HSI_freq".into(), Value::Scalar((*hsi_freq).into()));
            liquid_object.insert("HSI_freq".into(), Value::Scalar(freq.into()));
        }
        _ => {}
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn walk_dir(
    include_list: &Vec<String>,
    template_dir: &Path,
    liquid_object: &mut Object,
) -> Result<()> {
    let liquid_engine = create_liquid_engine();

    let mp = progressbar::new();
    let spinner_style = spinner();

    if include_list.is_empty() {
        bail!("No files to include in the template.");
    }
    let total = include_list.len().to_string();
    for (progress, filename) in include_list.iter().enumerate() {
        let pb = mp.add(ProgressBar::new(50));
        pb.set_style(spinner_style.clone());
        pb.set_prefix(format!(
            "[{:width$}/{}]",
            progress + 1,
            total,
            width = total.len()
        ));
        let filepath = PathBuf::from(template_dir).join(filename);
        pb.set_message(format!("Processing: {filename:?}"));
        if !filepath.exists() {
            bail!(
                "File `{}` does not exist in the template directory.",
                filepath.display()
            );
        }
        match template_process_file(liquid_object, &liquid_engine, &filepath) {
            Ok(new_contents) => {
                pb.inc(25);
                fs::create_dir_all(filepath.parent().unwrap()).unwrap();
                fs::write(filepath, new_contents).with_context(|| {
                    format!(
                        "⛔ {} `{}`",
                        style("Error writing rendered file.").bold().red(),
                        style(filename).bold()
                    )
                })?;
                pb.inc(50);
                pb.finish_with_message(format!("Done: {filename}"));
            }
            Err(e) => {
                bail!(
                    "⛔ Error processing file `{}`: {}",
                    filepath.display(),
                    e.to_string()
                );
            }
        }
    }

    Ok(())
}

fn template_process_file(context: &mut Object, parser: &Parser, file: &Path) -> Result<String> {
    let content =
        fs::read_to_string(file).map_err(|e| liquid_core::Error::with_msg(e.to_string()))?;
    render_string_gracefully(context, parser, content.as_str())
}

pub fn render_string_gracefully(
    context: &mut Object,
    parser: &Parser,
    content: &str,
) -> Result<String> {
    let template = parser.parse(content)?;

    // Liquid engine needs access to the context.
    // At the same time, our own `rhai` liquid filter may also need it, but doesn't have access
    // to the one provided to the liquid engine, thus it has it's own cloned `Arc` for it. These
    // WILL collide and cause the `Mutex` to hang in case the user tries to modify any variable
    // inside a rhai filter script - so we currently clone it, and let any rhai filter manipulate
    // the original. Note that hooks do not run at the same time as liquid, thus they do not
    // suffer these limitations.
    let render_object_view = context.clone();
    let render_result = template.render(&render_object_view);
    match render_result {
        liquid_core::Result::Ok(ctx) => liquid_core::Result::Ok(ctx),
        Err(e) => {
            // handle it gracefully
            let msg = e.to_string();
            println!("render msg:{msg}");
            if msg.contains("requested variable") {
                // so, we miss a variable that is present in the file to render
                let requested_var =
                    regex::Regex::new(r"(?P<p>.*requested\svariable=)(?P<v>.*)").unwrap();
                let captures = requested_var.captures(msg.as_str()).unwrap();
                if let Some(Some(req_var)) = captures.iter().last() {
                    let missing_variable = KString::from(req_var.as_str().to_string());
                    // The missing variable might have been supplied by a rhai filter,
                    // if not, substitute an empty string before retrying
                    let _ = context
                        .entry(missing_variable)
                        .or_insert_with(|| Value::scalar("".to_string()));
                    return render_string_gracefully(context, parser, content);
                }
            }
            // todo: find nice way to have this happening outside of this fn
            // error!(
            //     "{} `{}`",
            //     style("Error rendering template, file has been copied without rendering.")
            //         .bold()
            //         .red(),
            //     style(filename.display()).bold()
            // );
            // todo: end

            // fallback: no rendering, keep things original
            Ok(content.to_string())
        }
    }
}

/// To get source template and put in into a temperary direction
/// TemplateLocation: Local path or git path
pub fn get_source_template_into_temp(template_location: &TemplateLocation) -> Result<TempDir> {
    match template_location {
        TemplateLocation::Git(git) => {
            let result = git::clone_git_template_into_temp(
                git.url(),
                git.branch(),
                git.tag(),
                git.revision(),
                git.identity(),
                git.gitconfig(),
                git.skip_submodules,
            );
            if let Result::Ok(ref temp_dir) = result {
                git::remove_history(temp_dir.path())?;
            };
            result
        }
        TemplateLocation::Path(path) => {
            let temp_dir = tmp_dir()?;
            //copy_files_recursively(path, temp_dir.path(), false)?;
            let mut file_list = fs::read_dir(path)?
                .map(|res| res.map(|e| e.path()))
                .collect::<Result<Vec<_>, std::io::Error>>()?;
            // for filename in  fs::read_dir(path)?
            //     .map(|res| res.map(|e| e.path()))
            //     .collect::<Result<Vec<_>, std::io::Error>>()?
            loop {
                if file_list.is_empty() {
                    break;
                }
                let filename = file_list.pop().unwrap();
                if filename.file_name().unwrap() == ".git" {
                    continue;
                }
                if filename.file_name().unwrap() == ".gitignore" {
                    continue;
                }
                if filename.file_name().unwrap() == "README.md" {
                    continue;
                }

                let mut dst_path = temp_dir.path().join(filename.strip_prefix(path).unwrap());
                if filename.is_dir() {
                    std::fs::create_dir_all(&dst_path)?;
                    file_list.append(
                        &mut fs::read_dir(filename)?
                            .map(|res| res.map(|e| e.path()))
                            .collect::<Result<Vec<_>, std::io::Error>>()?,
                    );
                    continue;
                }
                if !dst_path.exists() {
                    std::fs::create_dir_all(dst_path.parent().unwrap())?;
                }
                if dst_path.file_name().unwrap() == "README.md.liquid" {
                    dst_path.set_file_name("README.md");
                }
                std::fs::copy(filename, dst_path)?;
            }
            git::remove_history(temp_dir.path())?;
            Ok(temp_dir)
        }
    }
}

/// resolve the template location for the actual template to expand
pub fn resolve_template_dir(template_base_dir: &TempDir) -> Result<PathBuf> {
    let template_dir = template_base_dir.path().to_path_buf();
    auto_locate_template_dir(template_dir, &mut |slots| prompt_and_check_variable(slots))
}

/// look through the template folder structure and attempt to find a suitable template.
fn auto_locate_template_dir(
    template_base_dir: PathBuf,
    prompt: &mut impl FnMut(&TemplateSlots) -> Result<String>,
) -> Result<PathBuf> {
    let config_paths = locate_template_configs(&template_base_dir)?;
    match config_paths.len() {
        0 => {
            // No configurations found, so this *must* be a template
            bail!("No template file found. Please check the template folder structure.");
        }
        1 => {
            // A single configuration found, but it may contain multiple configured sub-templates
            let template_dir = &template_base_dir.join(&config_paths[0]);
            Ok(template_dir.to_path_buf())
        }
        _ => {
            // Multiple configurations found, each in different "roots"
            // let user select between them
            let prompt_args = TemplateSlots {
                prompt: "Which template should be expanded?".into(),
                var_name: "Template".into(),
                var_info: VarInfo::Select {
                    choices: config_paths
                        .into_iter()
                        .map(|p| p.display().to_string())
                        .collect(),
                    default: None,
                },
            };
            let path = prompt(&prompt_args)?;

            // recursively retry to resolve the template,
            // until we hit a single or no config, idetifying the final template folder
            let template_dir = &template_base_dir.join(path);
            Ok(template_dir.to_path_buf())
        }
    }
}
