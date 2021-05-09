use structopt::StructOpt;
use std::path::PathBuf;
// use std::ffi::OsStr;

// fn my_from_os_str(path: &OsStr) -> Option<Option<PathBuf>> {
//     dbg!(path);
//     if path.is_empty() {
//         return Some(None);
//     }
//     Some(Some(PathBuf::from(path)))
// }

fn toggle_bool(i: u64) -> bool {
    i > 0
}

/// Jump to any directory fast and smart
#[derive(StructOpt)]
pub struct Opts {
    /// The directory to jump to, composed of parts of a path
    // pub paths: Option<Vec<String>>,
    // workaround for zero positional argument
    #[structopt(requires_if("bar", "increase"), parse(from_os_str))]
    pub paths: Vec<PathBuf>,
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u32,
    /// Add a path with a default weight
    #[structopt(short, long, value_name = "path", parse(from_os_str))]
    pub add: Option<PathBuf>,
    /// Increase the current directory weight
    #[structopt(short, long, value_name = "weight")]
    pub increase: Option<f32>,
    /// Decrease the current directory weight
    #[structopt(short, long, value_name = "weight")]
    pub decrease: Option<f32>,
    /// Used for tab completion
    #[structopt(long, parse(from_occurrences = toggle_bool))]
    pub complete: bool,
    /// Remove non-existent paths from database
    #[structopt(long, parse(from_occurrences = toggle_bool))]
    pub purge: bool,
    /// Show database entries and their weights
    #[structopt(short, long, parse(from_occurrences = toggle_bool))]
    pub stat: bool,
    /// Dry run
    #[structopt(long, parse(from_occurrences = toggle_bool))]
    pub dryrun: bool,
}

/// Install/Uninstall necessary files.
#[derive(StructOpt)]
pub struct InstallOpts {
    /// Install all necessary files to the user directory
    // TODO: give up for Option<Option<PathBuf>>, not working
    #[structopt(short, long, value_name = "directory")]
    pub install: Option<Option<String>>,
    /// Uninstall all necessary files from the user directory
    #[structopt(short, long, parse(from_occurrences = toggle_bool))]
    pub uninstall: bool,
    /// Remove the user database as well
    #[structopt(long, parse(from_occurrences = toggle_bool))]
    pub purge: bool,
    /// The prefix of the directory to install
    // TODO: String -> PathBuf
    #[structopt(long, value_name = "directory")]
    pub prefix: Option<String>,
    /// Set zsh share destination
    #[structopt(long, value_name = "directory", parse(from_os_str))]
    pub zshshare: Option<PathBuf>,
    /// Set clink directory location (Windows only)
    #[structopt(long, value_name = "directory", parse(from_os_str))]
    pub clinkdir: Option<PathBuf>,
    /// Dry run
    #[structopt(short, long, parse(from_occurrences = toggle_bool))]
    pub dryrun: bool,
    /// Force install by skipping root user, shell type checks
    #[structopt(short, long, parse(from_occurrences = toggle_bool))]
    pub force: bool,
    /// Install system wide for all users
    #[structopt(short, long, parse(from_occurrences = toggle_bool))]
    pub system: bool,
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u32,
}
