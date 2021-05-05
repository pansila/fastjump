use crate::common::r#match::{match_anywhere, match_consecutive, match_fuzzy};
#[cfg(target_family = "unix")]
use anyhow::bail;
use anyhow::Result;
use const_format::concatcp;
use lazy_static::lazy_static;
use log::LevelFilter;
use log::{debug, info};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Display;
use std::iter::Iterator;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};

const PKGNAME: &str = env!("CARGO_PKG_NAME");

lazy_static! {
    static ref CWD: PathBuf = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("./"));
}

#[derive(Default)]
pub struct Config {
    pub data_path: PathBuf,
    pub backup_path: PathBuf,
}

/// Copy a file in a directory.
///
/// It will return with an Error if failed.
#[macro_export]
macro_rules! copy_in {
    ( $file:expr, $path:expr ) => {
        std::fs::copy(
            $file,
            $path.join(
                Path::new($file)
                    .file_name()
                    .ok_or(anyhow!("can't find the basename"))?,
            ),
        )
    };
}

/// Convert string arguments into a PathBuf.
// TODO: Support vec, array
#[macro_export]
macro_rules! format_path {
    ( $( $x:expr ),* ) => {
        [$(
            $x,
        )*].iter().collect::<PathBuf>()
    };
}

#[cfg(not(target_os = "windows"))]
fn is_sourced() -> bool {
    match std::env::var(format!("{}_SOURCED", PKGNAME.to_ascii_uppercase()).as_str()).as_deref() {
        Ok("0") | Ok("false") => false,
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn environment_check() -> Result<()> {
    #[cfg(not(target_os = "windows"))]
    if !is_sourced() {
        bail!(format!(
            "Please source the correct {} file in your shell's \
            startup file. For more information, please reinstall {} \
            and read the post installation instructions.",
            PKGNAME, PKGNAME
        ));
    }
    Ok(())
}

impl Config {
    pub fn new() -> Self {
        let data_path: PathBuf = [PKGNAME, concatcp!(PKGNAME, ".db")].iter().collect();
        let backup_path: PathBuf = [PKGNAME, concatcp!(PKGNAME, ".db.bak")].iter().collect();
        let mut config = Config::default();
        let data_home = get_app_path();

        config.data_path = data_home.join(data_path);
        config.backup_path = data_home.join(backup_path);

        config
    }
}

pub fn get_app_path() -> PathBuf {
    if cfg!(test) {
        return (*CWD).clone();
    }

    #[cfg(target_os = "macos")]
    let data_home = shellexpand::tilde("~/Library");
    #[cfg(target_os = "windows")]
    let data_home =
        shellexpand::env("$APPDATA").expect("Can't find the environment variable %APPDATA%");
    #[cfg(target_os = "linux")]
    let data_home =
        shellexpand::env("XDG_DATA_HOME").unwrap_or(shellexpand::tilde("~/.local/share"));
    PathBuf::from(data_home.as_ref())
}

pub fn get_install_path() -> PathBuf {
    if cfg!(test) {
        return (*CWD).clone();
    }

    #[cfg(target_family = "windows")]
    let install_dir = shellexpand::env(concatcp!("$LOCALAPPDATA\\", PKGNAME))
        .expect("Can't find the environment variable %LOCALAPPDATA%");
    #[cfg(target_family = "unix")]
    let install_dir = shellexpand::tilde(concatcp!("~/", PKGNAME));
    PathBuf::from(install_dir.as_ref())
}

pub fn into_level(verbose: u32) -> log::LevelFilter {
    match verbose {
        0 => LevelFilter::Off,
        1 => LevelFilter::Error,
        2 => LevelFilter::Warn,
        3 => LevelFilter::Info,
        4 => LevelFilter::Debug,
        5..=u32::MAX => LevelFilter::Trace,
    }
}

/// Convert the lowercase drive name to an uppercase one on Windows
pub fn normalize_path(path: &str) -> String {
    #[cfg(target_family = "windows")]
    {
        let mut key = path.to_string();
        let mut chars = key.chars();
        let drive = chars.next();
        let semicolon = chars.next();
        if let Some(d) = drive {
            if d.is_ascii_lowercase() && Some(':') == semicolon {
                key.as_mut_str()
                    .get_mut(0..1)
                    .unwrap() // never fail
                    .make_ascii_uppercase();
                debug!("normalize the path from {} to {}", path, key);
            }
        }
        key
    }
    #[cfg(target_family = "unix")]
    path.to_string()
}

pub fn print_item<T: Display>((path, weight): (T, f32)) {
    info!("{:.2}\t\t{}", weight, path);
}

pub fn print_stats(data: &HashMap<String, f32>, data_path: &Path) {
    info!("Weight\t\tPath");
    info!("{}", "-".repeat(80));
    let mut count_vec: Vec<_> = data.iter().collect();
    count_vec.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(Ordering::Equal));
    for (path, weight) in count_vec {
        print_item((path.as_str(), *weight));
    }

    let sum = data.values().sum::<f32>();
    info!("{}", "_".repeat(80));
    info!("{:.2}\t\ttotal weight", sum);
    info!(
        "{:width$}\t\ttotal entries",
        data.len(),
        width = (sum.log10().floor() as usize) + 4
    );

    if let Ok(cwd) = std::env::current_dir() {
        if let Some(key) = cwd.as_path().to_str() {
            info!(
                "{:.2}\t\tcurrent directory weight",
                data.get(&normalize_path(key)).unwrap_or(&0.0)
            );
        }
    }

    info!("");
    info!("database file:\t{}", data_path.to_str().unwrap()); // never fail
}

/// Prints the tab completion menu according to the following format:
/// ```text
///     [needle]__[index]__[possible_match]
/// ```
/// The needle (search pattern) and index are necessary to recreate the results
/// on subsequent calls.
pub fn print_tab_menu<'a>(
    needle: &'a str,
    tab_entries: impl Iterator<Item = &'a (String, f32)>,
    separator: &str,
) {
    for (i, entry) in tab_entries.enumerate() {
        if entry.0 != "" {
            println!("{}{}{}{}{}", needle, separator, i + 1, separator, entry.0,);
        }
    }
}

