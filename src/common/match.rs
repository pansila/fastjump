use crate::database::Database;
use lazy_static::lazy_static;
use log::{debug, trace};
use std::borrow::Cow;
use std::path::{Path, MAIN_SEPARATOR};
use strsim::normalized_levenshtein;

const ENTRIES_COUNT: usize = 9; // TODO
const PKGNAME: &str = env!("CARGO_PKG_NAME");

// TODO: type Items<'a> = Vec<(&'a Path, f32)>;

lazy_static! {
    static ref FUZZY_MATCH_THRESHOLD: f64 =
        shellexpand::env(format!("${}_FUZZY_THRESHOLD", PKGNAME.to_ascii_lowercase()).as_str())
            .unwrap_or(Cow::from("0.6"))
            .parse()
            .unwrap_or(0.6);
}

pub trait MakeAsciiLowercaseCow {
    fn make_ascii_lowercase_cow(self: &mut Self);
}

pub trait MakeAsciiUppercaseCow {
    fn make_ascii_uppercase_cow(self: &mut Self);
}

impl MakeAsciiLowercaseCow for Cow<'_, str> {
    fn make_ascii_lowercase_cow(self: &mut Self) {
        if self.chars().any(|x| x.is_ascii_uppercase()) {
            self.to_mut().make_ascii_lowercase();
        }
    }
}

impl MakeAsciiUppercaseCow for Cow<'_, str> {
    fn make_ascii_uppercase_cow(self: &mut Self) {
        if self.chars().any(|x| x.is_ascii_lowercase()) {
            self.to_mut().make_ascii_uppercase();
        }
    }
}

/// Matches needles anywhere in the path as long as they're in the same (but
/// not necessarily consecutive) order.
///
/// Please see examples in the tests
pub fn match_anywhere<'a>(
    needles: &[&Path],
    data: &'a Database,
    ignore_case: bool,
) -> Vec<(&'a Path, f32)> {
    let mut candidates: Vec<(&'a Path, f32)> = Vec::with_capacity(ENTRIES_COUNT);

    for (k, v) in data.iter() {
        let mut path = k.to_string_lossy();
        path.make_ascii_lowercase_cow();
        if needles.iter().try_fold((), &mut |_, needle: &&Path| {
            // TODO: do overlapped cases matter?
            // TODO: does needles order matter?
            if ignore_case {
                let mut needle = needle.to_string_lossy();
                needle.make_ascii_lowercase_cow();
                if !path.contains(needle.as_ref()) {
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
            trace!("pushing ({}, {})", k.to_string_lossy(), v);
            candidates.push((k, *v));
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
    needles: &[&Path],
    data: &'a Database,
    ignore_case: bool,
) -> Vec<(&'a Path, f32)> {
    let mut candidates: Vec<(&'a Path, f32)> = Vec::with_capacity(ENTRIES_COUNT);

    for (k, v) in data.iter() {
        // we don't use components as the path has been normalized
        let path = k.to_string_lossy();
        let mut part_iter = path.split(MAIN_SEPARATOR).rev();
        if needles.iter().rev().try_fold((), |_, needle| {
            if let Some(part) = part_iter.next() {
                if ignore_case {
                    let mut part = Cow::from(part);
                    part.make_ascii_lowercase_cow();
                    let mut needle = needle.to_string_lossy();
                    needle.make_ascii_lowercase_cow();

                    if !part.contains(needle.as_ref()) {
                        return None;
                    }
                } else {
                    if !part.contains(needle.to_string_lossy().as_ref()) {
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
            candidates.push((k, *v));
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
    needles: &[&Path],
    data: &'a Database,
    ignore_case: bool,
    threshold: Option<f64>,
) -> Vec<(&'a Path, f32)> {
    let end_dir = |path: &'a Path| -> Cow<'a, str> {
        path.file_name()
            .expect("expect a non-empty path")
            .to_string_lossy()
    };
    let mut needle = needles
        .last()
        .expect("Expect a non-empty path to search")
        .to_string_lossy();
    needle.make_ascii_lowercase_cow();
    let mut match_percent: Box<dyn FnMut(&'a Path) -> f64> = if ignore_case {
        Box::new(|path: &Path| {
            let mut end = end_dir(path);
            end.make_ascii_lowercase_cow();
            normalized_levenshtein(needle.as_ref(), end.as_ref())
        })
    } else {
        Box::new(|path: &Path| normalized_levenshtein(needle.as_ref(), end_dir(path).as_ref()))
    };
    let mut meets_threshold = |path: &'a Path| {
        let score = match_percent(path);
        debug!("fuzzy score {}: {}", path.to_string_lossy(), score);
        score >= threshold.unwrap_or(*FUZZY_MATCH_THRESHOLD)
    };
    let mut candidates: Vec<(&'a Path, f32)> = Vec::with_capacity(ENTRIES_COUNT);

    for (k, v) in data.iter() {
        if meets_threshold(k) {
            trace!("pushing ({}, {})", k.to_string_lossy(), v);
            candidates.push((k, *v));
        }
    }
    candidates
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::iter::FromIterator;
    use std::path::PathBuf;

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
        let data = Database::from(HashMap::from_iter(
            test_set.iter().map(|x| (x.0.iter().collect(), 10.0f32)),
        ));
        let results = match_anywhere(
            &needles.iter().map(|x| Path::new(x)).collect::<Vec<_>>(),
            &data,
            true,
        );
        dbg!(&results);

        for v in test_set.iter() {
            dbg!(v);
            assert_eq!(
                results
                    .iter()
                    .any(|x| x.0 == &v.0.iter().collect::<PathBuf>() && x.1 == 10.0f32),
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
        let data = Database::from(HashMap::from_iter(
            test_set.iter().map(|x| (x.0.iter().collect(), 10.0f32)),
        ));
        let results = match_consecutive(
            &needles.iter().map(|x| Path::new(x)).collect::<Vec<_>>(),
            &data,
            true,
        );
        dbg!(&results);

        for v in test_set.iter() {
            dbg!(v);
            assert_eq!(
                results
                    .iter()
                    .any(|x| x.0 == &v.0.iter().collect::<PathBuf>() && x.1 == 10.0f32),
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
        let data = Database::from(HashMap::from_iter(
            test_set.iter().map(|x| (x.0.iter().collect(), 10.0f32)),
        ));
        let results = match_fuzzy(
            &needles.iter().map(|x| Path::new(x)).collect::<Vec<_>>(),
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
                    .any(|x| x.0 == &v.0.iter().collect::<PathBuf>() && x.1 == 10.0f32),
                v.1
            );
        }
    }
}
