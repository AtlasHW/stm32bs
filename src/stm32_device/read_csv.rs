use csv::Reader;
use std::path::Path;
use anyhow::Result;

pub fn read_csv_file<P: AsRef<Path>>(path: P, skip_lines: usize) -> Result<Vec<Vec<String>>> {
    let mut rdr = Reader::from_path(path)?;
    let mut records = Vec::new();

    // Skip the first x rows, excluding the header
    for result in rdr.records().skip(skip_lines) {
        let record = result?;
        records.push(record.iter().map(|s| s.to_string()).collect());
    }
    Ok(records)
}

#[cfg(test)]
mod read_csv_tests {
    use super::*;

    #[test]
    fn test_read_csv() {
        let file_path = "/home/atlassong-k/rust/template/ProductsList.csv"; // Replace with your CSV file path
        assert!(
            read_csv_file(file_path, 10).is_ok(),
            "Failed to read the CSV file"
        );
    }
}
