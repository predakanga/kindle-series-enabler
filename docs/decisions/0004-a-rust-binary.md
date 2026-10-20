# 0004: A Rust binary, with SQLite built in

- Status: Proposed
- Date: 2026-10-20

## Context
The current tool is a shell script (`package/series-grouping.sh`) that
needs the `sqlite3` command-line tool. The reference device has it, as
do the other devices seen so far, but nothing guarantees it on every
firmware. The developer prefers a Rust tool (2026-10-20), as Kindle
Wireless Sync is (its ADR 0006): one static armv7 musl binary with no
runtime dependencies.

ADRs 0001 to 0003 need: reading and writing rows in two SQLite
databases that running services hold open, a small state file, and
plain-text output. No daemon (ADR 0003), so no LIPC.

## Decision
**One static binary, `kindle-series-enabler`**, built for
`armv7-unknown-linux-musleabihf`, with these commands:
- `status`: the values, where each comes from (`dcc`, `dcd` default or
  missing), whether the switch is shown, and the original record.
- `install`: record the originals (ADR 0002), then apply (ADR 0001).
- `reapply`: put the override back, and set `dcc` `enabled` back to the
  UserDefaults value only if the two disagree (ADR 0003).
- `restore`: put back what's still ours, then delete the record (ADR
  0002).
- `--appreg PATH`, `--user-defaults PATH` and `--state DIR` options for
  tests and development.

The hooks become thin wrappers: `install.sh` runs `install`,
`uninstall.sh` runs `restore` (except on upgrade), and `launch.sh` runs
`reapply`. The binary runs from the KPM package directory. It's one-shot
and nothing runs it during an upgrade, so it doesn't need copying to
`/var/local` the way Wireless Sync's service does.

**SQLite: `rusqlite` with its `bundled` feature,** which compiles SQLite
into the binary. Every connection gets a busy timeout (5 s, like the
script's `.timeout 5000`), because `appreg.db` is held open by running
services. Each change is a single statement or one transaction.

The repo layout follows Wireless Sync's: `cargo` with a `justfile`,
`just check` (fmt and clippy, plus shellcheck for the hooks), `just test`
(Rust tests against fixture databases built in the tests), and the
version moves to `Cargo.toml`, so `tools/release.sh` and the release
workflow go back to Wireless Sync's versions. The shell script stays
until the binary replaces it, then goes.

## Alternatives considered
- **Keep the shell script:** smallest and simplest, but depends on the
  `sqlite3` CLI being present, and its SQL is built by string
  interpolation (guarded today by checking the marketplace ID's
  characters).
- **Link the device's own `libsqlite3.so`:** a static musl binary can't
  link it, and a dynamic glibc build would tie the binary to the
  firmware's library versions.
- **Write SQLite pages without the library:** no.
- **The `sqlite` crate, or `sqlx`:** `rusqlite` is the usual choice, and
  supports `bundled` directly.

## Consequences
- Binary size: SQLite adds roughly 1 MB. To be measured with `just
  size-kindle` before relying on that figure. KPM packages aren't
  size-critical, but keep it well under 5 MB, as Wireless Sync does.
- The package no longer depends on `sqlite3`, so the tests don't either
  (CI drops the `sqlite3` install).
- Shell stays for the KPM hooks, which KPM runs with `sh`.
