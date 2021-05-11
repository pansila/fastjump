use anyhow::Result;
use fastjump::common::config::Config;
use fastjump::common::opts::Opts;
use structopt::StructOpt;
use fastjump::common::utils::{environment_check, setup_logger, CWD};
use fastjump::database::Database;
use fastjump::handlers::{
    handle_add_path, handle_decrease_path, handle_jump, handle_print_stats, handle_purge,
    handle_tab_completion,
};

// TODO: cleanup - remove unwrap
// TODO: cleanup - remove panicable calls, eg. expect, toss up as a result
// TODO: cleanup - unnecessary features
// TODO: add env versions collection
// TODO: j <empty> go to the most recently dir
// TODO: expand to abs path for add

fn main() -> Result<()> {
    environment_check()?;

    let opts = Opts::from_args();
    setup_logger(&opts);

    let config = Config::new();
    let mut data = Database::new(&config)?;

    if let Some(directory) = opts.add {
        handle_add_path(&config, &mut data, &directory, None, opts.dryrun)?;
    } else if opts.complete {
        handle_tab_completion(
            &opts.paths.iter().map(|x| x.as_path()).collect::<Vec<_>>(),
            &data,
        )?;
    } else if opts.decrease.is_some() {
        handle_decrease_path(&config, &mut data, &CWD, None, opts.dryrun)?;
    } else if opts.increase.is_some() {
        handle_add_path(&config, &mut data, &CWD, opts.increase, opts.dryrun)?;
    } else if opts.purge {
        handle_purge(&config, &mut data, opts.dryrun)?;
    } else if opts.stat {
        handle_print_stats(&data, config.data_path.as_path());
    } else {
        // TODO: move to the top
        handle_jump(
            &opts.paths.iter().map(|x| x.as_path()).collect::<Vec<_>>(),
            &data,
        )?;
    }
    Ok(())
}
