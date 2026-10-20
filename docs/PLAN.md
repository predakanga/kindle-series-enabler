# Plan

## Goal

A KPM package for jailbroken Kindles that turns on the built-in Library's
**series grouping** (one tile per series, in order) on devices whose
marketplace Amazon doesn't offer it, such as Australia. It has **no UI**:
installing and uninstalling the package are the controls. Whether it also
offers a toggle (for the grouping itself, or for the marketplace override
that shows the Settings switch) is undecided; see the open questions.

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
2. No UI. No daemon unless research shows the setting gets reset and has
   to be re-applied.
3. Changes only what grouping needs: today that's two `dcc` rows in
   `/var/local/appreg.db` (`docs/research/series-grouping-gate.md`). Never
   writes `cc.db` or `ksdk.asset.db`.
4. Uninstalling puts things back as they were.
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

### Phase 2: Decide the package's behaviour

**Proposed (2026-10-20), awaiting review:** ADR 0001 (install turns
grouping on and shows the switch, writing UserDefaults too), 0002
(uninstall restores only rows still holding our values), 0003 (no daemon;
`kpm launch` re-applies, using the UserDefaults comparison) and 0004 (a
Rust binary with SQLite built in). The questions they answer:

- What installing does: turn grouping on **and** show the Settings switch,
  or only show the switch and leave the choice to the user in Settings.
- Whether there's a toggle besides Settings, and in what form with no UI
  (a `kpm launch` script, a Library scriptlet, a KUAL entry).
- Whether and how to re-apply after a reset (Phase 1, item 1).
- Confirmation: Wireless Sync's plan wanted its script to require the
  device's UUID as an argument. For a package the user installs on
  purpose, the install may be confirmation enough.
- The implementation: the shell script, or a Rust binary that doesn't
  depend on the `sqlite3` or `lipc-*` tools.

### Phase 3: Build and release

- Hooks as decided, safe to run again, with tests.
- The README: what it does, the risk, how to undo it by hand.
- An install, upgrade (from a temporary repository) and uninstall test on
  the reference device.
- First release, 0.1.0 (ask before publishing).

## Open questions

- See Phase 1 and Phase 2.

## Later

- Nothing yet.
