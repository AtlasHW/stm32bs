use anyhow::bail;
use std::path::Path;

use crate::interactive::select;
use crate::stm32_device::read_csv::read_csv_file;

pub struct PAC {
    pub pac_name: String,
    pub version: String,
    pub features: String,
}
impl PAC {
    pub fn from_csv_file<P: AsRef<Path>>(path: P, pn: &String) -> Result<Self, anyhow::Error> {
        let file_path = path.as_ref();
        if file_path.file_name().is_none() {
            bail!("File does not have a valid name: {:?}", file_path);
        }
        if file_path.extension().unwrap() != "csv" {
            bail!("File is not a csv file: {:?}", file_path);
        }
        if !file_path.exists() {
            bail!("File does not exist: {:?}", file_path);
        }
        if !file_path.is_file() {
            bail!("Path is not a file: {:?}", file_path);
        }
        let data = read_csv_file(&path, 0)
            .unwrap_or_else(|err| panic!("Failed to read CSV file: {:?}", err));
        if data.is_empty() {
            bail!("CSV file is empty: {:?}", file_path);
        }
        let mut pac_name = String::new();
        let mut version = String::new();
        let mut features = String::new();
        for row in data {
            if row.len() < 4 {
                continue;
            }
            if row[0] == pn.as_str() {
                if row[1] == "-" {
                    bail!(
                        "Sorry, no PAC is found for {}, Pls update pac_info.csv in template folder",
                        pn
                    );
                } else {
                    pac_name = row[1].to_string();
                }

                if row[2] == "_" {
                    bail!(
                        "Sorry, no version is found for {}, Pls update pac_info.csv in template folder",
                        pn
                    );
                } else {
                    version = row[2].to_string();
                }
                if row[3] == "_" {
                    bail!(
                        "Sorry, no features are found for {}, Pls update pac_info.csv in template folder",
                        pn
                    );
                } else {
                    let column3 = row[3].to_string();
                    if column3.contains(",") {
                        let feature_list = column3.split(",").collect::<Vec<_>>();
                        features = select(
                            &feature_list,
                            "Multy features in PAC, please select a feature you want to use",
                            None,
                        )?;
                    } else {
                        features = column3;
                    }
                    break;
                }
            }
        }

        if features.is_empty() {
            bail!(
                "Sorry, no PAC are found for {}, Pls update pac_info.csv in template folder",
                pn
            );
        }

        Ok(Self {
            pac_name,
            version,
            features,
        })
    }
}
