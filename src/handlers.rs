use crate::common::config::Config;
use crate::common::utils::{
    absolute_path, find_matches, normalize_path, print_item, print_tab_menu, CWD,
};
use crate::database::Database;
use anyhow::{bail, Result};
use log::info;
use std::cmp::Ordering;
use std::path::Path;

const TAB_ENTRIES_COUNT: usize = 9;
const TAB_SEPARATOR: &str = "__";

/// Add a new path or increment an existing one.
/// path.canonicalize() is not used because it's preferable to use symlinks
/// with resulting duplicate entries in the database than a single canonical path.
pub fn handle_add_path(
    config: &Config,
    data: &mut Database,
    path: &Path,
    weight: Option<f32>,
    dryrun: bool,
) -> Result<()> {
    let entry = absolute_path(normalize_path(path));
    // TODO: what is it used for?
    if entry == Path::new(shellexpand::tilde("~").as_ref()) {
        print_item((entry.to_string_lossy(), 0.0));
        return Ok(());
    }

    let value =
        (data.get(&entry).unwrap_or(&0.0).powf(2.0) + weight.unwrap_or(10.0).powf(2.0)).sqrt();

    print_item((entry.to_string_lossy(), value));

    data.insert(entry, value);
    if !dryrun {
        data.save(&config)?;
    }

    Ok(())
}

/// Decrease or zero out a path.
pub fn handle_decrease_path(
    config: &Config,
    data: &mut Database,
    path: &Path,
    weight: Option<f32>,
    dryrun: bool,
) -> Result<()> {
    let entry = normalize_path(path);
    let value = (data.get(&entry).unwrap_or(&0.0) - weight.unwrap_or(15.0)).max(0.0);

    print_item((entry.to_string_lossy(), value));

    data.insert(entry, value);
    if !dryrun {
        data.save(&config)?;
    }

    Ok(())
}

/// print the statistics from the database
pub fn handle_print_stats(data: &Database, data_path: &Path) {
    info!("Weight\t\tPath");
    info!("{}", "-".repeat(80));
    let mut count_vec: Vec<_> = data.iter().collect();
    count_vec.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(Ordering::Equal));
    for (path, weight) in count_vec {
        print_item((path.display(), *weight));
    }

    let sum: f32 = data.values().sum();
    info!("{}", "_".repeat(80));
    info!("{:.2}\t\ttotal weight", sum);
    info!(
        "{:width$}\t\ttotal entries",
        data.len(),
        width = (sum.log10().floor() as usize) + 4
    );

    info!(
        "{:.2}\t\tcurrent directory weight",
        data.get(&normalize_path(&CWD)).unwrap_or(&0.0)
    );

    info!("");
    info!("database file:\t{}", data_path.to_str().unwrap()); // never fail
}

/// Find matched results
///
/// Given a tab entry in the following format return needle, index, and path:
/// ```text
///        [needle]__[index]__[path]
/// ```
fn find_results(needles: &[&Path], data: &Database, complete: bool) -> Result<()> {
    // TODO: invalidate instead of normalize?
    let needles: Vec<_> = needles.iter().map(|x| normalize_path(x)).collect();
    let needles: Vec<_> = needles.iter().map(|x| x.as_path()).collect();
    let first_needle = needles.get(0).unwrap_or(&Path::new("")).to_string_lossy();
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
            find_matches(data, &[Path::new(tab_needle.unwrap())], false) // never fail
                .get(index)
                .unwrap() // never fail
                .0
                .to_string_lossy()
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
            assert!(!path.as_os_str().is_empty());
            println!("{}", path.to_string_lossy());
        }
    } else {
        bail!("unexpected result from the &str split");
    }
    Ok(())
}

/// Provide tab completion hints
pub fn handle_tab_completion(needles: &[&Path], data: &Database) -> Result<()> {
    find_results(needles, data, true)
}

/// Provide the result path best matched
pub fn handle_jump(needles: &[&Path], data: &Database) -> Result<()> {
    find_results(needles, data, false)
}

pub fn handle_purge(config: &Config, data: &mut Database, dryrun: bool) -> Result<()> {
    let old_entries = data.len();
    data.retain(|key, _| key.exists());
    if !dryrun {
        data.save(&config)?;
    }
    info!("Purged {} entries.", old_entries - data.len());
    Ok(())
}
