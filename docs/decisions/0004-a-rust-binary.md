# 0004: A Rust binary, with SQLite and a JSON parser built in

- Status: Proposed
- Date: 2026-10-20

## Context
The package (ADRs 0001 to 0003) has to:
- read and write rows in `appreg.db`, which running services hold open;
- add and remove a key in up to three JSON files on the root filesystem.
  A broken file would break every search command, and KMC's patcher with
  them (`device-hooks.md`);
- record the override's previous value in a small state file.

What the device offers (`device-hooks.md`):
- `sqlite3` is there wherever KMC is (KMC uses it). But the developer's
  testing found older firmware whose `sqlite3` has no JSON functions.
- There's no `jq` and no `python3`, so the shell has nothing that parses
  JSON.

The developer prefers Rust (2026-10-20), as Kindle Wireless Sync uses
(its ADR 0006).

## Decision
**One static binary, `kindle-series-enabler`,** built for
`armv7-unknown-linux-musleabihf`, with these commands:
- `apply` (**the default, with no arguments**): ADR 0001.
- `status`: the marketplace, `series.grouping.supported`, the `dcc` and
  `dcd` lists, whether the switch is shown, `enabled` (and whether that's
  the `dcd` default), and the state file's record.
- `register` / `unregister`: add or remove `;series` in the command files.
  `unregister` removes it only if its value is ours.
- `restore`: ADR 0003.
- Options for tests and development: `--appreg PATH`, `--state DIR`, and
  `--commands FILE` (repeatable; by default whichever of the three
  command files exist).

How it works:
- **SQLite:** `rusqlite` with `bundled`, so the device's `sqlite3` doesn't
  matter. Every connection gets a 5-second busy timeout, and each change
  is one statement or one transaction. Values are bound, not interpolated.
- **JSON:** `serde_json` with `preserve_order`, so the other entries stay
  in order. It refuses to write a file that isn't a JSON object of strings,
  and leaves it untouched. It writes tab-indented, like KMC's patcher, to a
  temporary file in the same directory, keeps the original's mode, then
  renames it into place.
- **Output:** plain lines on stdout (errors on stderr), and an exit code.
  The shell entry points decide where the lines go (ADR 0002).

Shell stays for what KPM and upstart run: the hooks, `boot.sh`,
`series.sh` and `launch.sh`, with `mntroot` and the FBInk loop.

The repo follows Wireless Sync's layout: `cargo` with a `justfile`, `just
check` (fmt, clippy and shellcheck), `just test` (Rust tests against
fixture databases and command files). The version moves to `Cargo.toml`,
so `tools/release.sh` and the release workflow go back to Wireless Sync's
versions. `package/series-grouping.sh` stays as a dev tool until `status`
replaces it.

## Alternatives considered
- **Shell with `sqlite3`'s JSON functions:** works on 5.20.1, but older
  firmware's `sqlite3` lacks them.
- **Shell with a small awk parser** for the flat command files: no
  toolchain, but it's our own JSON parser to get right, on a file whose
  failure breaks every search command. The SQL is still built from
  strings.
- **Shell editing the files line by line with `sed`:** depends on the
  exact layout KMC's patcher writes.
- **Link the device's own `libsqlite3.so`:** a static musl binary can't
  link it, and a dynamic build would tie the binary to the firmware's
  library versions.

## Consequences
- Binary size: SQLite adds roughly 1 MB. Measure it with `just
  size-kindle` before relying on that figure, and keep it well under 5 MB,
  as Wireless Sync does.
- The tests and CI no longer need `sqlite3`.
- A cross-compiling toolchain, as in Wireless Sync.
