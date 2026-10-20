// SPDX-License-Identifier: GPL-3.0-or-later
//! `apply`, `restore` and `status` against fixture databases (ADR 0001, 0003).

mod common;

use common::{Fixture, AU, DEFAULTS, MARKETPLACES, US};
use kse::appreg::{Appreg, Handler};
use kse::grouping::{self, Applied, Restored, Status};
use kse::state::{self, Record};

fn open(f: &Fixture) -> Appreg {
    Appreg::open(&f.appreg(), true).unwrap()
}

#[test]
fn apply_adds_the_marketplace_to_the_defaults() {
    let f = Fixture::reference();
    let result = grouping::apply(&open(&f), &f.state()).unwrap();
    let written = format!("{DEFAULTS},{AU}");
    assert_eq!(
        result,
        Applied::Written {
            id: AU.into(),
            record: Record {
                previous: None,
                written: written.clone()
            }
        }
    );
    assert_eq!(f.get("dcc", MARKETPLACES), Some(written.clone()));
    assert_eq!(
        state::load(&f.state()).unwrap(),
        Some(Record {
            previous: None,
            written
        })
    );
}

#[test]
fn apply_changes_nothing_else() {
    let f = Fixture::reference();
    let before = f.rows();
    grouping::apply(&open(&f), &f.state()).unwrap();
    let after: Vec<_> = f
        .rows()
        .into_iter()
        .filter(|(h, n, _)| !(h == "dcc" && n == MARKETPLACES))
        .collect();
    assert_eq!(after, before);
    assert_eq!(
        f.get("dcc", "series.grouping.enabled"),
        Some("false".into())
    );
}

#[test]
fn apply_twice_writes_once() {
    let f = Fixture::reference();
    grouping::apply(&open(&f), &f.state()).unwrap();
    let record = state::load(&f.state()).unwrap();
    let result = grouping::apply(&open(&f), &f.state()).unwrap();
    assert_eq!(
        result,
        Applied::AlreadyListed {
            id: AU.into(),
            source: Handler::Dcc
        }
    );
    assert_eq!(f.get("dcc", MARKETPLACES), Some(format!("{DEFAULTS},{AU}")));
    assert_eq!(state::load(&f.state()).unwrap(), record);
}

#[test]
fn a_supported_marketplace_needs_nothing() {
    let f = Fixture::with_marketplace(US);
    let result = grouping::apply(&open(&f), &f.state()).unwrap();
    assert_eq!(
        result,
        Applied::AlreadyListed {
            id: US.into(),
            source: Handler::Dcd
        }
    );
    assert_eq!(f.get("dcc", MARKETPLACES), None);
    assert_eq!(state::load(&f.state()).unwrap(), None);
}

#[test]
fn apply_adds_to_a_pushed_override_and_records_it() {
    let f = Fixture::reference();
    f.set("dcc", MARKETPLACES, US);
    grouping::apply(&open(&f), &f.state()).unwrap();
    assert_eq!(f.get("dcc", MARKETPLACES), Some(format!("{US},{AU}")));
    assert_eq!(
        state::load(&f.state()).unwrap().unwrap().previous,
        Some(US.into())
    );
}

#[test]
fn apply_with_an_empty_list_writes_just_the_marketplace() {
    let f = Fixture::reference();
    f.delete("dcd", MARKETPLACES);
    grouping::apply(&open(&f), &f.state()).unwrap();
    assert_eq!(f.get("dcc", MARKETPLACES), Some(AU.into()));
}

#[test]
fn the_marketplace_falls_back_to_the_default() {
    let f = Fixture::reference();
    f.delete("dcc", "marketplace.obfuscated.id");
    let result = grouping::apply(&open(&f), &f.state()).unwrap();
    assert!(matches!(result, Applied::AlreadyListed { id, .. } if id == US));
}

#[test]
fn an_odd_marketplace_id_is_refused() {
    for id in ["A39 IBJ-37", "A39,IBJ", "a39ibj", ""] {
        let f = Fixture::with_marketplace(id);
        let before = f.rows();
        assert!(grouping::apply(&open(&f), &f.state()).is_err(), "{id:?}");
        assert_eq!(f.rows(), before);
    }
}

