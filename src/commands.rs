// SPDX-License-Identifier: GPL-3.0-or-later
//! The `;series` search-bar command, in the JSON files KPPMainApp reads at
//! startup (research note `device-hooks.md`, ADR 0003). A broken file would
//! break every search command, so a file is only written after it parses as
//! an object of strings, and is replaced by renaming a complete new copy.

use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::{Map, Value};

pub const COMMAND: &str = ";series";
pub const TARGET: &str = "/bin/sh /var/local/kindle-series-enabler/series.sh";

/// Every file that has held search commands. KMC's patcher writes to each
/// one that exists, and so do we.
pub const DEFAULT_FILES: [&str; 3] = [
    "/app/kpp_app_cmds.json",
    "/usr/share/app/kpp_sys_cmds.json",
    "/usr/share/webkit-1.0/pillow/debug_cmds.json",
];

#[derive(Debug, PartialEq, Eq)]
pub enum Entry {
    /// The file doesn't exist.
    NoFile,
    /// No `;series`.
    Absent,
    /// `;series` runs our script.
    Ours,
    /// `;series` runs something else.
    Other(String),
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Entry::NoFile => write!(f, "no file"),
            Entry::Absent => write!(f, "not registered"),
            Entry::Ours => write!(f, "registered"),
            Entry::Other(v) => write!(f, "{COMMAND} runs {v:?}"),
        }
    }
}

/// Reads a command file, or `None` if it doesn't exist.
fn read(path: &Path) -> crate::Result<Option<Map<String, Value>>> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("{}: {e}", path.display()).into()),
    };
    let bad = || format!("{} is not a JSON object of strings", path.display());
    let Ok(Value::Object(map)) = serde_json::from_str(&text) else {
        return Err(bad().into());
    };
    if !map.values().all(Value::is_string) {
        return Err(bad().into());
    }
    Ok(Some(map))
}

fn entry(map: &Option<Map<String, Value>>) -> Entry {
    let Some(map) = map else {
        return Entry::NoFile;
    };
    match map.get(COMMAND).and_then(Value::as_str) {
        None => Entry::Absent,
        Some(TARGET) => Entry::Ours,
        Some(other) => Entry::Other(other.to_owned()),
    }
}

/// What each file holds for `;series`.
pub fn check(files: &[PathBuf]) -> Vec<(PathBuf, crate::Result<Entry>)> {
    files
        .iter()
        .map(|path| (path.clone(), read(path).map(|map| entry(&map))))
        .collect()
}

/// Writes tab-indented, like KMC's patcher, to a temporary file in the same
/// directory with the original's permissions, then renames it into place.
fn write(path: &Path, map: &Map<String, Value>) -> io::Result<()> {
    let mut text = Vec::new();
    let mut ser = serde_json::Serializer::with_formatter(
        &mut text,
        serde_json::ser::PrettyFormatter::with_indent(b"\t"),
    );
    map.serialize(&mut ser).map_err(io::Error::other)?;
    text.push(b'\n');

    let name = path.file_name().unwrap_or_default().to_string_lossy();
    let tmp = path.with_file_name(format!(".{name}.kse-new"));
    let permissions = fs::metadata(path)?.permissions();
    let result = (|| {
        let mut file = fs::File::create(&tmp)?;
        file.write_all(&text)?;
        file.set_permissions(permissions)?;
        file.sync_all()?;
        fs::rename(&tmp, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result
}

#[derive(Debug, PartialEq, Eq)]
pub enum Change {
    NoFile,
    Unchanged,
    Added,
    Removed,
    /// `unregister` found someone else's `;series` and kept it.
    KeptOther,
}

impl fmt::Display for Change {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Change::NoFile => "no file",
            Change::Unchanged => "unchanged",
            Change::Added => "added",
            Change::Removed => "removed",
            Change::KeptOther => "kept (not ours)",
        })
    }
}

/// Reads every file first, so a bad file or someone else's `;series`
/// stops it before anything is written.
fn read_all(files: &[PathBuf]) -> crate::Result<Vec<Option<Map<String, Value>>>> {
    files.iter().map(|path| read(path)).collect()
}

/// Adds `;series` to every file that exists. Refuses, writing nothing, if
/// any file holds a different `;series`.
pub fn register(files: &[PathBuf]) -> crate::Result<Vec<(PathBuf, Change)>> {
    let maps = read_all(files)?;
    for (path, map) in files.iter().zip(&maps) {
        if let Entry::Other(v) = entry(map) {
            return Err(format!(
                "{} already has {COMMAND} for {v:?}. Changed nothing.",
                path.display()
            )
            .into());
        }
    }
    let mut changes = Vec::new();
    for (path, map) in files.iter().zip(maps) {
        let change = match (entry(&map), map) {
            (_, None) => Change::NoFile,
            (Entry::Ours, Some(_)) => Change::Unchanged,
            (_, Some(mut map)) => {
                map.insert(COMMAND.to_owned(), Value::String(TARGET.to_owned()));
                write(path, &map).map_err(|e| format!("{}: {e}", path.display()))?;
                Change::Added
            }
        };
        changes.push((path.clone(), change));
    }
    Ok(changes)
}

/// Removes `;series` from every file where it's ours.
pub fn unregister(files: &[PathBuf]) -> crate::Result<Vec<(PathBuf, Change)>> {
    let maps = read_all(files)?;
    let mut changes = Vec::new();
    for (path, map) in files.iter().zip(maps) {
        let change = match entry(&map) {
            Entry::NoFile => Change::NoFile,
            Entry::Absent => Change::Unchanged,
            Entry::Other(_) => Change::KeptOther,
            Entry::Ours => {
                let mut map = map.expect("Ours implies a file");
                // shift_remove keeps the other entries in order.
                map.shift_remove(COMMAND);
                write(path, &map).map_err(|e| format!("{}: {e}", path.display()))?;
                Change::Removed
            }
        };
        changes.push((path.clone(), change));
    }
    Ok(changes)
}
