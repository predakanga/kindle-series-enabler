// SPDX-License-Identifier: GPL-3.0-or-later
//! The record of `apply`'s last write to the override, so that uninstall can
//! put back what it replaced (ADR 0003).

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

pub const DEFAULT_DIR: &str = "/var/local/kindle-series-enabler";
const FILE: &str = "state";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    /// The override's value before the write, or `None` if there was no row.
    pub previous: Option<String>,
    /// The value `apply` wrote.
    pub written: String,
}

fn path(dir: &Path) -> PathBuf {
    dir.join(FILE)
}

pub fn load(dir: &Path) -> crate::Result<Option<Record>> {
    let text = match fs::read_to_string(path(dir)) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    let bad = || format!("{} is not a valid record", path(dir).display());
    let value: Value = serde_json::from_str(&text).map_err(|_| bad())?;
    let written = value["written"].as_str().ok_or_else(bad)?.to_owned();
    let previous = match &value["previous"] {
        Value::Null => None,
        Value::String(s) => Some(s.clone()),
        _ => return Err(bad().into()),
    };
    Ok(Some(Record { previous, written }))
}

/// Replaces the record, writing a temporary file and renaming it into place.
pub fn save(dir: &Path, record: &Record) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    let text = json!({ "previous": record.previous, "written": record.written });
    let tmp = dir.join(format!(".{FILE}.new"));
    let mut file = fs::File::create(&tmp)?;
    writeln!(file, "{text}")?;
    file.sync_all()?;
    fs::rename(&tmp, path(dir))
}

pub fn remove(dir: &Path) -> io::Result<()> {
    match fs::remove_file(path(dir)) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        other => other,
    }
}
