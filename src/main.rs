mod cli;
mod error;
mod model;
mod path_merge;
mod store;

use clap::Parser;

use crate::cli::{Cli, Command};
use crate::error::AppResult;
use crate::model::{Entry, EntryStatus};
use crate::path_merge::{PathCase, normalize_path_entry, split_path, stable_path_id};
use crate::store::{EnvStore, create_store};

#[cfg(windows)]
const PATH_SEPARATOR: char = ';';
#[cfg(windows)]
const PATH_CASE: PathCase = PathCase::Insensitive;
#[cfg(not(windows))]
const PATH_SEPARATOR: char = ':';
#[cfg(not(windows))]
const PATH_CASE: PathCase = PathCase::Sensitive;

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> AppResult<()> {
    let cli = Cli::parse();
    let scope = cli::ScopeArg::resolve(cli.scope, cli.system)?;
    let store = create_store(scope);

    match cli.command {
        Command::Add {
            name,
            value,
            tips,
            disabled,
        } => {
            let mut entries = store.load_entries()?;
            sync_entries(store.as_ref(), &mut entries)?;
            if entries.iter().any(|entry| entry.name == name) {
                return Err(format!("entry '{name}' already exists").into());
            }
            ensure_unique_id(&mut entries);

            entries.push(Entry::new(
                name,
                value,
                tips,
                EntryStatus::from_disabled(disabled),
            ));
            store.save_entries(&entries)?;
            store.apply_path(&entries)?;
            println!("added");
        }
        Command::List { all } => {
            let mut entries = store.load_entries()?;
            sync_entries(store.as_ref(), &mut entries)?;
            let current_path = store.load_path()?;
            print_entries(&entries, &current_path, all);
        }
        Command::Show { name } => {
            let mut entries = store.load_entries()?;
            sync_entries(store.as_ref(), &mut entries)?;
            let current_path = store.load_path()?;
            show_entry(&entries, &current_path, &name)?;
        }
        Command::Remove { name } => {
            let mut entries = store.load_entries()?;
            sync_entries(store.as_ref(), &mut entries)?;
            remove_entry(store.as_ref(), &mut entries, &name)?;
            println!("removed");
        }
        Command::Enable { name } => {
            update_status(store.as_ref(), &name, EntryStatus::Enabled)?;
            println!("enabled");
        }
        Command::Disable { name } => {
            update_status(store.as_ref(), &name, EntryStatus::Disabled)?;
            println!("disabled");
        }
        Command::Apply => {
            let mut entries = store.load_entries()?;
            sync_entries(store.as_ref(), &mut entries)?;
            store.apply_path(&entries)?;
            println!("applied");
        }
        Command::Export => {
            let mut entries = store.load_entries()?;
            sync_entries(store.as_ref(), &mut entries)?;
            println!("{}", serde_json::to_string_pretty(&entries)?);
        }
    }

    Ok(())
}

fn update_status(store: &dyn EnvStore, name: &str, status: EntryStatus) -> AppResult<()> {
    let mut entries = store.load_entries()?;
    sync_entries(store, &mut entries)?;
    if let Ok(index) = find_entry_index(&entries, name) {
        entries[index].status = status;
        store.save_entries(&entries)?;
        return store.apply_path(&entries);
    }

    let current_path = store.load_path()?;
    let Some(external) = find_external_entry(&entries, &current_path, name)? else {
        return Err(format!("entry '{name}' not found").into());
    };

    match status {
        EntryStatus::Enabled => Ok(()),
        EntryStatus::Disabled => {
            let entry = Entry::new(
                external.name,
                external.value.clone(),
                Some("adopted from external PATH entry".to_owned()),
                EntryStatus::Disabled,
            );
            entries.push(entry);
            store.save_entries(&entries)?;
            remove_path_value(store, &current_path, &external.value)
        }
    }
}

fn find_entry_index(entries: &[Entry], query: &str) -> AppResult<usize> {
    let matches = entries
        .iter()
        .enumerate()
        .filter(|(_, entry)| entry.name == query || entry.id.starts_with(query))
        .map(|(index, _)| index)
        .collect::<Vec<_>>();

    match matches.as_slice() {
        [index] => Ok(*index),
        [] => Err(format!("entry '{query}' not found").into()),
        _ => Err(format!("entry id prefix '{query}' is ambiguous").into()),
    }
}

fn sync_entries(store: &dyn EnvStore, entries: &mut [Entry]) -> AppResult<()> {
    let current_path = store.load_path()?;
    let current_values = split_path(&current_path, PATH_SEPARATOR)
        .map(|part| normalize_path_entry(part, PATH_CASE))
        .collect::<std::collections::HashSet<_>>();

    let mut changed = ensure_unique_id(entries);

    for entry in entries.iter_mut() {
        let exists = current_values.contains(&normalize_path_entry(&entry.value, PATH_CASE));
        let next_status = if exists {
            EntryStatus::Enabled
        } else {
            EntryStatus::Disabled
        };

        if entry.status != next_status {
            entry.status = next_status;
            changed = true;
        }
    }

    if changed {
        store.save_entries(entries)?;
    }
    Ok(())
}

fn remove_entry(store: &dyn EnvStore, entries: &mut Vec<Entry>, query: &str) -> AppResult<()> {
    if let Ok(index) = find_entry_index(entries, query) {
        entries.remove(index);
        store.save_entries(entries)?;
        return store.apply_path(entries);
    }

    let current_path = store.load_path()?;
    let Some(external) = find_external_entry(entries, &current_path, query)? else {
        return Err(format!("entry '{query}' not found").into());
    };

    remove_path_value(store, &current_path, &external.value)
}

