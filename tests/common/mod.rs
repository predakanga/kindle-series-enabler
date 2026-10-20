// SPDX-License-Identifier: GPL-3.0-or-later
//! Fixtures: an `appreg.db` with the reference device's schema and values
//! (FW 5.20.1, AU marketplace; research note `series-grouping-gate.md`).
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use rusqlite::{params, Connection};
use tempfile::TempDir;

pub const US: &str = "ATVPDKIKX0DER";
pub const AU: &str = "A39IBJ37TRP1C6";
pub const DEFAULTS: &str = "A1VC38T7YXB528,ATVPDKIKX0DER,A1F83G8C2ARO7P,A1PA6795UKMFR9";
pub const MARKETPLACES: &str = "series.grouping.supported.marketplaces";

pub struct Fixture {
    pub dir: TempDir,
}

impl Fixture {
    /// The reference device as found, with `marketplace` as the dcc
    /// marketplace ID.
    pub fn with_marketplace(marketplace: &str) -> Fixture {
        let dir = tempfile::tempdir().unwrap();
        let conn = Connection::open(dir.path().join("appreg.db")).unwrap();
        conn.execute_batch(
            "CREATE TABLE handlerIds ( handlerId TEXT PRIMARY KEY NOT NULL ON CONFLICT IGNORE );
             CREATE TABLE properties ( handlerId TEXT NOT NULL REFERENCES handlerIds( handlerId ),
                 name TEXT NOT NULL, value TEXT, PRIMARY KEY (handlerId, name) ON CONFLICT REPLACE );
             INSERT INTO handlerIds VALUES ('dcc'), ('dcd');",
        )
        .unwrap();
        let rows = [
            ("dcd", "marketplace.obfuscated.id", US),
            ("dcd", "series.grouping.enabled", "true"),
            ("dcd", "series.grouping.supported", "true"),
            ("dcd", MARKETPLACES, DEFAULTS),
            ("dcc", "marketplace.obfuscated.id", marketplace),
            ("dcc", "series.grouping.enabled", "false"),
        ];
        for (handler, name, value) in rows {
            conn.execute(
                "INSERT INTO properties VALUES (?1, ?2, ?3)",
                params![handler, name, value],
            )
            .unwrap();
        }
        Fixture { dir }
    }

    pub fn reference() -> Fixture {
        Fixture::with_marketplace(AU)
    }

    pub fn appreg(&self) -> PathBuf {
        self.dir.path().join("appreg.db")
    }

    pub fn state(&self) -> PathBuf {
        self.dir.path().join("state")
    }

    fn conn(&self) -> Connection {
        Connection::open(self.appreg()).unwrap()
    }

    pub fn get(&self, handler: &str, name: &str) -> Option<String> {
        self.conn()
            .query_row(
                "SELECT value FROM properties WHERE handlerId = ?1 AND name = ?2",
                params![handler, name],
                |row| row.get(0),
            )
            .ok()
    }

    pub fn set(&self, handler: &str, name: &str, value: &str) {
        self.conn()
            .execute(
                "INSERT OR REPLACE INTO properties VALUES (?1, ?2, ?3)",
                params![handler, name, value],
            )
            .unwrap();
    }

    pub fn delete(&self, handler: &str, name: &str) {
        self.conn()
            .execute(
                "DELETE FROM properties WHERE handlerId = ?1 AND name = ?2",
                params![handler, name],
            )
            .unwrap();
    }

    /// Every row, to check that nothing else changed.
    pub fn rows(&self) -> Vec<(String, String, Option<String>)> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare("SELECT handlerId, name, value FROM properties ORDER BY handlerId, name")
            .unwrap();
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    }
}

/// The reference device's `/app/kpp_app_cmds.json` as KMC leaves it
/// (shortened), in KMC's cJSON layout.
pub const APP_CMDS: &str = "{\n\t\";framer\":\t\"/app/tools/framerTool.sh\",\n\t\";kppchrome2\":\t\"/app/tools/switchChrome.sh kpp 2\",\n\t\";kpm\":\t\"/var/local/kmc/sbin/kpm.sh\",\n\t\";log\":\t\"/usr/bin/logThis.sh\",\n\t\";kmclog\":\t\"/var/local/kmc/sbin/kmclog.sh\"\n}";

/// The same for `/usr/share/app/kpp_sys_cmds.json`.
pub const SYS_CMDS: &str = "{\n\t\";711\":\t\"/usr/bin/711.sh\",\n\t\";un\":\t\"/usr/local/bin/usbnetwork.sh start\",\n\t\"freset\":\t\"/bin/sh -c /usr/sbin/factory_reset\",\n\t\";kpm\":\t\"/var/local/kmc/sbin/kpm.sh\"\n}";

pub fn write(path: &Path, text: &str) {
    std::fs::write(path, text).unwrap();
}

/// The keys of a command file, in order.
pub fn keys(path: &Path) -> Vec<String> {
    let text = std::fs::read_to_string(path).unwrap();
    let map: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&text).unwrap();
    map.keys().cloned().collect()
}
