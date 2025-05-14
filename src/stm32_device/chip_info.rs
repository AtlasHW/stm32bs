use anyhow::bail;
use anyhow::Result;
use core::fmt;
//use std::path::Path;

///! This module contains the ChipInfo struct and its associated methods.
///! It is used to parse the STM32 chip information from a CSV file.
///! The ChipInfo struct contains various fields such as part number, family, description,
///! status, package, core, frequency, FPU, co_type, co_freq, flash, RAM1, RAM2, RAM3,
///! and target, and pac information.
pub struct ChipInfo {
    pub cpn: String,
    pub refname: String,
    pub family: String,
    //    pub description: String,
    //    pub status: ChipStatus,
    //    pub package: String,
    pub core: ArmCore,
    pub core2: Option<ArmCore>,
    pub freq: u32,
    //    pub fpu: bool,
    //    pub co_type: Option<String>,
    //    pub co_freq: Option<u32>,
    pub flash: u32,
    pub ram: u32,
    pub ccmram: u32,
    pub target: String,
    pub pac_name: String,
    pub pac_ver: String,
    pub pac_feature: String,
}

impl std::fmt::Display for ChipInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CPN: {}\r\n\trefname: {}\r\n\tFamily: {}\r\n\tCore: {:?}\r\n\t\
            Freq: {:?}\r\n\tFlash: {}\r\n\tRAM: {}\r\n\tCCMRAM: {}\r\n\t\
            Target: {}\r\n\t\tPAC: {}\r\n\t\tver: {}\r\n\t\tfeature: {}",
            self.cpn,
            self.refname,
            self.family,
            self.core,
            self.freq,
            self.flash,
            self.ram,
            self.ccmram,
            self.target,
            self.pac_name,
            self.pac_ver,
            self.pac_feature,
        )
    }
}

impl std::fmt::Debug for ChipInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CPN: {}\r\n\trefname: {}\r\n\tFamily: {}\r\n\tCore: {:?}\r\n\t\
            Freq: {:?}\r\n\tFlash: {}\r\n\tRAM: {}\r\n\tCCMRAM: {}\r\n\t\
            Target: {}\r\n\t\tPAC: {}\r\n\t\tver: {}\r\n\t\tfeature: {}",
            self.cpn,
            self.refname,
            self.family,
            self.core,
            self.freq,
            self.flash,
            self.ram,
            self.ccmram,
            self.target,
            self.pac_name,
            self.pac_ver,
            self.pac_feature,
        )
    }
}

// Assuming ArmCore is defined elsewhere in your project
#[derive(Debug, Clone, Copy)]
pub enum ArmCore {
    CortexM0Plus,
    CortexM0,
    CortexM3,
    CortexM4,
    CortexM7,
    CortexM33,
    CortexM55,
}

impl ToString for ArmCore {
    fn to_string(&self) -> String {
        match self {
            ArmCore::CortexM0 => "Cortex-M0".to_string(),
            ArmCore::CortexM0Plus => "Cortex-M0+".to_string(),
            ArmCore::CortexM3 => "Cortex-M3".to_string(),
            ArmCore::CortexM4 => "Cortex-M4".to_string(),
            ArmCore::CortexM7 => "Cortex-M7".to_string(),
            ArmCore::CortexM33 => "Cortex-M33".to_string(),
            ArmCore::CortexM55 => "Cortex-M55".to_string(),
        }
    }
}

impl ArmCore {
    pub fn try_from_short<T: ToString>(data: T) -> Result<ArmCore> {
        match data.to_string().as_str() {
            "0" => Ok(ArmCore::CortexM0),
            "0+" => Ok(ArmCore::CortexM0Plus),
            "3" => Ok(ArmCore::CortexM3),
            "4" => Ok(ArmCore::CortexM4),
            "7" => Ok(ArmCore::CortexM7),
            "33" => Ok(ArmCore::CortexM33),
            "55" => Ok(ArmCore::CortexM55),
            s => bail!("`{}` is unknown core type!", s),
        }
    }
}

pub const HSI_DEFAULT: [(&str, u32); 16] = [
    ("STM32C0", 48_000_000),
    ("STM32F0", 8_000_000),
    ("STM32F1", 8_000_000),
    ("STM32F2", 16_000_000),
    ("STM32F3", 8_000_000),
    ("STM32F4", 16_000_000),
    ("STM32F7", 16_000_000),
    ("STM32H7", 64_000_000),
    ("STM32L0", 8_000_000),
    ("STM32L1", 16_000_000),
    ("STM32L4", 16_000_000),
    ("STM32L5", 16_000_000),
    ("STM32G0", 16_000_000),
    ("STM32G4", 16_000_000),
    ("STM32WB", 16_000_000),
    ("STM32WL", 48_000_000),
];
