// SPDX-License-Identifier: GPL-3.0-or-later
//! `/var/local/appreg.db`'s `properties` table: `dcd` holds the defaults,
//! `dcc` the current values that override them (research note
//! `series-grouping-gate.md`, "Where the values live").

use std::fmt;
use std::path::Path;
use std::time::Duration;

use rusqlite::{params, Connection, OpenFlags, OptionalExtension};

pub const DEFAULT_PATH: &str = "/var/local/appreg.db";

/// Running services hold the database open, so wait out their locks.
const BUSY_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Handler {
    /// Current values, written by dynconfig and the Settings switch.
    Dcc,
    /// Defaults, from the firmware's `ServerConfig.db`.
    Dcd,
}

impl Handler {
    fn as_str(self) -> &'static str {
        match self {
            Handler::Dcc => "dcc",
            Handler::Dcd => "dcd",
        }
    }
}

impl fmt::Display for Handler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

pub struct Appreg {
    conn: Connection,
}

impl Appreg {
    /// Opens an existing database (never creates one). `writable` is false
    /// for commands that only read.
    pub fn open(path: &Path, writable: bool) -> rusqlite::Result<Self> {
        let mode = if writable {
            OpenFlags::SQLITE_OPEN_READ_WRITE
        } else {
            OpenFlags::SQLITE_OPEN_READ_ONLY
        };
        let conn = Connection::open_with_flags(path, mode | OpenFlags::SQLITE_OPEN_NO_MUTEX)?;
        conn.busy_timeout(BUSY_TIMEOUT)?;
        Ok(Appreg { conn })
    }

    /// One handler's value. A row whose value is NULL counts as missing.
    pub fn get(&self, handler: Handler, name: &str) -> rusqlite::Result<Option<String>> {
        let value: Option<Option<String>> = self
            .conn
            .query_row(
                "SELECT value FROM properties WHERE handlerId = ?1 AND name = ?2",
                params![handler.as_str(), name],
                |row| row.get(0),
            )
            .optional()?;
        Ok(value.flatten())
    }

    /// The value in effect: `dcc` if it has one, else the `dcd` default.
    pub fn effective(&self, name: &str) -> rusqlite::Result<Option<(String, Handler)>> {
        for handler in [Handler::Dcc, Handler::Dcd] {
            if let Some(value) = self.get(handler, name)? {
                return Ok(Some((value, handler)));
            }
        }
        Ok(None)
    }

    pub fn set_dcc(&self, name: &str, value: &str) -> rusqlite::Result<()> {
        // The primary key is ON CONFLICT REPLACE, as dynconfig relies on.
        self.conn.execute(
            "INSERT OR REPLACE INTO properties (handlerId, name, value) VALUES ('dcc', ?1, ?2)",
            params![name, value],
        )?;
        Ok(())
    }

    pub fn delete_dcc(&self, name: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "DELETE FROM properties WHERE handlerId = 'dcc' AND name = ?1",
            params![name],
        )?;
        Ok(())
    }
}
