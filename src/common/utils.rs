use crate::common::r#match::{match_anywhere, match_consecutive, match_fuzzy};
use crate::common::opts::Opts;
use crate::database::Database;
#[cfg(target_family = "unix")]
use anyhow::bail;
use anyhow::Result;
use const_format::concatcp;
use lazy_static::lazy_static;
use log::LevelFilter;
use log::{debug, info};
use std::borrow::Cow;
use std::cmp::Ordering;
use std::fmt::Display;
use std::iter::Iterator;
use std::path::{Component, Path, PathBuf};

const PKGNAME: &str = env!("CARGO_PKG_NAME");

lazy_static! {
    static ref CWD: PathBuf = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("./"));
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

pub fn setup_logger(opts: &Opts) {
    let mut builder = env_logger::builder();
    #[cfg(not(debug_assertions))]
    let builder = builder.format_timestamp(None).format_module_path(false);
    builder
        .filter_level(into_level(log::LevelFilter::Info as u32 + opts.verbose))
        .parse_default_env()
        .init();
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

/// normalizatize and convert the lowercase drive name to an uppercase one on Windows
pub fn normalize_path(path: &Path) -> PathBuf {
    path.components()
        .map(|x| {
            if let Component::Prefix(prefix) = x {
                // TODO
                #[cfg(feature = "osstring_ascii")]
                {
                    let p = prefix.as_os_str().to_os_string();
                    p.make_ascii_uppercase();
                    p
                }
                #[cfg(not(feature = "osstring_ascii"))]
                {
                    prefix.as_os_str().to_string_lossy().to_ascii_uppercase()
                }
            } else {
                #[cfg(feature = "osstring_ascii")]
                {
                    x.as_os_str().to_os_string()
                }
                #[cfg(not(feature = "osstring_ascii"))]
                {
                    x.as_os_str().to_string_lossy().into_owned()
                }
            }
        })
        .collect()
}

pub fn print_item<T: Display>((path, weight): (T, f32)) {
    info!("{:.2}\t\t{}", weight, path);
}

/// Prints the tab completion menu according to the following format:
/// ```text
///     [needle]__[index]__[possible_match]
/// ```
/// The needle (search pattern) and index are necessary to recreate the results
/// on subsequent calls.
pub fn print_tab_menu<'a>(
    needle: &'a str,
    tab_entries: impl Iterator<Item = &'a (Cow<'a, Path>, f32)>,
    separator: &str,
) {
    for (i, entry) in tab_entries.enumerate() {
        if !entry.0.as_os_str().is_empty() {
            println!(
                "{}{}{}{}{}",
                needle,
                separator,
                i + 1,
                separator,
                entry.0.to_string_lossy()
            );
        }
    }
}

/// If any needles contain an uppercase letter then use case sensitive
/// searching. Otherwise use case insensitive searching.
fn detect_smartcase(needles: &[PathBuf]) -> bool {
    needles.iter().any(|s| {
        s.to_string_lossy()
            .as_ref()
            .chars()
            .any(|c| c.is_ascii_uppercase())
    })
}

/// Return a vec containing matched result.
///
/// Will return `[("".to_string(), 0.0)]` avoid get error in the caller if
/// 1. if found no matched result
/// 2. if needles is empty
pub fn find_matches<'a>(
    data: &'a Database,
    needles: &[PathBuf],
    check_existence: bool,
) -> Vec<(Cow<'a, Path>, f32)> {
    if let Some(needle) = needles.get(0) {
        if needle.as_os_str().is_empty() {
            let mut candidates: Vec<(Cow<Path>, f32)> = Vec::with_capacity(1);
            candidates.push((Cow::Borrowed(Path::new(".")), 0.0));
            return candidates;
        }
    }

    let ignore_case = !detect_smartcase(needles);
    let cwd = std::env::current_dir().expect("Can't find the current directory");
    let is_cwd = |path: &Path| path == cwd;

    let path_exists = if check_existence {
        |path: &Path| path.exists()
    } else {
        |_: &Path| true
    };

    let sort = |a: &(Cow<'a, Path>, f32), b: &(Cow<'a, Path>, f32)| {
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

    let mut ret: Vec<(Cow<'a, Path>, f32)> = match_1
        .into_iter()
        .chain(match_2.into_iter())
        .chain(match_3.into_iter())
        .filter(|(path, _)| !is_cwd(path) && path_exists(path))
        .collect();
    debug!("=> match results: {:?}", ret);

    if ret.len() == 0 {
        ret.push((Cow::Borrowed(Path::new(".")), 0.0));
    }
    ret
}
