use crate::common::config::Config;
use anyhow::{bail, Result};
use std::collections::HashMap;
use std::fs::{copy, create_dir_all, read, rename};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use tempfile::NamedTempFile;
use std::ops::{Deref, DerefMut};

const BACKUP_THRESHOLD: u64 = 24 * 60 * 60;

pub struct Database {
    data: HashMap<PathBuf, f32>
}

impl DerefMut for Database {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl Deref for Database {
    type Target = HashMap<PathBuf, f32>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

// TODO: https://github.com/rust-lang/rfcs/issues/814
// impl<T> Drop for Database<T> {
//     fn drop(&mut self) {
//         &self.data
//     }
// }

impl Database<> {
    fn load_data(config: &Config) -> Result<Database<>> {
        if !config.data_path.exists() {
            Ok(Database{data: HashMap::new()})
        } else {
            let hash = bincode::deserialize(&read(&config.data_path)?)?;
            Ok(Database{data: hash})
        }
    }
    
    fn load_backup(config: &Config) -> Result<Database<>> {
        if config.backup_path.exists() {
            rename(config.backup_path.as_path(), config.data_path.as_path())?;
            return Database::load_data(config);
        }
        Ok(Database{data: HashMap::new()})
    }

    pub fn new(config: &Config) -> Result<Database<>> {
        if !config.data_path.exists() {
            Database::load_backup(config)
        } else {
            Database::load_data(config)
        }
    }
    
    pub fn save(&self, config: &Config) -> Result<()> {
        let parent = config.data_path.parent();
        if let Some(path) = parent {
            if !path.exists() {
                create_dir_all(path)?;
            }
            let temp_file = NamedTempFile::new_in(path)?;
            let (temp_file, temp_file_path) = temp_file.keep()?;
            let mut buffer = BufWriter::new(temp_file);
            buffer.write(&bincode::serialize(self.deref())?)?;
            buffer.flush()?;
            rename(temp_file_path, config.data_path.as_path())?;
    
            // create backup file if it doesn't exist or is older than BACKUP_THRESHOLD
            if !config.backup_path.exists()
                || (SystemTime::now().duration_since(config.backup_path.metadata()?.modified()?)?
                    > Duration::from_secs(BACKUP_THRESHOLD))
            {
                copy(config.data_path.as_path(), config.backup_path.as_path())?;
            }
    
            Ok(())
        } else {
            bail!("parent of {} not found", config.data_path.display());
        }
    }

}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    #[ignore]
    fn bincode_layout() {
        assert_eq!(2, 1);
    }
}
