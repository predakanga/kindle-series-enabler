// SPDX-License-Identifier: GPL-3.0-or-later
//! The binary's command line: `apply` by default, exit codes, output.

mod common;

use std::process::{Command, Output};

use common::{Fixture, AU, DEFAULTS, MARKETPLACES};

fn run(f: &Fixture, args: &[&str]) -> Output {
    let commands = f.dir.path().join("cmds.json");
    if !commands.exists() {
        common::write(&commands, common::APP_CMDS);
    }
    Command::new(env!("CARGO_BIN_EXE_kindle-series-enabler"))
        .args(args)
        .arg("--appreg")
        .arg(f.appreg())
        .arg("--state")
        .arg(f.state())
        .arg("--commands")
        .arg(&commands)
        .output()
        .unwrap()
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

#[test]
fn no_command_means_apply() {
    let f = Fixture::reference();
    let o = run(&f, &[]);
    assert!(o.status.success(), "{o:?}");
    assert!(
        stdout(&o).contains(&format!("Added marketplace {AU}")),
        "{o:?}"
    );
    assert_eq!(f.get("dcc", MARKETPLACES), Some(format!("{DEFAULTS},{AU}")));
}

#[test]
fn apply_again_says_already_listed() {
    let f = Fixture::reference();
    let o = run(&f, &[]);
    assert!(o.status.success());
    let o = run(&f, &["apply"]);
    assert!(stdout(&o).contains("already listed"), "{o:?}");
}

#[test]
fn an_unknown_command_fails_and_changes_nothing() {
    let f = Fixture::reference();
    let o = run(&f, &["foo"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&o.stderr).contains("unknown command \"foo\""));
    assert_eq!(f.get("dcc", MARKETPLACES), None);
}

#[test]
fn an_error_exits_non_zero() {
    let f = Fixture::with_marketplace("A39 IBJ");
    let o = run(&f, &[]);
    assert_eq!(o.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&o.stderr).starts_with("error: unexpected marketplace ID"));
}

#[test]
fn status_reads_without_writing() {
    let f = Fixture::reference();
    let before = f.rows();
    let o = run(&f, &["status"]);
    assert!(o.status.success(), "{o:?}");
    let text = stdout(&o);
    assert!(text.contains("switch shown:  no"), "{text}");
    assert!(text.contains("cmds.json: not registered"), "{text}");
    assert_eq!(f.rows(), before);
}

#[test]
fn register_restore_and_unregister() {
    let f = Fixture::reference();
    assert!(run(&f, &["register"]).status.success());
    assert!(stdout(&run(&f, &["status"])).contains("cmds.json: registered"));
    run(&f, &[]);
    let o = run(&f, &["restore"]);
    assert!(stdout(&o).contains("Removed the override."), "{o:?}");
    assert_eq!(f.get("dcc", MARKETPLACES), None);
    let o = run(&f, &["unregister"]);
    assert!(stdout(&o).contains("cmds.json: removed"), "{o:?}");
}

#[test]
fn help_prints_usage() {
    let f = Fixture::reference();
    let o = run(&f, &["--help"]);
    assert!(o.status.success());
    assert!(stdout(&o).starts_with("usage: kindle-series-enabler"));
    assert_eq!(f.get("dcc", MARKETPLACES), None);
}
