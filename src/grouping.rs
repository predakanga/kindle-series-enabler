// SPDX-License-Identifier: GPL-3.0-or-later
//! The marketplace override that shows the "Group series" switch (ADR 0001),
//! and putting it back on uninstall (ADR 0003).

use std::fmt;
use std::path::Path;

use crate::appreg::{Appreg, Handler};
use crate::state::{self, Record};

pub const MARKETPLACE_ID: &str = "marketplace.obfuscated.id";
pub const SUPPORTED: &str = "series.grouping.supported";
pub const MARKETPLACES: &str = "series.grouping.supported.marketplaces";
pub const ENABLED: &str = "series.grouping.enabled";

/// This device's marketplace, from `dcc` or else `dcd`. The list is
/// comma-separated, so anything but `A-Z0-9` is refused.
pub fn marketplace(db: &Appreg) -> crate::Result<String> {
    let id = match db.effective(MARKETPLACE_ID)? {
        Some((id, _)) => id,
        None => return Err("this Kindle has no marketplace ID".into()),
    };
    let valid = !id.is_empty()
        && id
            .bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit());
    if !valid {
        return Err(format!("unexpected marketplace ID {id:?}").into());
    }
    Ok(id)
}

fn listed(list: &str, id: &str) -> bool {
    list.split(',').any(|m| m.trim() == id)
}

#[derive(Debug, PartialEq, Eq)]
pub enum Applied {
    /// The marketplace is already in the list in effect: nothing written.
    AlreadyListed { id: String, source: Handler },
    /// The override was written.
    Written { id: String, record: Record },
}

impl fmt::Display for Applied {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Applied::AlreadyListed { id, source } => {
                let whose = match source {
                    Handler::Dcc => "in the override",
                    Handler::Dcd => "in Amazon's defaults",
                };
                writeln!(f, "Marketplace {id} is already listed")?;
                write!(f, "{whose}. Nothing to change.")
            }
            Applied::Written { id, .. } => {
                writeln!(f, "Added marketplace {id} to the list.")?;
                // No ">": KPM 0.2.2 shows hook output HTML-escaped.
                writeln!(f, "Group series is now in Settings, under")?;
                write!(f, "Home and Library, Library tab.")
            }
        }
    }
}

/// Adds this device's marketplace to the override if the list in effect
/// doesn't have it, recording the old value first (ADR 0001, 0003).
pub fn apply(db: &Appreg, state_dir: &Path) -> crate::Result<Applied> {
    let id = marketplace(db)?;
    match db.effective(SUPPORTED)? {
        Some((value, _)) if value == "true" => {}
        other => {
            let value = other.map_or("missing".to_owned(), |(v, _)| format!("{v:?}"));
            return Err(format!("{SUPPORTED} is {value}, not \"true\". Changed nothing.").into());
        }
    }
    let current = db.get(Handler::Dcc, MARKETPLACES)?;
    let (list, source) = match &current {
        Some(list) => (list.clone(), Handler::Dcc),
        None => (
            db.get(Handler::Dcd, MARKETPLACES)?.unwrap_or_default(),
            Handler::Dcd,
        ),
    };
    if listed(&list, &id) {
        return Ok(Applied::AlreadyListed { id, source });
    }
    let written = if list.is_empty() {
        id.clone()
    } else {
        format!("{list},{id}")
    };
    let record = Record {
        previous: current,
        written,
    };
    // Record first: if the write then fails, the row isn't ours, and
    // restore leaves it alone.
    state::save(state_dir, &record)?;
    db.set_dcc(MARKETPLACES, &record.written)?;
    Ok(Applied::Written { id, record })
}

#[derive(Debug, PartialEq, Eq)]
pub enum Restored {
    /// Nothing recorded (never written, or the record was lost).
    NoRecord,
    /// The override still held our value; put back what it replaced.
    PutBack { previous: Option<String> },
    /// The override has changed since (a push): left alone.
    LeftAlone { current: Option<String> },
}

impl fmt::Display for Restored {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Restored::NoRecord => write!(f, "No change recorded; left the override alone."),
            Restored::PutBack { previous: None } => write!(f, "Removed the override."),
            Restored::PutBack { previous: Some(_) } => {
                write!(f, "Put back the override's previous value.")
            }
            Restored::LeftAlone { .. } => {
                write!(f, "The override changed since; left it alone.")
            }
        }
    }
}

/// Puts the override back as it was before `apply`'s last write, if it still
/// holds that write's value, then deletes the record (ADR 0003).
pub fn restore(db: &Appreg, state_dir: &Path) -> crate::Result<Restored> {
    let Some(record) = state::load(state_dir)? else {
        return Ok(Restored::NoRecord);
    };
    let current = db.get(Handler::Dcc, MARKETPLACES)?;
    let result = if current.as_deref() == Some(record.written.as_str()) {
        match &record.previous {
            None => db.delete_dcc(MARKETPLACES)?,
            Some(previous) => db.set_dcc(MARKETPLACES, previous)?,
        }
        Restored::PutBack {
            previous: record.previous,
        }
    } else {
        Restored::LeftAlone { current }
    };
    state::remove(state_dir)?;
    Ok(result)
}

/// Everything `status` shows.
pub struct Status {
    pub marketplace: crate::Result<String>,
    pub supported: Option<(String, Handler)>,
    pub defaults: Option<String>,
    pub overridden: Option<String>,
    pub enabled: Option<(String, Handler)>,
    pub record: crate::Result<Option<Record>>,
}

impl Status {
    pub fn read(db: &Appreg, state_dir: &Path) -> crate::Result<Status> {
        Ok(Status {
            marketplace: marketplace(db),
            supported: db.effective(SUPPORTED)?,
            defaults: db.get(Handler::Dcd, MARKETPLACES)?,
            overridden: db.get(Handler::Dcc, MARKETPLACES)?,
            enabled: db.effective(ENABLED)?,
            record: state::load(state_dir),
        })
    }

    /// Whether Settings shows the switch: the list in effect has this
    /// device's marketplace.
    pub fn switch_shown(&self) -> bool {
        let list = self.overridden.as_ref().or(self.defaults.as_ref());
        match (&self.marketplace, list) {
            (Ok(id), Some(list)) => listed(list, id),
            _ => false,
        }
    }
}

fn sourced(value: &Option<(String, Handler)>) -> String {
    match value {
        Some((v, Handler::Dcc)) => v.clone(),
        Some((v, Handler::Dcd)) => format!("{v} (default)"),
        None => "missing".to_owned(),
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.marketplace {
            Ok(id) => writeln!(f, "marketplace:   {id}")?,
            Err(e) => writeln!(f, "marketplace:   {e}")?,
        }
        writeln!(f, "supported:     {}", sourced(&self.supported))?;
        let shown = if self.switch_shown() { "yes" } else { "no" };
        writeln!(f, "switch shown:  {shown}")?;
        writeln!(f, "grouping on:   {}", sourced(&self.enabled))?;
        writeln!(
            f,
            "defaults:      {}",
            self.defaults.as_deref().unwrap_or("none")
        )?;
        writeln!(
            f,
            "override:      {}",
            self.overridden.as_deref().unwrap_or("none")
        )?;
        match &self.record {
            Ok(None) => write!(f, "recorded:      nothing"),
            Ok(Some(r)) => {
                let previous = r.previous.as_deref().unwrap_or("no override");
                write!(f, "recorded:      wrote the override over {previous}")
            }
            Err(e) => write!(f, "recorded:      {e}"),
        }
    }
}
