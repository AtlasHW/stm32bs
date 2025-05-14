use anyhow::Ok;
use console::style;
use log::warn;

use crate::project_variables::{TemplateSlots, VarInfo};
use crate::user_parsed_input::UserParsedInput;
use crate::{database, interactive};

pub fn get_chip_pn(user_parsed_input: &UserParsedInput) -> Result<String, anyhow::Error> {
    let arg_pn = user_parsed_input.chip_pn();
    let pn = match arg_pn {
        Some(name) => name.to_string(),
        None => interactive::chip_pn().unwrap(),
    };
    let mut pn = pn.to_uppercase();
    loop {
        let list = database::cpn::cpn_query(&pn)?;
        match list.len() {
            1 => {
                return Ok(list.get(0).unwrap().to_string());
            }
            2..30 => {
                let prompt_args = TemplateSlots {
                    prompt: "Which part should be used?".into(),
                    var_name: "pn".into(),
                    var_info: VarInfo::Select {
                        choices: list.into_iter().map(|p| p.to_string()).collect(),
                        default: None,
                    },
                };
                return interactive::prompt_and_check_variable(&prompt_args);
            }
            30.. => {
                warn!(
                    "Include \"{}\"'s P/N over 30, pls input more information",
                    style(&pn).bold().yellow(),
                );
            }
            _ => {
                warn!(
                    "{} \"{}\" {}",
                    style("Sorry,").bold().red(),
                    style(&pn).bold().yellow(),
                    style("is not a valid value for chip_pn").bold().red()
                );
            }
        }
        // if list.len() == 1 {
        //     return Ok(list.get(0).unwrap().to_string());
        // } else if list.len() > 1 && list.len() < 30 {
        //     let prompt_args = TemplateSlots {
        //         prompt: "Which part should be used?".into(),
        //         var_name: "pn".into(),
        //         var_info: VarInfo::Select {
        //             choices: list.into_iter().map(|p| p.to_string()).collect(),
        //             default: None,
        //         },
        //     };
        //     return interactive::prompt_and_check_variable(&prompt_args);
        // } else if list.len() >= 30 {
        //     warn!(
        //         "Include \"{}\"'s P/N over 30, pls input more information",
        //         style(&pn).bold().yellow(),
        //     );
        // } else {
        //     warn!(
        //         "{} \"{}\" {}",
        //         style("Sorry,").bold().red(),
        //         style(&pn).bold().yellow(),
        //         style("is not a valid value for chip_pn").bold().red()
        //     );
        // }

        pn = interactive::chip_pn().unwrap().to_uppercase();
        continue;
    }
}