/// edge case to allow '/' as a valid path
pub fn sanitize<'a>(directory: impl Iterator<Item = &'a String>) -> impl Iterator<Item = &'a str> {
    #[cfg(not(target_os = "windows"))]
    return directory.map(|path| {
        if path == &MAIN_SEPARATOR.to_string() {
            path.as_str()
        } else {
            path.trim_end_matches(MAIN_SEPARATOR)
        }
    });
    #[cfg(target_os = "windows")]
    return directory.map(|path| path.trim_end_matches(MAIN_SEPARATOR));
}

/// If any needles contain an uppercase letter then use case sensitive
/// searching. Otherwise use case insensitive searching.
fn detect_smartcase(needles: &[&str]) -> bool {
    needles
        .iter()
        .any(|&s| s.chars().any(|c| c.is_ascii_uppercase()))
}

/// Return a vec containing matched result.
///
/// Will return `[("".to_string(), 0.0)]` avoid get error in the caller if
/// 1. if found no matched result
/// 2. if needles is empty
pub fn find_matches(
    data: &HashMap<String, f32>,
    needles: &[&str],
    check_existence: bool,
) -> Vec<(String, f32)> {
    if needles.len() == 0 || needles.get(0).unwrap().is_empty() {
        // never fail
        let mut candidates: Vec<(String, f32)> = Vec::with_capacity(1);
        candidates.push(("".to_string(), 0.0));
        return candidates;
    }

    let ignore_case = !detect_smartcase(needles);
    let cwd = std::env::current_dir().expect("Can't find the current directory");
    let is_cwd = |path: &str| Path::new(path) == cwd;

    let path_exists = if check_existence {
        |path: &str| Path::new(path).exists()
    } else {
        |_: &str| true
    };

    let sort = |a: &(String, f32), b: &(String, f32)| {
        let weight =
            b.1.partial_cmp(&a.1)
                .expect("can't compare the two float numbers");
        if weight == Ordering::Equal {
            b.0.cmp(&a.0)
        } else {
            weight
        }
    };

    let mut match_1 = match_consecutive(needles, data, ignore_case);
    let mut match_2 = match_fuzzy(needles, data, ignore_case, None);
    let mut match_3 = match_anywhere(needles, data, ignore_case);

    match_1.sort_unstable_by(sort);
    match_2.sort_unstable_by(sort);
    match_3.sort_unstable_by(sort);

    debug!("match consecutive: {:?}", match_1);
    debug!("match fuzzy: {:?}", match_2);
    debug!("match anywhere: {:?}", match_3);

    let mut ret: Vec<(String, f32)> = match_1
        .into_iter()
        .chain(match_2.into_iter())
        .chain(match_3.into_iter())
        .filter(|(path, _)| !is_cwd(path) && path_exists(path))
        .collect();
    debug!("=> match results: {:?}", ret);

    if ret.len() == 0 {
        ret.push(("".to_string(), 0.0));
    }
    ret
}
