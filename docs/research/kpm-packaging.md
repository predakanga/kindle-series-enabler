# KPM packaging and releases

What this package relies on from KPM and from the release setup. Taken
from [Kindle Wireless Sync](https://github.com/predakanga/kindle-wireless-sync)'s
ADR 0012 (package lifecycle) and ADR 0014 (releases), which have the full
evidence. The reference device runs KPM 0.2.2 from the Kindle Modding Club
(github.com/KindleModding/KPM, commits `07495e0` to `94608b3^`).
github.com/gingrspacecadet/kpm is a different tool with the same name.

## How KPM runs a package

- **Install:** KPM deletes `/mnt/us/kmc/kpm/packages/<id>/`, extracts the
  `.kpkg` there, and runs `sh install.sh` from that directory. If the script
  exits non-zero, KPM deletes the directory and doesn't record the package,
  but nothing undoes what the script already did. So `install.sh` must be
  safe to run again.
- **Reinstall or upgrade:** KPM first runs the old package's
  `sh uninstall.sh upgrade` (a failure there is only a warning), then
  installs as above.
- **Uninstall:** `sh uninstall.sh`. If it exits non-zero, KPM keeps the
  files and its record, so the user can retry.
- **Launch:** `kpm launch <id>` runs the package's `launch.sh`, if any.
- After every install or uninstall, KPM starts a full library rescan.
- **Hook output is shown HTML-escaped** (KPM 0.2.2, seen 2026-10-20): a
  `>` printed by `install.sh` appeared as `&gt;`. Avoid `<`, `>` and `&`
  in hook messages.
- **Hooks inherit the umask** of the shell that ran `kpm`, which can be
  `077` over some SSH servers. Set `umask 022` in hooks that create files
  other users need to read.
- **Coming after 0.2.2 (master, unreleased):** `install.sh` gets an
  `upgrade` argument, and a failed `install.sh` makes KPM run
  `uninstall.sh` without `upgrade`.

## Package format

A gzip tar with every name at the archive root (no `./` prefix):
`manifest.json` (`manifest_version` 2, `id`, `name`, `author`,
`description`, `version` as `[X, Y, Z]`, `dependencies`,
`supported_platforms`), the hooks, and the package's files. Root-owned
entries and no macOS metadata. `tools/package.sh` handles both bsdtar
(macOS) and GNU tar (CI).

## Local files can't upgrade

`kpm install file:///x.kpkg` fails with "Could not resolve dependency
graph" when the package is already installed, and `kpm upgrade` skips
packages installed from a file. Upgrades from a **repository** index work.
So users install from the KPM index (the shared one built by
`predakanga/kindle-apps`, ADR 0005), and an upgrade test
serves the package from a temporary `file://` `repo.json` (`kpm add-repo`,
`kpm update`, `kpm install`), removing the repo afterwards.

A local `file://` install also skips the `supported_platforms` check,
which only applies to indexed artifacts.

## Index format

`repo.json` (`src/index.c`): `manifest_version` 1, `id`, `name`,
`description`, and `packages.<id>` with `name`, `author`, `description`
and `artifacts[]` (`url`, `version`, `dependencies`,
`supported_platforms`). KPM lists artifacts newest first and installs the
first, so the index lists every release. KPM follows redirects, so GitHub
Release assets work as artifact URLs. One index can list any number of
packages; KPM stores them per repository and ID.
