# 0002: What uninstalling restores

- Status: Proposed
- Date: 2026-10-20

## Context
The dev tool's `disable` assumes the reference device's starting state:
`dcc series.grouping.enabled = false`, no override and no UserDefaults
key. Other devices may start differently. For example, a device might
have no `dcc` row at all, so the `dcd` default (`true`) applies.

Between install and uninstall, three things can change the values:
- the user, through the Settings switch (writes `dcc` and UserDefaults);
- a server config push, which merges keys into `dcc` (research note,
  "What refreshes `dcc`");
- a firmware update, which may change the `dcd` defaults.

KPM runs `uninstall.sh upgrade` before an upgrade, then `install.sh`, and
it deletes the package directory each time (`kpm-packaging.md`).

## Decision
**Record what was there at the first install, and on removal put back
only what's still ours.**
- `install.sh` records the starting value of each row it's about to
  change. That's the row's value, or the fact that there was no row. The
  record goes in `/var/local/kindle-series-enabler/original`, outside the
  package directory so upgrades keep it. If a record already exists
  (upgrade or reinstall), it's kept, not overwritten.
- `uninstall.sh` (removal) restores each row **only if it still holds
  the value the package wrote**. Restoring means putting back the
  recorded value, or deleting the row if there was none. A row that
  changed since is left alone, because someone else owns it now:
  - `enabled` or UserDefaults changed: the user used the Settings
    switch, or a push set it.
  - The override changed: a push set the marketplace list.
  Then it deletes the record and the directory.
- `uninstall.sh upgrade` changes nothing.

**Exception:** if the user turned grouping off with the Settings switch,
`enabled` is already `false`, which isn't what the package wrote. So
uninstall leaves it `false`, even if it started with no row at all.
That's the user's own last choice, so it's the right result.

## Alternatives considered
- **Always reset to the reference device's state** (the dev tool's
  `disable`): simple, but wrong for a device that started with a
  different value. It would also turn grouping off for a user whose
  marketplace had it all along.
- **Always restore the recorded values,** whatever happened since: undoes
  the user's own Settings choices and any server push since the install.
- **Keep the settings after removal:** then removal wouldn't really
  remove anything. Users who want grouping without the package can turn
  it on again by hand.

## Consequences
- A state file in `/var/local/kindle-series-enabler/`, which uninstall
  removes.
- The tool needs a `restore` command, separate from the dev tool's
  `disable`. The dev tool's semantics stay available for development.
- If the record is lost (for example, `/var/local` wiped by a factory
  reset, which also resets the settings), removal only deletes the
  override if it's still ours, and leaves `enabled` alone.
