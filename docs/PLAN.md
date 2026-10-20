# Plan

## Goal

A KPM package for jailbroken Kindles that turns on the built-in Library's
**series grouping** (one tile per series, in order) on devices whose
marketplace Amazon doesn't offer it, such as Australia. It has **no UI**.
It makes grouping *available* by showing Amazon's own "Group series"
switch in Settings, and the user turns grouping on there. A one-shot job
re-applies it at boot, and typing `;series` in the search bar does the
same by hand.

It was split out of
[Kindle Wireless Sync](https://github.com/predakanga/kindle-wireless-sync)
on 2026-10-20. That package sends Calibre series as native series (its ADR
0016), which only show where grouping is on. Its plan had this as "Later
4": a packaged script, kept out of its UI, because Wireless Sync mustn't
change Amazon's settings itself.

## Requirements

1. Installs, upgrades and uninstalls through KPM, with releases on GitHub
   and a KPM index on GitHub Pages, as Wireless Sync does
   (`docs/research/kpm-packaging.md`).
2. No UI and no daemon: a one-shot boot job and a `;series` search-bar
   command (developer, 2026-10-20).
3. Changes only what makes grouping available: one `dcc` row in
   `/var/local/appreg.db` (`docs/research/series-grouping-gate.md`), plus
   its own job and search command. Never writes `cc.db` or
   `ksdk.asset.db`.
4. Uninstalling removes what the package added. It leaves the user's own
   grouping choice alone.
5. The README says plainly that this changes an Amazon-side setting, and
   what that could mean.
6. Reference device: Kindle Paperwhite 12th Gen on FW 5.20.1 (AU). Other
   devices and firmware are best-effort.

## Phases

### Phase 0: Set up the repository ✅ (2026-10-20)

- ✅ Research moved from Wireless Sync: the gate (`series-grouping-gate.md`)
  and KPM packaging (`kpm-packaging.md`).
- ✅ `package/series-grouping.sh` (Wireless Sync's dev tool), with tests
  against a fixture database (`just test`).
- ✅ Release tooling modelled on Wireless Sync: `just package`, `just
  release X.Y.Z`, `just publish-release`, and the CI and release
  workflows. The version lives in `VERSION`.
- ✅ Draft hooks: `install.sh` runs `enable` and `uninstall.sh` runs
  `disable` (skipped on upgrade). Phase 2 decides what they really do.

### Phase 1: Research

1. **What resets it.** *Mostly answered (2026-10-20, research note "What
   refreshes `dcc`"):* `dynconfig` merges server-pushed config files into
   `dcc`, only when Amazon queues one, so keys a file doesn't mention
   survive. After each push it sends `com.lab126.dynconfig`
   `dynamicConfigUpdated` with the key names, and the shell can subscribe
   to it. Still open: whether a real push ever includes the grouping keys
   (a listener is logging events on the device), and what re-writes
   `enabled` at boot. Original question: Which process writes `dcc` rows in `appreg.db`, and
   what triggers a server config refresh (boot, network up, registration,
   a timer?). Does a refresh set `series.grouping.enabled` back to `false`,
   or drop the marketplace override? Decides whether the package needs to
   re-apply the values, and how (an upstart job needs a root filesystem
   write). Also: **is there a notification when `dcc` is refreshed** (an
   LIPC event, an upstart event, a file change) that the package could
   react to?
2. **Restoring what was there.** The `disable` command assumes the
   reference device's starting state (`enabled = false`, no override).
   Check what other devices start with, and whether recording the values at
   install and restoring them on uninstall is better.
3. **The Settings switch.** With the override in place, check that tapping
   "Group series" in Settings turns grouping off and on, and that the
   choice lasts.
4. **Other devices and firmware.** The developer also has a Paperwhite
   10th Gen on FW 5.17.1. Look for the same values there, and for devices
   with the library re-architecture, where UserDefaults holds the choice
   instead. A device in a supported marketplace needs nothing.
5. **Quick Settings tile** (lowest priority). *Answered (2026-10-20):* the
   legacy panel this device uses has no group-series tile; the newer
   panel that would show it is gated by a feature with no config on
   5.20.1, so it's off. Not pursued further (research note, "Quick
   Settings").

### Phase 2: Decide the package's behaviour ✅ (2026-10-20)

All accepted on 2026-10-20, for the scope the developer set that day:
- ADR 0001: the package writes only the marketplace override. The user
  turns grouping on in Settings.
- ADR 0002: `apply` runs from a one-shot boot job (logs to `/mnt/us`, or
  `/var/local` if that fails), `;series` (logs to the screen) and `kpm
  launch` (stdout). `install.sh` runs it once after setting up. There's no
  listener.
- ADR 0003: files in `/var/local/kindle-series-enabler/`. The job and
  `;series` need a root filesystem write. `;series` goes in every command
  file that exists, as KMC does. Uninstall restores the override only if
  it's still ours, and leaves `enabled` alone.
- ADR 0004: a Rust binary (`apply` by default), with SQLite and a JSON
  parser built in. Older firmware's `sqlite3` has no JSON functions.

The search-bar tests (`device-hooks.md`) showed that commands are read at
startup, run as root, get the rest of the line as arguments, and do
nothing if their target is missing.

### Phase 3: Build and release

- The Rust binary (ADR 0004): `apply`, `status`, `register`/`unregister`
  and `restore`, with tests against fixture databases and command files.
  Measure its size.
- The shell around it: `install.sh`, `uninstall.sh`, `launch.sh`,
  `boot.sh`, `series.sh` and the upstart job, safe to run again. Move the
  version to `Cargo.toml` and the tooling back to Wireless Sync's.
- The README: what it does, the risk, how to undo it by hand.
- An install, upgrade (from a temporary repository) and uninstall test on
  the reference device.
- First release, 0.1.0 (ask before publishing).

## Open questions

- See Phase 1 (still open: whether a push ever sets the override, the
  boot-time writer, other devices and firmware).

## Later

- Nothing yet.
