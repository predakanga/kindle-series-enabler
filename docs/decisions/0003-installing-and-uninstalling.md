# 0003: Installing and uninstalling

- Status: Proposed
- Date: 2026-10-20

## Context
- The boot job and `;series` (ADR 0002) run as root. Kindle Wireless Sync
  (its ADR 0004 and 0012) keeps everything root runs in `/var/local`,
  because `/mnt/us` is reachable over USB, and because KPM deletes the
  package directory on every upgrade.
- The upstart job and the two search-command files are on the read-only
  root filesystem (`mntroot rw` to change them), and a firmware update
  replaces them. KMC's patcher re-adds its own commands to both files
  after an update, but not anyone else's (`device-hooks.md`).
- KPPMainApp reads the command files only at startup.
- KPM's hooks (`kpm-packaging.md`): install runs `install.sh`, which must be
  safe to run again. Upgrade runs `uninstall.sh upgrade`, then
  `install.sh`. Uninstall runs `uninstall.sh`.
- Between install and uninstall, a server push could set the override,
  and the boot run could then write it again.

## Decision
**Install** (`install.sh`):
1. Copy the binary, `boot.sh` and `series.sh` into
   `/var/local/kindle-series-enabler/`, each to a temporary name and then
   `mv`, as Wireless Sync does.
2. `mntroot rw`. Install the upstart job, and add `";series": "/bin/sh
   /var/local/kindle-series-enabler/series.sh"` to both command files
   (`register`, ADR 0004). Then `mntroot ro`, also on failure. If the root
   filesystem was already writable, leave it writable.
3. Run `apply`, so the switch appears without a reboot.
4. Say that `;series` works after the next restart.

**Uninstall** (`uninstall.sh`, not `upgrade`):
1. `mntroot rw`. Remove `;series` from both files if its value is ours
   (`unregister`), and remove the job. `mntroot ro`.
2. Put the override back (`restore`). Each time `apply` writes the
   override, it records the value it replaced (or that there was none) and
   the value it wrote, in `/var/local/kindle-series-enabler/state`. If the
   row still holds what it wrote, `restore` puts back the recorded value,
   or deletes the row. Otherwise someone else (a push) owns it, and it's
   left alone.
3. Delete `/var/local/kindle-series-enabler/`.

**`enabled` is left alone.** If grouping was on, it stays on with no
switch to turn it off. Users who want it off can reinstall, switch it off
and uninstall again (the developer, 2026-10-20).

**`uninstall.sh upgrade` does nothing.** The install that follows
replaces the files and keeps the state file.

## Alternatives considered
- **Run from the KPM package directory:** root would run files anyone
  with USB access can replace, and KPM deletes that directory during
  upgrades (Wireless Sync ADR 0004 and 0012).
- **Restart the framework at install,** so `;series` works at once: KPM
  can itself be run from `;kpm`, as a child of KPPMainApp, and the restart
  could kill it mid-install. The switch works without a restart anyway.
- **Only one of the command files:** it isn't known which one each
  firmware reads. KMC writes both, so we do too.
- **Set `enabled` back to `false` on uninstall,** where the marketplace
  isn't supported: rejected by the developer.
- **Always delete the override on uninstall:** wrong if a push set it.

## Consequences
- Install and uninstall each need a root filesystem write.
- After a firmware update, the job and `;series` are gone. The override
  stays in `appreg.db` until a push changes it. Reinstalling from KPM puts
  them back. The README says so.
- After uninstall, a running KPPMainApp still knows `;series` until it
  restarts. Typing it then just fails, because the script is gone.
- If `/var/local/kindle-series-enabler/state` is lost, `restore` leaves the
  override alone.
