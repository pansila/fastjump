use clap::{AppSettings, Clap};
use std::ffi::OsStr;
use std::path::PathBuf;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");

fn toggle_bool(i: u64) -> bool {
    i > 0
}

// TODO: test
fn my_from_os_str(path: &OsStr) -> Option<Option<PathBuf>> {
    dbg!(path);
    if path.is_empty() {
        return Some(None);
    }
    Some(Some(PathBuf::from(path)))
}

/// Jump to any directory fast and smart
#[derive(Clap)]
#[clap(version = VERSION, author = AUTHOR)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
    /// The directory to jump to, composed of parts of a path
    // pub paths: Option<Vec<String>>,
    // workaround for zero positional argument
    #[clap(requires_if("bar", "increase"), parse(from_os_str))]
    pub paths: Vec<PathBuf>,
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: u32,
    /// Add a path with a default weight
    #[clap(short, long, value_name = "path", parse(from_os_str))]
    pub add: Option<PathBuf>,
    /// Increase the current directory weight
    #[clap(short, long, value_name = "weight")]
    pub increase: Option<f32>,
    /// Decrease the current directory weight
    #[clap(short, long, value_name = "weight")]
    pub decrease: Option<f32>,
    /// Used for tab completion
    #[clap(long, parse(from_occurrences = toggle_bool))]
    pub complete: bool,
    /// Remove non-existent paths from database
    #[clap(long, parse(from_occurrences = toggle_bool))]
    pub purge: bool,
    /// Show database entries and their weights
    #[clap(short, long, parse(from_occurrences = toggle_bool))]
    pub stat: bool,
    /// Dry run
    #[clap(long, parse(from_occurrences = toggle_bool))]
    pub dryrun: bool,
}

/// Install/Uninstall necessary files.
#[derive(Clap)]
#[clap(version = VERSION, author = AUTHOR)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct InstallOpts {
    /// Install all necessary files to the user directory
    #[clap(short, long, value_name = "directory", parse(from_os_str = my_from_os_str))]
    pub install: Option<Option<PathBuf>>,
    /// Uninstall all necessary files from the user directory
    #[clap(short, long, parse(from_occurrences = toggle_bool))]
    pub uninstall: bool,
    /// Remove the user database as well
    #[clap(long, parse(from_occurrences = toggle_bool))]
    pub purge: bool,
    /// The prefix of the directory to install
    // TODO: String -> PathBuf
    #[clap(long, value_name = "directory")]
    pub prefix: Option<String>,
    /// Set zsh share destination
    #[clap(long, value_name = "directory", parse(from_os_str))]
    pub zshshare: Option<PathBuf>,
    /// Set clink directory location (Windows only)
    #[clap(long, value_name = "directory", parse(from_os_str))]
    pub clinkdir: Option<PathBuf>,
    /// Dry run
    #[clap(short, long, parse(from_occurrences = toggle_bool))]
    pub dryrun: bool,
    /// Force install by skipping root user, shell type checks
    #[clap(short, long, parse(from_occurrences = toggle_bool))]
    pub force: bool,
    /// Install system wide for all users
    #[clap(short, long, parse(from_occurrences = toggle_bool))]
    pub system: bool,
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: u32,
}
