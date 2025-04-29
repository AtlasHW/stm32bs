use core::fmt;
use std::path::Path;

use crate::stm32_device::pac::PAC;

///! This module contains the ChipInfo struct and its associated methods.
///! It is used to parse the STM32 chip information from a CSV file.
///! The ChipInfo struct contains various fields such as part number, family, description,
///! status, package, core, frequency, FPU, co_type, co_freq, flash, RAM1, RAM2, RAM3,
///! and target, and pac information.
pub struct ChipInfo {
    pub pn: String,
    pub family: STM32Family,
    pub description: String,
    pub status: ChipStatus,
    pub package: String,
    pub core: ArmCore,
    pub freq: FREQ,
    pub fpu: bool,
    pub co_type: Option<String>,
    pub co_freq: Option<u32>,
    pub flash: u32,
    pub ram1: u32,
    pub ram2: u32,
    pub ram3: u32,
    pub target: String,
    pub pac: PAC,
}

impl std::fmt::Display for ChipInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PN: {}, Family: {}, Description: {}, Status: {:?}, Package: {}, Core: {:?}, Freq: {:?}, FPU: {}, CO Type: {:?}, CO Freq: {:?}, Flash: {}, RAM1: {}, RAM2: {}, RAM3: {}, Target: {}",
            self.pn, self.family, self.description, self.status, self.package, self.core, self.freq, self.fpu, self.co_type, self.co_freq, self.flash, self.ram1, self.ram2, self.ram3, self.target)
    }
}

impl std::fmt::Debug for ChipInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PN: {}\r\n\tFamily: {}\r\n\tDescription: {}\r\n\tStatus: {:?}\r\n\tPackage: {}\r\n\tCore: {:?}\r\n\tFreq: {:?}\r\n\tFPU: {}\r\n\tCO Type: {:?}\r\n\tCO Freq: {:?}\r\n\tFlash: {}\r\n\tRAM1: {}\r\n\tRAM2: {}\r\n\tRAM3: {}\r\n\tTarget: {}",
            self.pn, self.family, self.description, self.status, self.package, self.core, self.freq, self.fpu, self.co_type, self.co_freq, self.flash, self.ram1, self.ram2, self.ram3, self.target)
    }
}

