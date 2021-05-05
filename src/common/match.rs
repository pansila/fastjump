use lazy_static::lazy_static;
use log::{debug, trace};
use regex::{escape, Regex};
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::MAIN_SEPARATOR;
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
pub fn match_anywhere(
    needles: &[&str],
    data: &HashMap<String, f32>,
    ignore_case: bool,
) -> Vec<(String, f32)> {
    let re = Regex::new(
        format!(
            "{}.*{}.*",
            if ignore_case { "(?i)" } else { "" },
            needles.join(".*")
        )
        .as_str(),
    )
    .expect("Invalid regex expression");
    let mut candidates: Vec<(String, f32)> = Vec::with_capacity(ENTRIES_COUNT);

    for (k, v) in data.iter() {
        if re.is_match(k.as_str()) {
            candidates.push((k.clone(), *v));
            trace!("pushing ({}, {})", k, v);
        }
    }
    candidates
}

/// Matches consecutive needles at the end of a path.
///
/// Please see examples in the tests
pub fn match_consecutive(
    needles: &[&str],
    data: &HashMap<String, f32>,
    ignore_case: bool,
) -> Vec<(String, f32)> {
    let escape_sep = escape(MAIN_SEPARATOR.to_string().as_str());
    let regex_no_sep = format!("[^{}]*", escape_sep);
    let regex_no_sep_end = format!("{}$", regex_no_sep);
    let regex_one_sep = format!("{}{}{}", regex_no_sep, escape_sep, regex_no_sep);
    let regex_needle = needles
        .iter()
        .map(|s| escape(s))
        .collect::<Vec<_>>()
        .join(regex_one_sep.as_str())
        + regex_no_sep_end.as_str();
    let re =
        Regex::new(format!("{}{}", if ignore_case { "(?i)" } else { "" }, regex_needle).as_str())
            .expect("Invalid regex expression");
    let mut candidates: Vec<(String, f32)> = Vec::with_capacity(ENTRIES_COUNT);

    for (k, v) in data.iter() {
        if re.is_match(k.as_str()) {
            candidates.push((k.clone(), *v));
            trace!("pushing ({}, {})", k, v);
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
    needles: &'a [&str],
    data: &'a HashMap<String, f32>,
    ignore_case: bool,
    threshold: Option<f64>,
) -> Vec<(String, f32)> {
    let end_dir = |path: &'a str| -> &'a str {
        path.split(MAIN_SEPARATOR)
            .last()
            .expect("expect a non-empty path")
    };
    let last = needles.last().expect("Expect a non-empty path to search");
    let match_percent: Box<dyn Fn(&'a str) -> f64> = if ignore_case {
        Box::new(|path: &'a str| {
            let needle = last.to_ascii_lowercase();
            normalized_levenshtein(needle.as_str(), end_dir(path).to_ascii_lowercase().as_str())
        })
    } else {
        Box::new(|path: &str| normalized_levenshtein(last, end_dir(path)))
    };
    let meets_threshold = |path: &'a str| {
        let score = match_percent(path);
        debug!("fuzzy score {}: {}", path, score);
        score >= threshold.unwrap_or(*FUZZY_MATCH_THRESHOLD)
    };
    let mut candidates: Vec<(String, f32)> = Vec::with_capacity(ENTRIES_COUNT);

    for (k, v) in data.iter() {
        if meets_threshold(k) {
            candidates.push((k.clone(), *v));
            trace!("pushing ({}, {})", k, v);
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
        let data_items: Vec<_> = test_set
            .iter()
            .map(|x| (x.0.join(MAIN_SEPARATOR.to_string().as_str()), 10.0f32))
            .collect();
        let data = HashMap::from_iter(data_items.iter().cloned());
        let results = match_anywhere(&needles, &data, true);
        dbg!(&results);

        for v in test_set.iter() {
            assert_eq!(results.iter().any(|x| x.0 == v.0.join(MAIN_SEPARATOR.to_string().as_str()) && x.1 == 10.0f32), v.1);
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
        let data_items: Vec<_> = test_set
            .iter()
            .map(|x| (x.0.join(MAIN_SEPARATOR.to_string().as_str()), 10.0f32))
            .collect();
        let data = HashMap::from_iter(data_items.iter().cloned());
        let results = match_consecutive(&needles, &data, true);
        dbg!(&results);

        for v in test_set.iter() {
            assert_eq!(results.iter().any(|x| x.0 == v.0.join(MAIN_SEPARATOR.to_string().as_str()) && x.1 == 10.0f32), v.1);
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
        let data_items: Vec<_> = test_set
            .iter()
            .map(|x| (x.0.join(MAIN_SEPARATOR.to_string().as_str()), 10.0f32))
            .collect();
        let data = HashMap::from_iter(data_items.iter().cloned());
        let results = match_fuzzy(&needles, &data, true, None);
        dbg!(&results);

        for v in test_set.iter() {
            assert_eq!(results.iter().any(|x| x.0 == v.0.join(MAIN_SEPARATOR.to_string().as_str()) && x.1 == 10.0f32), v.1);
        }
    }
}
