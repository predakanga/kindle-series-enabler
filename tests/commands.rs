// SPDX-License-Identifier: GPL-3.0-or-later
//! `register` and `unregister` against fixture command files (ADR 0003, 0004).

mod common;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use common::{keys, write, APP_CMDS, SYS_CMDS};
use kse::commands::{self, Change, Entry, COMMAND, TARGET};
use tempfile::TempDir;

/// The app and sys files, plus a missing pillow file.
fn files() -> (TempDir, Vec<PathBuf>) {
    let dir = tempfile::tempdir().unwrap();
    let app = dir.path().join("kpp_app_cmds.json");
    let sys = dir.path().join("kpp_sys_cmds.json");
    write(&app, APP_CMDS);
    write(&sys, SYS_CMDS);
    fs::set_permissions(&app, fs::Permissions::from_mode(0o755)).unwrap();
    fs::set_permissions(&sys, fs::Permissions::from_mode(0o644)).unwrap();
    let pillow = dir.path().join("debug_cmds.json");
    (dir, vec![app, sys, pillow])
}

fn value(path: &PathBuf, key: &str) -> Option<String> {
    let text = fs::read_to_string(path).unwrap();
    let map: serde_json::Value = serde_json::from_str(&text).unwrap();
    map[key].as_str().map(str::to_owned)
}

fn mode(path: &PathBuf) -> u32 {
    fs::metadata(path).unwrap().permissions().mode() & 0o777
}

#[test]
fn register_adds_to_each_file_that_exists() {
    let (_dir, files) = files();
    let changes = commands::register(&files).unwrap();
    let changes: Vec<_> = changes.into_iter().map(|(_, c)| c).collect();
    assert_eq!(changes, [Change::Added, Change::Added, Change::NoFile]);
    assert_eq!(value(&files[0], COMMAND).as_deref(), Some(TARGET));
    assert_eq!(value(&files[1], COMMAND).as_deref(), Some(TARGET));
    assert!(!files[2].exists());
}

#[test]
fn register_keeps_the_other_entries_in_order() {
    let (_dir, files) = files();
    let before = keys(&files[0]);
    commands::register(&files).unwrap();
    let mut expected = before;
    expected.push(COMMAND.to_owned());
    assert_eq!(keys(&files[0]), expected);
    assert_eq!(
        value(&files[0], ";kppchrome2").as_deref(),
        Some("/app/tools/switchChrome.sh kpp 2")
    );
}

#[test]
fn register_keeps_permissions_and_leaves_no_temporary_files() {
    let (dir, files) = files();
    commands::register(&files).unwrap();
    assert_eq!(mode(&files[0]), 0o755);
    assert_eq!(mode(&files[1]), 0o644);
    let names: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    assert_eq!(names.len(), 2, "{names:?}");
}

#[test]
fn register_twice_changes_nothing() {
    let (_dir, files) = files();
    commands::register(&files).unwrap();
    let text = fs::read_to_string(&files[0]).unwrap();
    let changes = commands::register(&files).unwrap();
    assert!(changes
        .iter()
        .all(|(_, c)| matches!(c, Change::Unchanged | Change::NoFile)));
    assert_eq!(fs::read_to_string(&files[0]).unwrap(), text);
}

#[test]
fn register_refuses_someone_elses_series_and_writes_nothing() {
    let (_dir, files) = files();
    let other = SYS_CMDS.replace("\n}", ",\n\t\";series\":\t\"/mnt/us/other.sh\"\n}");
    write(&files[1], &other);
    let err = commands::register(&files).unwrap_err();
    assert!(err.to_string().contains("Changed nothing"), "{err}");
    assert_eq!(fs::read_to_string(&files[0]).unwrap(), APP_CMDS);
    assert_eq!(fs::read_to_string(&files[1]).unwrap(), other);
}

#[test]
fn a_broken_file_stops_everything() {
    for broken in ["{ \";kpm\": ", "[\"a\"]", "{\";x\": 1}", ""] {
        let (_dir, files) = files();
        write(&files[1], broken);
        assert!(commands::register(&files).is_err(), "{broken:?}");
        assert_eq!(fs::read_to_string(&files[0]).unwrap(), APP_CMDS);
        assert_eq!(fs::read_to_string(&files[1]).unwrap(), broken);
        assert!(commands::unregister(&files).is_err(), "{broken:?}");
    }
}

#[test]
fn unregister_puts_the_files_back() {
    let (_dir, files) = files();
    let before = (keys(&files[0]), keys(&files[1]));
    commands::register(&files).unwrap();
    let changes = commands::unregister(&files).unwrap();
    let changes: Vec<_> = changes.into_iter().map(|(_, c)| c).collect();
    assert_eq!(changes, [Change::Removed, Change::Removed, Change::NoFile]);
    assert_eq!((keys(&files[0]), keys(&files[1])), before);
}

#[test]
fn unregister_keeps_someone_elses_series() {
    let (_dir, files) = files();
    let other = SYS_CMDS.replace("\n}", ",\n\t\";series\":\t\"/mnt/us/other.sh\"\n}");
    write(&files[1], &other);
    let changes = commands::unregister(&files).unwrap();
    assert_eq!(changes[0].1, Change::Unchanged);
    assert_eq!(changes[1].1, Change::KeptOther);
    assert_eq!(fs::read_to_string(&files[1]).unwrap(), other);
}

#[test]
fn check_reports_each_file() {
    let (_dir, files) = files();
    commands::register(&files[..1]).unwrap();
    let entries: Vec<_> = commands::check(&files)
        .into_iter()
        .map(|(_, e)| e.unwrap())
        .collect();
    assert_eq!(entries, [Entry::Ours, Entry::Absent, Entry::NoFile]);
}
