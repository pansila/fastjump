//#![feature(osstring_ascii)]
// TODO: uppercase check from to_string_lossy

use anyhow::{bail, Context, Result};
use clap::Clap;
use fastjump::common::opts::Opts;
use fastjump::common::utils::{
    normalize_path, environment_check, find_matches, into_level, print_item,
    print_stats, print_tab_menu, Config,
};
use fastjump::database::{load_data, save_data};
use log::info;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const TAB_ENTRIES_COUNT: usize = 9;
const TAB_SEPARATOR: &str = "__";


// TODO: compare a string and a path, not to coerce path to string, do it conversely to keep from info loss
// TODO: cleanup - Cow<str> pass an argument, may not need to call as_ref
// TODO: cleanup - remove unwrap
// TODO: cleanup - remove panicable calls, eg. expect, toss up as a result
// TODO: cleanup - cargo.lock



/// Add a new path or increment an existing one.
/// path.canonicalize() is not used because it's preferable to use symlinks
/// with resulting duplicate entries in the database than a single canonical path.
fn add_path(data: &mut HashMap<PathBuf, f32>, path: &Path, weight: Option<f32>) -> (String, f32) {
    let entry = normalize_path(path);
    // TODO: what is it used for?
    if entry == Path::new(shellexpand::tilde("~").as_ref()) {
        return (entry.to_string_lossy().into_owned(), 0.0);
    }

    let key_ret = entry.to_string_lossy().into_owned();
    let value =
        (data.get(&entry).unwrap_or(&0.0).powf(2.0) + weight.unwrap_or(10.0).powf(2.0)).sqrt();

    data.insert(entry, value);

    (key_ret, value)
}

/// Decrease or zero out a path.
fn decrease_path(
    data: &mut HashMap<PathBuf, f32>,
    path: &Path,
    weight: Option<f32>,
) -> (String, f32) {
    let entry = normalize_path(path);
    let key_ret = entry.to_string_lossy().into_owned();
    let value = (data.get(&entry).unwrap_or(&0.0) - weight.unwrap_or(15.0)).max(0.0);

    data.insert(entry, value);

    (key_ret, value)
}

/// Find matched results
///
/// Given a tab entry in the following format return needle, index, and path:
/// ```
///        [needle]__[index]__[path]
/// ```
fn find_results(needles: &[PathBuf], data: &HashMap<PathBuf, f32>, complete: bool) -> Result<()> {
    // TODO: invalidate instead of normalize?
    let needles: Vec<_> = needles.iter().map(|x| normalize_path(x)).collect();
    let first_needle = needles
        .get(0)
        .context("needles are empty")?
        .to_string_lossy();
    let mut tabs = first_needle.split(TAB_SEPARATOR);
    let tab_needle = tabs.next();
    let tab_index = tabs.next();
    let tab_path = tabs.next();

    if let Some(path) = tab_path {
        println!("{}", path);
    } else if let Some(_index) = tab_index {
        let index = _index.parse().unwrap_or(0);
        println!(
            "{}",
            find_matches(data, &[PathBuf::from(tab_needle.unwrap())], false) // never fail
                .get(index)
                .unwrap() // never fail
                .0
        );
    } else if let Some(_needle) = tab_needle {
        // found partial tab completion entry
        if complete {
            print_tab_menu(
                _needle,
                find_matches(data, &needles, false)
                    .iter()
                    .take(TAB_ENTRIES_COUNT),
                TAB_SEPARATOR,
            );
        } else {
            let results = find_matches(data, &needles, true);
            let path = &results.get(0).unwrap().0; // never fail
            if path == "" {
                println!(".");
            } else {
                println!("{}", path);
            }
        }
    } else {
        bail!("unexpected result from the &str split");
    }
    Ok(())
}

/// Provide tab completion hints
fn handle_tab_completion(needles: &[PathBuf], data: &HashMap<PathBuf, f32>) -> Result<()> {
    find_results(needles, data, true)
}

/// Provide the result path best matched
fn handle_jump(needles: &[PathBuf], data: &HashMap<PathBuf, f32>) -> Result<()> {
    find_results(needles, data, false)
}

fn main() -> Result<()> {
    environment_check()?;

    let opts = Opts::parse();

    let mut builder = env_logger::builder();
    #[cfg(not(debug_assertions))]
    let builder = builder.format_timestamp(None).format_module_path(false);
    builder
        .filter_level(into_level(log::LevelFilter::Info as u32 + opts.verbose))
        .parse_default_env()
        .init();

    let config = Config::new();
    let mut data = load_data(&config)?;

    if let Some(directory) = opts.add {
        let result = add_path(&mut data, &directory, None);
        if !opts.dryrun {
            save_data(&config, &data)?;
        }
        print_item(result);
    } else if opts.complete {
        handle_tab_completion(&opts.paths, &data)?;
    } else if opts.decrease.is_some() {
        let result = decrease_path(&mut data, &std::env::current_dir()?, None);
        if !opts.dryrun {
            save_data(&config, &data)?;
        }
        print_item(result);
    } else if opts.increase.is_some() {
        let result = add_path(&mut data, &std::env::current_dir()?, opts.increase);
        if !opts.dryrun {
            save_data(&config, &data)?;
        }
        print_item(result);
    } else if opts.purge {
        let old_entries = data.len();
        data.retain(|key, _| key.exists());
        if !opts.dryrun {
            save_data(&config, &data)?;
        }
        info!("Purged {} entries.", old_entries - data.len());
    } else if opts.stat {
        print_stats(&data, config.data_path.as_path());
    } else {
        handle_jump(&opts.paths, &data)?;
    }
    Ok(())
}
