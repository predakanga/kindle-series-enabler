# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

A UI-less KPM package for jailbroken Kindles that turns on the built-in
Library's series grouping in marketplaces where Amazon doesn't offer it.
Split out of [Kindle Wireless Sync](https://github.com/predakanga/kindle-wireless-sync)
(`../kindle-wireless-sync`) on 2026-10-20, and modelled on its tooling.

The reference device is a Kindle Paperwhite 12th Gen on FW 5.20.1, in the
AU marketplace. Other devices and firmware are best-effort.

## Current status

**Start with `docs/PLAN.md`**: phases, open questions and what comes next.
Research is in `docs/research/` (`series-grouping-gate.md` for the
setting, `kpm-packaging.md` for KPM, `device-hooks.md` for search-bar
commands, on-screen output and the tools on the device). Decisions go in
`docs/decisions/`.

## Commands

```bash
just test                 # tests/series-grouping.sh against a fixture appreg.db (needs sqlite3)
just check                # shellcheck every script
just grouping [status|enable|disable]   # run package/series-grouping.sh on the Kindle
just package              # build target/package/kindle-series-enabler.kpkg
just kpm-install | kpm-uninstall   # install/remove the package with the device's own KPM
just screensaver-off | screensaver-on   # keep the Kindle awake for long tests (Wi-Fi drops with it)
just release X.Y.Z        # prepare a release: bump VERSION, date the changelog, commit (no push)
just publish-release [-y] # tag the prepared version and push main + tag (publishes; ask first)
```

**Releases:** as in Wireless Sync (its ADR 0014). `just release X.Y.Z`
(`tools/release.sh`) writes `VERSION`, moves the `Unreleased` entries in
`CHANGELOG.md` under `## [X.Y.Z] - today`, updates the compare links and
commits. It refuses a dirty tree, a branch other than `main`, a version
that isn't newer, or an empty `Unreleased`. Show the developer the commit,
then (after asking) `just publish-release -y` tags `vX.Y.Z` and pushes.
`.github/workflows/release.yml` checks, tests and packages, publishes the
GitHub release with the changelog section as notes, and rebuilds the KPM
index on GitHub Pages (`https://predakanga.github.io/kindle-series-enabler/repo.json`).

**CI:** `.github/workflows/ci.yml` runs `just check`, `just test` and
`just package` on pushes to `main` and on pull requests.

## Layout

- `package/`: what ships. `series-grouping.sh` does the work (`APPREG`
  overrides the database path); `install.sh` and `uninstall.sh` are the KPM
  hooks (drafts until PLAN Phase 2); `manifest.json.in` takes its version
  from `VERSION`; `pages/index.html` is the Pages site template.
- `tools/`: packaging, release, changelog notes and the KPM index.
- `tests/`: shell tests against a fixture `appreg.db` built from the
  reference device's schema and values.

## Working process

- **Plan, then research, then build.** Keep `docs/PLAN.md` current: tick
  off phases and move answered questions into decisions.
- **Record design decisions** as ADRs in `docs/decisions/` (template and
  index in its README). Write new ones as **Proposed**; the developer
  accepts them.
- Present options with pros, cons and a recommendation before changing an
  agreed design.
- Credit prior art precisely, naming the project an idea came from.
- **Keep `CHANGELOG.md` current:** user-facing changes under
  `## [Unreleased]`.
- **Licence headers:** every new script starts with an
  `SPDX-License-Identifier: GPL-3.0-or-later` line, after the shebang (`# `)
  or `<!DOCTYPE>` (`<!-- -->`). JSON is exempt.
- **Commit as you go**, in small commits.
- **Don't push or tag without asking.** Only propose a release for a real,
  user-facing change.

## Device access

- The reference Kindle is SSH host `kindle`, over USB networking (`usb0`,
  192.168.15.244).
- `/var/local/appreg.db` is the only file this package writes. Back it up
  before experiments (`/mnt/us/kws-appreg.db.bak` holds the state from
  before any change). Never write `/var/local/cc.db` or
  `/var/local/ksdk.asset.db`.
- This is Amazon's code: describe names, strings and behaviour from
  disassembly, but don't copy code into the repo.
- To check the UI from SSH, run `/usr/sbin/screenshot` (PNG in
  `/mnt/us/screenshots/`; delete it after fetching), and tap with Button
  Mapper's `/mnt/us/kindle-button-mapper/scripts/tap.sh X% Y%`. Coordinates
  for the Library, Settings and Quick Settings are in the research note.
  Do read-only UI checks yourself; ask the developer only for what needs
  their eyes or hands.
- Ask the developer before writes to the root filesystem (`mntroot`),
  reboots, or anything else hard to undo.
- When you ask the developer to check something on the device, wait for
  their answer, and don't undo what they're checking.
- The device has no `python3`. KPM is `/var/local/kmc/bin/kpm` (0.2.2);
  see `docs/research/kpm-packaging.md`, including why upgrades must be
  tested from a temporary `file://` `repo.json`.

## Gotchas

- **zsh:** an argument starting with `=` (for example `echo =====`) aborts
  the whole command line.
- **`pkill -f PATTERN` over SSH matches its own `sh -c` command line** and
  kills the session. Use a self-excluding pattern (`pgrep -f "[p]attern"`).
- `initctl restart kppmainapp` pops an "Application Error" dialog (Close
  at `tap.sh 75 57`), and isn't needed: the settings apply when the
  Library redraws (go to Home and back).
- KPM reports network failures only as a number: `(4)` is `KPM_CURL_ERROR`,
  which usually means Wi-Fi is down (screensaver).