#[test]
fn no_marketplace_is_refused() {
    let f = Fixture::reference();
    f.delete("dcc", "marketplace.obfuscated.id");
    f.delete("dcd", "marketplace.obfuscated.id");
    assert!(grouping::apply(&open(&f), &f.state()).is_err());
    assert_eq!(f.get("dcc", MARKETPLACES), None);
}

#[test]
fn grouping_not_supported_changes_nothing() {
    for supported in [Some("false"), None] {
        let f = Fixture::reference();
        match supported {
            Some(v) => f.set("dcc", "series.grouping.supported", v),
            None => f.delete("dcd", "series.grouping.supported"),
        }
        let before = f.rows();
        let err = grouping::apply(&open(&f), &f.state()).unwrap_err();
        assert!(err.to_string().contains("Changed nothing"), "{err}");
        assert_eq!(f.rows(), before);
        assert_eq!(state::load(&f.state()).unwrap(), None);
    }
}

#[test]
fn restore_removes_an_override_that_wasnt_there() {
    let f = Fixture::reference();
    let before = f.rows();
    grouping::apply(&open(&f), &f.state()).unwrap();
    let result = grouping::restore(&open(&f), &f.state()).unwrap();
    assert_eq!(result, Restored::PutBack { previous: None });
    assert_eq!(f.rows(), before);
    assert_eq!(state::load(&f.state()).unwrap(), None);
}

#[test]
fn restore_puts_back_a_replaced_override() {
    let f = Fixture::reference();
    f.set("dcc", MARKETPLACES, US);
    grouping::apply(&open(&f), &f.state()).unwrap();
    grouping::restore(&open(&f), &f.state()).unwrap();
    assert_eq!(f.get("dcc", MARKETPLACES), Some(US.into()));
}

#[test]
fn restore_leaves_an_override_that_changed_since() {
    let f = Fixture::reference();
    grouping::apply(&open(&f), &f.state()).unwrap();
    let pushed = format!("{US},{AU}");
    f.set("dcc", MARKETPLACES, &pushed);
    let result = grouping::restore(&open(&f), &f.state()).unwrap();
    assert_eq!(
        result,
        Restored::LeftAlone {
            current: Some(pushed.clone())
        }
    );
    assert_eq!(f.get("dcc", MARKETPLACES), Some(pushed));
    assert_eq!(state::load(&f.state()).unwrap(), None);
}

#[test]
fn restore_without_a_record_changes_nothing() {
    let f = Fixture::reference();
    f.set("dcc", MARKETPLACES, AU);
    let before = f.rows();
    let result = grouping::restore(&open(&f), &f.state()).unwrap();
    assert_eq!(result, Restored::NoRecord);
    assert_eq!(f.rows(), before);
}

#[test]
fn restore_leaves_enabled_alone() {
    let f = Fixture::reference();
    grouping::apply(&open(&f), &f.state()).unwrap();
    // The user turns grouping on with the Settings switch.
    f.set("dcc", "series.grouping.enabled", "true");
    grouping::restore(&open(&f), &f.state()).unwrap();
    assert_eq!(f.get("dcc", "series.grouping.enabled"), Some("true".into()));
}

#[test]
fn status_shows_the_switch() {
    let f = Fixture::reference();
    let db = Appreg::open(&f.appreg(), false).unwrap();
    let before = Status::read(&db, &f.state()).unwrap();
    assert!(!before.switch_shown());
    grouping::apply(&open(&f), &f.state()).unwrap();
    let after = Status::read(&db, &f.state()).unwrap();
    assert!(after.switch_shown());
    let text = after.to_string();
    assert!(text.contains(&format!("marketplace:   {AU}")), "{text}");
    assert!(text.contains("switch shown:  yes"), "{text}");
    assert!(text.contains("grouping on:   false\n"), "{text}");
    assert!(
        text.contains("wrote the override over no override"),
        "{text}"
    );
}

#[test]
fn a_missing_database_isnt_created() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("appreg.db");
    assert!(Appreg::open(&path, true).is_err());
    assert!(!path.exists());
}
