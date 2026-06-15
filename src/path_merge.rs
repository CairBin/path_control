use std::collections::HashSet;

use crate::model::{Entry, EntryStatus};

#[derive(Clone, Copy, Debug)]
pub enum PathCase {
    #[cfg_attr(windows, allow(dead_code))]
    Sensitive,
    Insensitive,
}

pub fn merge_path(
    current: &str,
    entries: &[Entry],
    separator: char,
    path_case: PathCase,
) -> String {
    let managed_values = entries
        .iter()
        .map(|entry| normalize_path_entry(&entry.value, path_case))
        .collect::<HashSet<_>>();

    let mut next_parts = current
        .split(separator)
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .filter(|part| !managed_values.contains(&normalize_path_entry(part, path_case)))
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();

    next_parts.extend(
        entries
            .iter()
            .filter(|entry| entry.status == EntryStatus::Enabled)
            .map(|entry| entry.value.clone()),
    );

    next_parts.join(&separator.to_string())
}

pub fn normalize_path_entry(value: &str, path_case: PathCase) -> String {
    let normalized = value.trim().trim_end_matches(['\\', '/']);
    match path_case {
        PathCase::Sensitive => normalized.to_owned(),
        PathCase::Insensitive => normalized.to_ascii_lowercase(),
    }
}

pub fn split_path(current: &str, separator: char) -> impl Iterator<Item = &str> {
    current
        .split(separator)
        .map(str::trim)
        .filter(|part| !part.is_empty())
}

pub fn stable_path_id(value: &str, path_case: PathCase) -> String {
    let normalized = normalize_path_entry(value, path_case);
    let mut hash = 0xcbf29ce484222325u64;

    for byte in normalized.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }

    format!("{hash:016x}")
}
