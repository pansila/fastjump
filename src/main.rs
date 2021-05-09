use anyhow::Result;
use clap::Clap;
use fastjump::common::config::Config;
use fastjump::common::opts::Opts;
use fastjump::common::utils::{setup_logger, CWD};
// use fastjump::common::utils::{environment_check, setup_logger, CWD};
use fastjump::database::Database;
use fastjump::handlers::{
    handle_add_path, handle_decrease_path, handle_jump, handle_print_stats, handle_purge,
    handle_tab_completion,
};

// TODO: compare a string and a path, not to coerce path to string, do it conversely to keep from info loss
// TODO: cleanup - remove unwrap
// TODO: cleanup - remove panicable calls, eg. expect, toss up as a result
// TODO: cleanup - cargo.lock

fn main() -> Result<()> {
    // environment_check()?;

    let opts = Opts::parse();
    setup_logger(&opts);

    let config = Config::new();
    let mut data = Database::new(&config)?;

    if let Some(directory) = opts.add {
        handle_add_path(&config, &mut data, &directory, None, opts.dryrun)?;
    } else if opts.complete {
        handle_tab_completion(&opts.paths.iter().map(|x| x.as_path()).collect::<Vec<_>>(), &data)?;
    } else if opts.decrease.is_some() {
        handle_decrease_path(
            &config,
            &mut data,
            &CWD,
            None,
            opts.dryrun,
        )?;
    } else if opts.increase.is_some() {
        handle_add_path(
            &config,
            &mut data,
            &CWD,
            opts.increase,
            opts.dryrun,
        )?;
    } else if opts.purge {
        handle_purge(&config, &mut data, opts.dryrun)?;
    } else if opts.stat {
        handle_print_stats(&data, config.data_path.as_path());
    } else {
        handle_jump(&opts.paths.iter().map(|x| x.as_path()).collect::<Vec<_>>(), &data)?;
    }
    Ok(())
}
