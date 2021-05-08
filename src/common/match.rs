use lazy_static::lazy_static;
use log::{debug, trace};
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use strsim::normalized_levenshtein;

const ENTRIES_COUNT: usize = 9; // TODO
const PKGNAME: &str = env!("CARGO_PKG_NAME");

lazy_static! {
    static ref FUZZY_MATCH_THRESHOLD: f64 =
        shellexpand::env(format!("${}_FUZZY_THRESHOLD", PKGNAME.to_ascii_lowercase()).as_str())
            .unwrap_or(Cow::from("0.6"))
            .as_ref()
            .parse()
            .unwrap_or(0.6);
}

/// Matches needles anywhere in the path as long as they're in the same (but
/// not necessarily consecutive) order.
///
/// Please see examples in the tests
pub fn match_anywhere<'a>(
    needles: &[PathBuf],
    data: &'a HashMap<PathBuf, f32>,
    ignore_case: bool,
) -> Vec<(Cow<'a, Path>, f32)> {
    let mut candidates: Vec<(Cow<Path>, f32)> = Vec::with_capacity(ENTRIES_COUNT);

    for (k, v) in data.iter() {
        let path = k.to_string_lossy();
        if needles.iter().try_fold((), |_, needle| {
            // TODO: do overlapped cases matter?
            // TODO: does needles order matter?
            if ignore_case {
                if !path
                    .to_ascii_lowercase()
                    .contains(&needle.to_string_lossy().to_ascii_lowercase())
                {
                    return None;
                }
            } else {
                if !path.contains(needle.to_string_lossy().as_ref()) {
                    return None;
                }
            }
            Some(())
        }) != None
        {
            trace!("pushing ({}, {})", path, v);
            candidates.push((Cow::Borrowed(k), *v));
        }
    }
    candidates
}

/// Matches consecutive needles at the end of a path.
///
/// Each needle must be part of one of the components of the path.
///
/// Please see examples in the tests
pub fn match_consecutive<'a>(
    needles: &[PathBuf],
    data: &'a HashMap<PathBuf, f32>,
    ignore_case: bool,
) -> Vec<(Cow<'a, Path>, f32)> {
    let mut candidates: Vec<(Cow<Path>, f32)> = Vec::with_capacity(ENTRIES_COUNT);

    for (k, v) in data.iter() {
        let mut comp_iter = k.components().rev();
        if needles.iter().rev().try_fold((), |_, needle| {
            // TODO: curdir and rootdir?
            if let Some(Component::Normal(comp)) = comp_iter.next() {
                if ignore_case {
                    if !comp
                        .to_string_lossy()
                        .to_ascii_lowercase()
                        .contains(&needle.to_string_lossy().to_ascii_lowercase())
                    {
                        return None;
                    }
                } else {
                    if !comp
                        .to_string_lossy()
                        .contains(needle.to_string_lossy().as_ref())
                    {
                        return None;
                    }
                }
            } else {
                return None;
            }
            Some(())
        }) != None
        {
            let path = k.to_string_lossy();
            trace!("pushing ({}, {})", path, v);
            candidates.push((Cow::Borrowed(k), *v));
        }
    }
    candidates
}

