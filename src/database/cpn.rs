use anyhow::{bail, Result};

use crate::database::DB_FILE_PATH;

pub fn cpn_query<T: ToString>(cpn: T) -> Result<Vec<String>> {
    let binding = DB_FILE_PATH.lock();
    let binding2 = binding.as_ref().unwrap().borrow();
    let path = binding2.as_ref();
    if path.is_none() {
        bail!("The database has not been initialized!");
    }
    let query_data = format!("%{}%", cpn.to_string());
    let query = "select cpn from cpn where cpn like ?;";
    let db = sqlite::open(path.unwrap()).unwrap();
    let mut sta = db.prepare(query)?;
    sta.bind((1, query_data.as_str()))?;
    let mut list: Vec<String> = Vec::new();
    while let Ok(sqlite::State::Row) = sta.next() {
        let cpn = sta.read::<String, _>("cpn").unwrap();
        list.push(cpn);
    }
    Ok(list)
}

pub fn get_refname<T: ToString>(cpn: T) -> Result<String> {
    let binding = DB_FILE_PATH.lock();
    let binding2 = binding.as_ref().unwrap().borrow();
    let path = binding2.as_ref();
    if path.is_none() {
        bail!("The database has not been initialized!");
    }
    let query_data = cpn.to_string();
    let query = "select refname from cpn where cpn = ?;";
    let db = sqlite::open(path.unwrap()).unwrap();
    let mut sta = db.prepare(query)?;
    sta.bind((1, query_data.as_str()))?;
    if let Ok(sqlite::State::Row) = sta.next() {
        let refname = sta.read::<String, _>("refname").unwrap();
        Ok(refname)
    } else {
        bail!("No record be found!");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::db_init;

    #[test]
    fn test_cpn_query_valid() {
        // Call the function and expect an error
        let result = cpn_query("test_cpn");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "The database has not been initialized!"
        );

        // Call the function and expect an error
        let result = get_refname("test_cpn");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "The database has not been initialized!"
        );

        // Create a mock database and table
        let db = sqlite::open("test.db").unwrap();
        db.execute(
            "
            CREATE TABLE IF NOT EXISTS cpn (cpn TEXT);
            INSERT INTO cpn (cpn) VALUES ('test_cpn1'), ('test_cpn2');
            ",
        )
        .unwrap();

        db_init("test.db").unwrap();

        // Call the function
        let result = cpn_query("test_cpn").unwrap();

        // Assert the result
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"test_cpn1".to_string()));
        assert!(result.contains(&"test_cpn2".to_string()));

        // Clean up
        std::fs::remove_file("test.db").unwrap();

        // Create a mock database and table
        let db = sqlite::open("test.db").unwrap();
        db.execute(
            "
            CREATE TABLE IF NOT EXISTS cpn (cpn TEXT);
            ",
        )
        .unwrap();

        db_init("test.db").unwrap();

        // Call the function
        let result = cpn_query("nonexistent_cpn").unwrap();

        // Assert the result
        assert!(result.is_empty());

        // Clean up
        std::fs::remove_file("test.db").unwrap();

        // Create a mock database and table
        let db = sqlite::open("test.db").unwrap();
        db.execute(
            "
            CREATE TABLE IF NOT EXISTS cpn (cpn TEXT, refname TEXT);
            INSERT INTO cpn (cpn, refname) VALUES ('test_cpn1', 'refname1'), ('test_cpn2', 'refname2');
            ",
        )
        .unwrap();

        db_init("test.db").unwrap();

        // Call the function
        let result = get_refname("test_cpn1").unwrap();

        // Assert the result
        assert_eq!(result, "refname1".to_string());

        // Clean up
        std::fs::remove_file("test.db").unwrap();

        // Create a mock database and table
        let db = sqlite::open("test.db").unwrap();
        db.execute(
            "
            CREATE TABLE IF NOT EXISTS cpn (cpn TEXT, refname TEXT);
            ",
        )
        .unwrap();

        db_init("test.db").unwrap();

        // Call the function
        let result = get_refname("nonexistent_cpn");

        // Assert the result
        assert!(result.is_err());

        // Clean up
        std::fs::remove_file("test.db").unwrap();
    }
}
