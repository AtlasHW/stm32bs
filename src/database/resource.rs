use anyhow::{bail, Result};

use crate::database;
use crate::database::DB_FILE_PATH;
use crate::stm32_device::chip_info::{ArmCore, ChipInfo};

pub fn get_resource<T: ToString + Clone>(cpn: T) -> Result<ChipInfo> {
    let refname = database::cpn::get_refname(cpn.clone())?;
    let binding = DB_FILE_PATH.lock();
    let binding2 = binding.as_ref().unwrap().borrow();
    let path = binding2.as_ref();
    if path.is_none() {
        bail!("The database has not been initialized!");
    }
    let query_data = refname.clone();
    let query = r###"
        SELECT * 
        from resource, pac_content 
        where resource.pac = pac_content.id 
        and refname = ?;
    "###;
    let db = sqlite::open(path.unwrap()).unwrap();
    let mut sta = db.prepare(query)?;
    sta.bind((1, query_data.as_str()))?;
    if let Ok(sqlite::State::Row) = sta.next() {
        let family = sta.read::<String, _>("family").unwrap();
        let core_str = sta.read::<String, _>("core").unwrap();
        let core2_raw = sta.read::<String, _>("core_second");
        let core = ArmCore::try_from_short(core_str)?;
        let core2 = if let Ok(core2_str) = core2_raw {
            if core2_str.as_str() == "" {
                None
            } else {
                Some(ArmCore::try_from_short(core2_str)?)
            }
        } else {
            None
        };
        let freq = sta.read::<i64, _>("frequency").unwrap();
        let flash = sta.read::<i64, _>("flash").unwrap();
        let ram = sta.read::<i64, _>("ram").unwrap();
        let ccmram = sta.read::<i64, _>("ccmram").unwrap();
        let target = match family.as_str() {
            "STM32F0" | "STM32G0" | "STM32L0" | "STM32C0" | "STM32U0" | "STM32WL3" | "STM32WB0" => {
                "thumbv6m-none-eabi".to_string()
            }
            "STM32F1" | "STM32F2" | "STM32L1" => "thumbv7m-none-eabi".to_string(),
            "STM32F3" | "STM32F4" | "STM32F7" | "STM32G4" | "STM32H7" | "STM32L4" | "STM32L4+"
            | "STM32WB" | "STM32WL" => "thumbv7em-none-eabi".to_string(),
            "STM32L5" | "STM32U5" | "STM32H5" | "STM32WBA" | "STM32N6" | "STM32U3" => {
                "thumbv8m.main-none-eabihf".to_string()
            }
            s => bail!("Family `{}` has not been related rust target!", s),
        };
        let pac_name = sta.read::<String, _>("pac_name").unwrap();
        let pac_ver = sta.read::<String, _>("pac_ver").unwrap();
        let pac_feature = sta.read::<String, _>("pac_feature").unwrap();
        if pac_name.eq("-") {
            bail!("PAC info is absent, pls update database!");
        }
        if pac_ver.eq("-") {
            bail!("PAC info is absent, pls update database!");
        }
        if pac_feature.eq("-") {
            bail!("PAC info is absent, pls update database!");
        }
        Ok(ChipInfo {
            cpn: cpn.to_string(),
            refname: refname.clone(),
            family,
            core,
            core2,
            freq: freq as u32,
            flash: flash as u32,
            ram: ram as u32,
            ccmram: ccmram as u32,
            target,
            pac_name,
            pac_ver,
            pac_feature,
        })
    } else {
        bail!("No record be found!");
    }
}
