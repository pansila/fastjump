use crate::common::utils::get_app_path;
use const_format::concatcp;
use std::path::PathBuf;

const PKGNAME: &str = env!("CARGO_PKG_NAME");

#[derive(Default)]
pub struct Config {
    pub data_path: PathBuf,
    pub backup_path: PathBuf,
}

impl Config {
    pub fn default() -> Self {
        let data_path: PathBuf = [PKGNAME, concatcp!(PKGNAME, ".db")].iter().collect();
        let backup_path: PathBuf = [PKGNAME, concatcp!(PKGNAME, ".db.bak")].iter().collect();
        let data_home = get_app_path();

        Config {
            data_path: data_home.join(data_path),
            backup_path: data_home.join(backup_path),
        }
    }
}
