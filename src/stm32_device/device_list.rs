use crate::stm32_device::read_csv::read_csv_file;
use std::{collections::HashMap, path::Path};

pub const PRODUCT_LIST_FILE_NAME: &str = "ProductsList.csv";
pub const PAC_INFO_FILE_NAME: &str = "pac_info.csv";
pub struct DeviceList {
    pub devices: HashMap<String, Vec<String>>,
}

impl DeviceList {
    /// Converts the device list to a vector of device part numbers.
    pub fn to_device_pn(&self) -> Vec<String> {
        self.devices.keys().cloned().collect()
    }

    /// Tries to create a DeviceList from a CSV file.
    pub fn try_from_path<P: AsRef<Path>>(path: P) -> Result<Self, anyhow::Error> {
        let file_path = path.as_ref();
        if file_path.file_name().is_none() {
            return Err(anyhow::anyhow!(
                "File does not have a valid name: {:?}",
                file_path
            ));
        }
        if file_path.extension().unwrap() != "csv" {
            return Err(anyhow::anyhow!("File is not a csv file: {:?}", file_path));
        }
        if !file_path.exists() {
            return Err(anyhow::anyhow!("File does not exist: {:?}", file_path));
        }
        if !file_path.is_file() {
            return Err(anyhow::anyhow!("Path is not a file: {:?}", file_path));
        }
        let data = read_csv_file(&path, 2)
            .unwrap_or_else(|err| panic!("Failed to read CSV file: {:?}", err));
        if data.is_empty() {
            return Err(anyhow::anyhow!("CSV file is empty: {:?}", path.as_ref()));
        }
        Ok(DeviceList::from(data))
    }
}

impl From<Vec<Vec<String>>> for DeviceList {
    fn from(data: Vec<Vec<String>>) -> Self {
        let mut devices = HashMap::new();
        for entry in data {
            if let Some((key, values)) = entry.split_first() {
                devices.insert(key.clone(), values.to_vec());
            }
        }
        DeviceList { devices }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_device_list_from() {
        let data = vec![
            vec![
                "Device1".to_string(),
                "FeatureA".to_string(),
                "FeatureB".to_string(),
            ],
            vec!["Device2".to_string(), "FeatureC".to_string()],
            vec!["Device3".to_string()],
        ];

        let device_list = DeviceList::from(data);

        assert_eq!(device_list.devices.len(), 3);
        assert_eq!(
            device_list.devices.get("Device1"),
            Some(&vec!["FeatureA".to_string(), "FeatureB".to_string()])
        );
        assert_eq!(
            device_list.devices.get("Device2"),
            Some(&vec!["FeatureC".to_string()])
        );
        assert_eq!(
            device_list.devices.get("Device3"),
            Some(&Vec::<String>::new())
        );
    }

    #[test]
    fn test_to_device_pn() {
        let data = vec![
            vec![
                "Device1".to_string(),
                "FeatureA".to_string(),
                "FeatureB".to_string(),
            ],
            vec!["Device2".to_string(), "FeatureC".to_string()],
            vec!["Device3".to_string()],
        ];

        let device_list = DeviceList::from(data);

        let device_pn = device_list.to_device_pn();
        let mut expected_device_pn = vec![
            "Device1".to_string(),
            "Device2".to_string(),
            "Device3".to_string(),
        ];
        expected_device_pn.sort();
        let mut device_pn_sorted = device_pn.clone();
        device_pn_sorted.sort();

        assert_eq!(device_pn_sorted, expected_device_pn);
    }
    #[test]
    fn test_try_from_valid_file() {
        let path = PathBuf::from("/home/atlassong-k/rust/template/ProductsList.csv");

        // Assuming the file exists and contains valid CSV data
        let device_list =
            DeviceList::try_from_path(path).expect("Failed to create DeviceList from file");

        // Perform some basic checks
        assert!(
            !device_list.devices.is_empty(),
            "Device list should not be empty"
        );

        // Example: Check if a specific device exists (adjust based on actual file content)
        assert!(
            device_list.devices.contains_key("STM32C011J4"),
            "STM32C011J4 should exist in the list"
        );

        // Example: Check if a specific device exists (adjust based on actual file content)
        assert!(
            device_list.devices.contains_key("STM32C011D6"),
            "STM32C011D6 should exist in the list"
        );

        // Example: Check if a specific device exists (adjust based on actual file content)
        assert!(
            device_list.devices.contains_key("STM32U083HC"),
            "STM32U083HC should exist in the list"
        );
    }

    #[test]
    fn test_try_from_nonexistent_file() {
        let path = PathBuf::from("/home/atlassong-k/rust/template/NonExistentFile.csv");

        // Assuming the file does not exist
        let result = DeviceList::try_from_path(path);

        assert!(result.is_err(), "Expected an error for a nonexistent file");
    }
}
