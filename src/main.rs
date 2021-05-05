use anyhow::{anyhow, bail, Result};
use clap::Clap;
use fastjump::common::opts::Opts;
use fastjump::common::utils::{
    environment_check, find_matches, into_level, normalize_path, print_item, print_stats,
    print_tab_menu, sanitize, Config,
};
use fastjump::database::{load_data, save_data};
use log::info;
use std::collections::HashMap;
use std::path::{Path, MAIN_SEPARATOR};

const TAB_ENTRIES_COUNT: usize = 9;
const TAB_SEPARATOR: &str = "__";

/// Add a new path or increment an existing one.
/// path.canonicalize() is not used because it's preferable to use symlinks
/// with resulting duplicate entries in the database than a single canonical path.
fn add_path(data: &mut HashMap<String, f32>, path: &str, weight: Option<f32>) -> (String, f32) {
    let entry = path.strip_suffix(MAIN_SEPARATOR).unwrap_or(path);
    if entry == shellexpand::tilde("~").as_ref() {
        return (entry.to_string(), 0.0);
    }

    let key = normalize_path(entry);
    let key_ret = key.clone();
    let value =
        (data.get(&key).unwrap_or(&0.0).powf(2.0) + weight.unwrap_or(10.0).powf(2.0)).sqrt();

    data.insert(key, value);

    (key_ret, value)
}

/// Decrease or zero out a path.
fn decrease_path(
    data: &mut HashMap<String, f32>,
    path: &str,
    weight: Option<f32>,
) -> (String, f32) {
    let entry = path.strip_suffix(MAIN_SEPARATOR).unwrap_or(path);
    let key = normalize_path(entry);
    let key_ret = key.clone();
    let value = (data.get(&key).unwrap_or(&0.0) - weight.unwrap_or(15.0)).max(0.0);

    data.insert(key, value);

    (key_ret, value)
}

/// Find matched results
///
/// Given a tab entry in the following format return needle, index, and path:
/// ```
///        [needle]__[index]__[path]
/// ```
fn find_results(needles: &[String], data: &HashMap<String, f32>, complete: bool) -> Result<()> {
    let needles = sanitize(needles.iter()).collect::<Vec<_>>();
    let mut split = needles.get(0).unwrap_or(&"").split(TAB_SEPARATOR);
    let tab_needle = split.next();
    let tab_index = split.next();
    let tab_path = split.next();

    if let Some(path) = tab_path {
        println!("{}", path);
    } else if let Some(_index) = tab_index {
        let index = _index.parse().unwrap_or(0);
        println!(
            "{}",
            find_matches(data, &[tab_needle.unwrap()], false) // never fail
                .get(index)
                .unwrap() // never fail
                .0
        );
    } else if let Some(_needle) = tab_needle {
        // found partial tab completion entry
        if complete {
            print_tab_menu(
                _needle,
                find_matches(data, needles.as_slice(), false)
                    .iter()
                    .take(TAB_ENTRIES_COUNT),
                TAB_SEPARATOR,
            );
        } else {
            let results = find_matches(data, needles.as_slice(), true);
            let path = &results.get(0).unwrap().0;   // never fail
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
fn handle_tab_completion(needles: &[String], data: &HashMap<String, f32>) -> Result<()> {
    find_results(needles, data, true)
}

/// Provide the result path best matched
fn handle_jump(needles: &[String], data: &HashMap<String, f32>) -> Result<()> {
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
        let result = add_path(&mut data, directory.as_str(), None);
        if !opts.dryrun {
            save_data(&config, &data)?;
        }
        print_item(result);
    } else if opts.complete {
        handle_tab_completion(opts.paths.as_slice(), &data)?;
    } else if opts.decrease.is_some() {
        let cwd = std::env::current_dir()?;
        let cwd = cwd
            .as_path()
            .to_str()
            .ok_or(anyhow!("can't get the current directory"))?;
        let result = decrease_path(&mut data, cwd, None);
        if !opts.dryrun {
            save_data(&config, &data)?;
        }
        print_item(result);
    } else if opts.increase.is_some() {
        let cwd = std::env::current_dir()?;
        let cwd = cwd
            .as_path()
            .to_str()
            .ok_or(anyhow!("can't get the current directory"))?;
        let result = add_path(&mut data, cwd, opts.increase);
        if !opts.dryrun {
            save_data(&config, &data)?;
        }
        print_item(result);
    } else if opts.purge {
        let old_entries = data.len();
        data.retain(|key, _| Path::new(key.as_str()).exists());
        if !opts.dryrun {
            save_data(&config, &data)?;
        }
        info!("Purged {} entries.", old_entries - data.len());
    } else if opts.stat {
        print_stats(&data, config.data_path.as_path());
    } else {
        handle_jump(opts.paths.as_slice(), &data)?;
    }
    Ok(())
}
