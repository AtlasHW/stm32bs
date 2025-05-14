/// Query table cpn from database
pub mod cpn;

/// Query table resource from database
pub mod resource;

use std::cell::RefCell;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::bail;
use anyhow::Result;
//use sqlite;

pub static DB_FILE_PATH: Mutex<RefCell<Option<PathBuf>>> = Mutex::new(RefCell::new(None));

pub fn db_init<T: AsRef<Path>>(path: T) -> Result<()> {
    let db_path: &Path = path.as_ref();
    if !db_path.exists() {
        bail!("Database file is not exists!");
    }
    let binding = DB_FILE_PATH.lock().unwrap();
    binding.borrow_mut().replace(db_path.to_path_buf());
    Ok(())
}
