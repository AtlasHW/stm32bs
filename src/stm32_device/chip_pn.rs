use console::style;
use log::warn;

use super::device_list::DeviceList;
use crate::interactive;
use crate::project_variables::{StringEntry, StringKind, TemplateSlots, VarInfo};
use crate::user_parsed_input::UserParsedInput;

pub fn get_chip_pn(
    user_parsed_input: &UserParsedInput,
    devicelist: &DeviceList,
) -> Result<String, anyhow::Error> {
    let arg_pn = user_parsed_input.chip_pn();
    let pn = match arg_pn {
        Some(name) => name.to_string(),
        None => interactive::chip_pn().unwrap(),
    };
    let mut pn = pn.to_uppercase();
    let pn_list = devicelist.to_device_pn();
    loop {
        if pn_list.contains(&pn) {
            break Ok(pn);
        }
        let mut prep_pn = vec![];
        for item in &pn_list {
            if pn.starts_with(item) {
                return Ok(item.to_string());
            }
            if item.starts_with(&pn) {
                prep_pn.push(item);
            }
        }
        if prep_pn.len() == 1 {
            return Ok(prep_pn[0].to_string());
        } else if prep_pn.len() > 1 {
            let prompt_args = TemplateSlots {
                prompt: "Which part should be used?".into(),
                var_name: "pn".into(),
                var_info: VarInfo::String {
                    entry: Box::new(StringEntry {
                        default: Some(prep_pn[0].to_string()),
                        kind: StringKind::Choices(
                            prep_pn.into_iter().map(|p| p.to_string()).collect(),
                        ),
                        regex: None,
                    }),
                },
            };
            return interactive::prompt_and_check_variable(&prompt_args);
        }
        warn!(
            "{} \"{}\" {}",
            style("Sorry,").bold().red(),
            style(&pn).bold().yellow(),
            style("is not a valid value for chip_pn").bold().red()
        );
        pn = interactive::chip_pn().unwrap().to_uppercase();
        continue;
    }
}
