use clap::{AppSettings, Clap};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");

fn toggle_bool(i: u64) -> bool {
    i > 0
}

/// Jump to any directory fast and smart
#[derive(Clap)]
#[clap(version = VERSION, author = AUTHOR)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
    /// The directory to jump to, composed of parts of a path
    // pub paths: Option<Vec<String>>,
    // workaround for zero positional argument
    #[clap(requires_if("bar", "increase"))]
    pub paths: Vec<String>,
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: u32,
    /// Add a path with a default weight
    #[clap(short, long, value_name = "path")]
    pub add: Option<String>,
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
    /// Install all necessary files to the user directory
    #[clap(long)]
    pub install: Option<Option<String>>,
    /// Uninstall all necessary files from the user directory
    #[clap(long, parse(from_occurrences = toggle_bool))]
    pub uninstall: bool,
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
    #[clap(short, long, value_name = "directory")]
    pub install: Option<Option<String>>,
    /// Uninstall all necessary files from the user directory
    #[clap(short, long, parse(from_occurrences = toggle_bool))]
    pub uninstall: bool,
    /// Remove the user database as well
    #[clap(long, parse(from_occurrences = toggle_bool))]
    pub purge: bool,
    /// The prefix of the directory to install
    #[clap(long, value_name = "directory")]
    pub prefix: Option<Option<String>>,
    /// Set zsh share destination
    #[clap(long, value_name = "directory")]
    pub zshshare: Option<Option<String>>,
    /// Set clink directory location (Windows only)
    #[clap(long, value_name = "directory")]
    pub clinkdir: Option<Option<String>>,
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
