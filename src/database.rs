use crate::common::utils::Config;
use anyhow::{bail, Result};
use std::collections::HashMap;
use std::fs::{copy, create_dir_all, read, rename};
use std::io::{BufWriter, Write};
use std::time::{Duration, SystemTime};
use tempfile::NamedTempFile;

const BACKUP_THRESHOLD: u64 = 24 * 60 * 60;

pub fn save_data(config: &Config, data: &HashMap<String, f32>) -> Result<()> {
    let parent = config.data_path.parent();
    if let Some(path) = parent {
        if !path.exists() {
            create_dir_all(path)?;
        }
        let temp_file = NamedTempFile::new_in(path)?;
        let (temp_file, temp_file_path) = temp_file.keep()?;
        let mut buffer = BufWriter::new(temp_file);
        buffer.write(&bincode::serialize(data)?)?;
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

pub fn load_data(config: &Config) -> Result<HashMap<String, f32>> {
    if !config.data_path.exists() {
        Ok(HashMap::new())
    } else {
        let hash = bincode::deserialize(&read(config.data_path.as_path())?)?;
        Ok(hash)
    }
}

pub fn load_backup(config: &Config) -> Result<HashMap<String, f32>> {
    if config.backup_path.exists() {
        rename(config.backup_path.as_path(), config.data_path.as_path())?;
        return load_data(config);
    }
    Ok(HashMap::new())
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