fn ensure_unique_id(entries: &mut [Entry]) -> bool {
    let mut changed = false;
    let mut seen = std::collections::HashSet::new();

    for entry in entries {
        if entry.id.is_empty() || !seen.insert(entry.id.clone()) {
            entry.id = Entry::new(
                entry.name.clone(),
                entry.value.clone(),
                entry.tips.clone(),
                entry.status,
            )
            .id;
            changed = true;
            seen.insert(entry.id.clone());
        }
    }

    changed
}

fn print_entries(entries: &[Entry], current_path: &str, all: bool) {
    let mut rows = entries
        .iter()
        .filter(|entry| all || entry.status == EntryStatus::Enabled)
        .map(ListRow::managed)
        .collect::<Vec<_>>();

    let managed_values = entries
        .iter()
        .map(|entry| normalize_path_entry(&entry.value, PATH_CASE))
        .collect::<std::collections::HashSet<_>>();

    rows.extend(
        split_path(current_path, PATH_SEPARATOR)
            .filter(|part| !managed_values.contains(&normalize_path_entry(part, PATH_CASE)))
            .enumerate()
            .map(|(index, value)| ListRow::external(index + 1, value)),
    );

    if rows.is_empty() {
        println!("no entries");
        return;
    }

    let name_width = rows
        .iter()
        .map(|row| row.name.len())
        .max()
        .unwrap_or("NAME".len())
        .max("NAME".len());
    let status_width = rows
        .iter()
        .map(|row| row.status.len())
        .max()
        .unwrap_or("STATUS".len())
        .max("STATUS".len());
    let source_width = "SOURCE".len();

    println!(
        "{:<12}  {:<name_width$}  {:<status_width$}  {:<source_width$}  VALUE  TIPS",
        "ID", "NAME", "STATUS", "SOURCE",
    );
    for row in rows {
        println!(
            "{:<12}  {:<name_width$}  {:<status_width$}  {:<source_width$}  {}  {}",
            row.id, row.name, row.status, row.source, row.value, row.tips,
        );
    }
}

struct ListRow {
    id: String,
    name: String,
    status: String,
    source: &'static str,
    value: String,
    tips: String,
}

impl ListRow {
    fn managed(entry: &Entry) -> Self {
        Self {
            id: entry.short_id().to_owned(),
            name: entry.name.clone(),
            status: entry.status.to_string(),
            source: "managed",
            value: entry.value.clone(),
            tips: entry.tips.clone().unwrap_or_else(|| "-".to_owned()),
        }
    }

    fn external(index: usize, value: &str) -> Self {
        Self {
            id: short_external_id(value),
            name: format!("external-{index}"),
            status: "enabled".to_owned(),
            source: "external",
            value: value.to_owned(),
            tips: "-".to_owned(),
        }
    }
}

fn show_entry(entries: &[Entry], current_path: &str, query: &str) -> AppResult<()> {
    if let Ok(index) = find_entry_index(entries, query) {
        let entry = &entries[index];
        println!("id: {}", entry.id);
        println!("name: {}", entry.name);
        println!("value: {}", entry.value);
        println!("status: {}", entry.status);
        println!("source: managed");
        if let Some(tips) = &entry.tips {
            println!("tips: {tips}");
        }
        return Ok(());
    }

    match find_external_entry(entries, current_path, query)? {
        Some(external) => {
            println!("id: {}", external.id);
            println!("name: {}", external.name);
            println!("value: {}", external.value);
            println!("status: enabled");
            println!("source: external");
            Ok(())
        }
        None => Err(format!("entry '{query}' not found").into()),
    }
}

fn short_external_id(value: &str) -> String {
    stable_path_id(value, PATH_CASE)
        .get(..12)
        .unwrap_or("")
        .to_owned()
}

struct ExternalEntry {
    id: String,
    name: String,
    value: String,
}

fn find_external_entry(
    entries: &[Entry],
    current_path: &str,
    query: &str,
) -> AppResult<Option<ExternalEntry>> {
    let managed_values = entries
        .iter()
        .map(|entry| normalize_path_entry(&entry.value, PATH_CASE))
        .collect::<std::collections::HashSet<_>>();

    let matches = split_path(current_path, PATH_SEPARATOR)
        .filter(|value| !managed_values.contains(&normalize_path_entry(value, PATH_CASE)))
        .enumerate()
        .filter_map(|(index, value)| {
            let id = stable_path_id(value, PATH_CASE);
            let short_id = id.get(..12).unwrap_or(&id);
            let name = format!("external-{}", index + 1);
            if id.starts_with(query) || short_id.starts_with(query) || name == query {
                Some(ExternalEntry {
                    id,
                    name,
                    value: value.to_owned(),
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    match matches.len() {
        0 => Ok(None),
        1 => Ok(matches.into_iter().next()),
        _ => Err(format!("entry id prefix '{query}' is ambiguous").into()),
    }
}

fn remove_path_value(store: &dyn EnvStore, current_path: &str, value: &str) -> AppResult<()> {
    let target = normalize_path_entry(value, PATH_CASE);
    let next = split_path(current_path, PATH_SEPARATOR)
        .filter(|part| normalize_path_entry(part, PATH_CASE) != target)
        .collect::<Vec<_>>()
        .join(&PATH_SEPARATOR.to_string());

    store.save_path(&next)
}