impl ChipInfo {
    pub fn try_from_string(
        pn: String,
        info: &Vec<String>,
        pac_file: &Path,
    ) -> Result<Self, String> {
        if info.len() != 66 {
            return Err("Insufficient information provided".to_string());
        }
        let family = match pn.split_at(7).0 {
            "STM32F0" => STM32Family::STM32F0,
            "STM32F1" => STM32Family::STM32F1,
            "STM32F2" => STM32Family::STM32F2,
            "STM32F3" => STM32Family::STM32F3,
            "STM32F4" => STM32Family::STM32F4,
            "STM32F7" => STM32Family::STM32F7,
            "STM32H7" => STM32Family::STM32H7,
            "STM32L0" => STM32Family::STM32L0,
            "STM32L1" => STM32Family::STM32L1,
            "STM32L4" => STM32Family::STM32L4,
            "STM32L5" => STM32Family::STM32L5,
            "STM32G0" => STM32Family::STM32G0,
            "STM32G4" => STM32Family::STM32G4,
            "STM32WB" => STM32Family::STM32WB,
            "STM32WL" => STM32Family::STM32WL,
            _ => return Err("Unknown STM32 family".to_string()),
        };

        let description = info[0].clone();

        let status = match info[1].as_str() {
            "Active" => ChipStatus::Active,
            "NRND" => ChipStatus::NRND,
            "Evaluation" => ChipStatus::Evaluation,
            _ => ChipStatus::Unknown,
        };

        let package = info[2].clone();

        let core = match info[3].as_str() {
            "Arm Cortex-M0" => ArmCore::CortexM0,
            "Arm Cortex-M0+" => ArmCore::CortexM0Plus,
            "Arm Cortex-M3" => ArmCore::CortexM3,
            "Arm Cortex-M4" => ArmCore::CortexM4,
            "Arm Cortex-M7" => ArmCore::CortexM7,
            "Arm Cortex-M33" => ArmCore::CortexM33,
            "Arm Cortex-M55" => ArmCore::CortexM55,
            "Arm Cortex-M4, Arm Cortex-M7" => ArmCore::CortexM4M7,
            _ => return Err("Unknown ARM core".to_string()),
        };

        let freq_temp: Vec<_> = info[4].split(',').collect();
        let freq = if freq_temp.len() == 1 {
            FREQ::SINGLE(
                freq_temp[0]
                    .trim()
                    .parse::<u32>()
                    .map_err(|_| "Invalid frequency".to_string())?,
            )
        } else if freq_temp.len() == 2 {
            println!("freq_temp: {:?}", freq_temp);
            FREQ::DUAL(
                freq_temp[0]
                    .trim()
                    .parse::<u32>()
                    .map_err(|_| "Invalid frequency".to_string())?,
                freq_temp[1]
                    .trim()
                    .parse::<u32>()
                    .map_err(|_| "Invalid frequency".to_string())?,
            )
        } else {
            return Err("Invalid frequency format".to_string());
        };
        let fpu = match info[5].as_str() {
            "Single-precision FPU" => true,
            "Double-precision FPU" => true,
            _ => false,
        };
        let co_type = if info[6].is_empty() || info[6].eq("-") {
            None
        } else {
            Some(info[4].clone())
        };
        let co_freq = match info[7].parse::<u32>() {
            Ok(freq) => Some(freq),
            Err(_) => None,
        };
        let flash = match info[9].parse::<u32>() {
            Ok(flash) => flash,
            Err(_) => 0,
        };
        let ram1 = match info[12].parse::<u32>() {
            Ok(ram) => ram,
            Err(_) => 0,
        };
        let ram2 = match info[13].parse::<u32>() {
            Ok(ram) => ram,
            Err(_) => 0,
        };
        let ram3 = match info[11].parse::<u32>() {
            Ok(ram) => ram,
            Err(_) => 0,
        };
        let target = match core {
            ArmCore::CortexM0 => "thumbv6m-none-eabi".to_string(),
            ArmCore::CortexM0Plus => "thumbv6m-none-eabi".to_string(),
            ArmCore::CortexM3 => "thumbv7m-none-eabi".to_string(),
            ArmCore::CortexM4 => {
                if fpu {
                    "thumbv7em-none-eabi".to_string()
                } else {
                    "thumbv7m-none-eabi".to_string()
                }
            }
            ArmCore::CortexM7 => {
                if fpu {
                    "thumbv7em-none-eabi".to_string()
                } else {
                    "thumbv7m-none-eabi".to_string()
                }
            }
            ArmCore::CortexM33 => {
                if fpu {
                    "thumbv8m.main-none-eabi".to_string()
                } else {
                    "thumbv8m.main-none-eabihf".to_string()
                }
            }
            ArmCore::CortexM55 => {
                if fpu {
                    "thumbv8m.main-none-eabi".to_string()
                } else {
                    "thumbv8m.main-none-eabihf".to_string()
                }
            }
            ArmCore::CortexM4M7 => {
                if fpu {
                    "thumbv7em-none-eabi".to_string()
                } else {
                    "thumbv7m-none-eabi".to_string()
                }
            }
        };

        let pac = PAC::from_csv_file(pac_file, &pn).unwrap();

        Ok(ChipInfo {
            pn,
            family,
            description,
            status,
            package,
            core,
            freq,
            fpu,
            co_type,
            co_freq,
            flash,
            ram1,
            ram2,
            ram3,
            target,
            pac,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChipStatus {
    Active,
    NRND,
    Evaluation,
    Unknown,
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
    CortexM4M7,
}

#[derive(Debug, Clone, Copy)]
pub enum STM32Family {
    STM32F0,
    STM32F1,
    STM32F2,
    STM32F3,
    STM32F4,
    STM32F7,
    STM32H7,
    STM32L0,
    STM32L1,
    STM32L4,
    STM32L5,
    STM32G0,
    STM32G4,
    STM32WB,
    STM32WL,
}
impl STM32Family {
    pub fn to_string(&self) -> String {
        match self {
            STM32Family::STM32F0 => "STM32F0".to_string(),
            STM32Family::STM32F1 => "STM32F1".to_string(),
            STM32Family::STM32F2 => "STM32F2".to_string(),
            STM32Family::STM32F3 => "STM32F3".to_string(),
            STM32Family::STM32F4 => "STM32F4".to_string(),
            STM32Family::STM32F7 => "STM32F7".to_string(),
            STM32Family::STM32H7 => "STM32H7".to_string(),
            STM32Family::STM32L0 => "STM32L0".to_string(),
            STM32Family::STM32L1 => "STM32L1".to_string(),
            STM32Family::STM32L4 => "STM32L4".to_string(),
            STM32Family::STM32L5 => "STM32L5".to_string(),
            STM32Family::STM32G0 => "STM32G0".to_string(),
            STM32Family::STM32G4 => "STM32G4".to_string(),
            STM32Family::STM32WB => "STM32WB".to_string(),
            STM32Family::STM32WL => "STM32WL".to_string(),
        }
    }
}
impl std::fmt::Display for STM32Family {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FREQ {
    SINGLE(u32),
    DUAL(u32, u32),
}

pub const HSI_DEFAULT: [(&str, u32); 15] = [
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_freq_single() {
        let freq = FREQ::SINGLE(72);
        match freq {
            FREQ::SINGLE(value) => assert_eq!(value, 72),
            _ => panic!("Expected FREQ::SINGLE"),
        }
    }

    #[test]
    fn test_freq_dual() {
        let freq = FREQ::DUAL(72, 48);
        match freq {
            FREQ::DUAL(value1, value2) => {
                assert_eq!(value1, 72);
                assert_eq!(value2, 48);
            }
            _ => panic!("Expected FREQ::DUAL"),
        }
    }

    #[test]
    fn test_freq_debug_format() {
        let freq_single = FREQ::SINGLE(72);
        let freq_dual = FREQ::DUAL(72, 48);
        assert_eq!(format!("{:?}", freq_single), "SINGLE(72)");
        assert_eq!(format!("{:?}", freq_dual), "DUAL(72, 48)");
    }
}