/// Performs an approximate match with the last needle against the end of
/// every path past an acceptable threshold.
///
/// This is a weak heuristic and used as a last resort to find matches.
///
/// Please see examples in the tests
pub fn match_fuzzy<'a>(
    needles: &[PathBuf],
    data: &'a HashMap<PathBuf, f32>,
    ignore_case: bool,
    threshold: Option<f64>,
) -> Vec<(Cow<'a, Path>, f32)> {
    let end_dir = |path: &'a Path| -> Cow<'a, str> {
        path.file_name()
            .expect("expect a non-empty path")
            .to_string_lossy()
    };
    let last = needles
        .last()
        .expect("Expect a non-empty path to search")
        .to_string_lossy();
    let match_percent: Box<dyn Fn(&'a Path) -> f64> = if ignore_case {
        Box::new(|path: &'a Path| {
            let needle = last.to_ascii_lowercase();
            normalized_levenshtein(needle.as_str(), end_dir(path).to_ascii_lowercase().as_str())
        })
    } else {
        Box::new(|path: &'a Path| normalized_levenshtein(last.as_ref(), end_dir(path).as_ref()))
    };
    let meets_threshold = |path: &'a Path| {
        let score = match_percent(path);
        debug!("fuzzy score {}: {}", path.to_string_lossy(), score);
        score >= threshold.unwrap_or(*FUZZY_MATCH_THRESHOLD)
    };
    let mut candidates: Vec<(Cow<Path>, f32)> = Vec::with_capacity(ENTRIES_COUNT);

    for (k, v) in data.iter() {
        if meets_threshold(k) {
            let path = k.to_string_lossy();
            trace!("pushing ({}, {})", path, v);
            candidates.push((Cow::Borrowed(k), *v));
        }
    }
    candidates
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::iter::FromIterator;

    #[ctor::ctor]
    fn init() {
        env_logger::init();
    }

    #[test]
    fn test_match_anywhere() {
        let needles = ["foo", "bar"];
        let test_set = [
            (vec!["", "foo", "bar", "baz"], true),
            (vec!["", "ffoof", "bbarb"], true),
            (vec!["", "ffoof", "baz", "bbarb"], true),
            (vec!["", "baz", "foo", "bar"], true),
            (vec!["", "foo", "baz"], false),
            (vec!["", "foo", "baz", "bar"], true),
            (vec!["", "foobarbaz"], true),
        ];
        let data = HashMap::from_iter(test_set.iter().map(|x| (x.0.iter().collect(), 10.0f32)));
        let results = match_anywhere(
            &needles.iter().map(|x| PathBuf::from(x)).collect::<Vec<_>>(),
            &data,
            true,
        );
        dbg!(&results);

        for v in test_set.iter() {
            dbg!(v);
            assert_eq!(
                results
                    .iter()
                    .any(|x| x.0 == v.0.iter().collect::<PathBuf>().to_string_lossy()
                        && x.1 == 10.0f32),
                v.1
            );
        }
    }

    #[test]
    fn test_match_consecutive() {
        let needles = ["foo", "bar"];
        let test_set = [
            (vec!["", "foo", "bar", "baz"], false),
            (vec!["", "ffoof", "bbarb"], true),
            (vec!["", "ffoof", "baz", "bbarb"], false),
            (vec!["", "baz", "foo", "bar"], true),
            (vec!["", "foo", "baz"], false),
            (vec!["", "foo", "baz", "bar"], false),
            (vec!["", "foobarbaz"], false),
        ];
        let data = HashMap::from_iter(test_set.iter().map(|x| (x.0.iter().collect(), 10.0f32)));
        let results = match_consecutive(
            &needles.iter().map(|x| PathBuf::from(x)).collect::<Vec<_>>(),
            &data,
            true,
        );
        dbg!(&results);

        for v in test_set.iter() {
            dbg!(v);
            assert_eq!(
                results
                    .iter()
                    .any(|x| x.0 == v.0.iter().collect::<PathBuf>().to_string_lossy()
                        && x.1 == 10.0f32),
                v.1
            );
        }
    }

    #[test]
    fn test_match_fuzzy() {
        let needles = ["foo", "hme"];
        let test_set = [
            (vec!["", "home"], true),
            (vec!["", "bar", "baz", "home"], true),
            (vec!["", "home", "bar", "baz"], false),
            (vec!["", "ffoof", "bbarb"], false),
        ];
        let data = HashMap::from_iter(test_set.iter().map(|x| (x.0.iter().collect(), 10.0f32)));
        let results = match_fuzzy(
            &needles.iter().map(|x| PathBuf::from(x)).collect::<Vec<_>>(),
            &data,
            true,
            None,
        );
        dbg!(&results);

        for v in test_set.iter() {
            dbg!(v);
            assert_eq!(
                results
                    .iter()
                    .any(|x| x.0 == v.0.iter().collect::<PathBuf>().to_string_lossy()
                        && x.1 == 10.0f32),
                v.1
            );
        }
    }
}
