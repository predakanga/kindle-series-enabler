# 0002: When it runs, and how it reports

- Status: Accepted (2026-10-20)
- Date: 2026-10-20

## Context
From the research note `series-grouping-gate.md` ("What refreshes `dcc`"):
- Only `dynconfig` applies server config, when Amazon queues a config
  ToDo, and it **merges** keys into `dcc`. The override is lost only if a
  pushed file sets that key. That hasn't been seen (no push since
  2026-10-16).
- Something early in boot re-writes `enabled` with its current value. The
  override has survived every boot.

From `device-hooks.md`:
- Search-bar commands come from JSON files that KPPMainApp reads at
  startup. A command runs as root, as a child of KPPMainApp, with the rest
  of the typed line as arguments, and nothing repaints the screen after
  it.
- FBInk is at `/var/local/kmc/bin/fbink` wherever KMC (and so KPM) is.
  `eips` is on every firmware.
- Wireless Sync's autostart job (`start on framework_ready`, `task`) is a
  one-shot that never blocks boot.

The developer's direction (2026-10-20):
- a one-shot service at boot that applies the override;
- a `;series` search-bar command to run it by hand, logging to the screen;
- the boot run logs to a file;
- `kpm launch` support if it's cheap, logging to stdout;
- no event listener. Users who lose the switch reboot or run `;series`.

## Decision
**Three entry points run `apply`** (ADR 0001), the binary's default
command (ADR 0004). The binary prints plain lines and exits non-zero on
failure. **The entry point decides where the output goes:**

| Entry point | Runs | Output |
|---|---|---|
| Boot: `/etc/upstart/kindle-series-enabler.conf` | `boot.sh`, which runs `apply` | a log file (below) |
| `;series [args]` in the search bar | `series.sh [args]`, which runs the binary with those arguments (none means `apply`) | the screen |
| `kpm launch kindle-series-enabler` | `launch.sh`, which runs `apply` | stdout |

Setting these up is separate and applies nothing by itself. Installing
the job and the search command (`register`) is ADR 0003. `install.sh`
then runs `apply` once, as its own last step, so the switch appears
without a reboot.

- **Boot job:** `start on framework_ready`, `task`, `console none`. It
  `exec`s `/var/local/kindle-series-enabler/boot.sh` if that's there, and
  otherwise exits 0, so a half-removed package can't break boot. This is
  Wireless Sync's autostart job. It runs after the framework, because the
  override is picked up live, so there's nothing to gain by delaying the
  framework, and it's also after the boot-time writer. It's a one-shot:
  nothing stays running.
- **The boot log doesn't depend on `/mnt/us`.** Everything `boot.sh` runs
  is in `/var/local` (ADR 0003). It logs to
  `/mnt/us/kindle-series-enabler.log`, replaced each boot, where users can
  reach it over USB. But nothing orders our job after the userstore mount,
  and the storage can be full, read-only or stuck (`device-hooks.md`). So
  `boot.sh` first checks that it can create the file. If it can't, it logs
  to `/var/local/kindle-series-enabler/boot.log` instead, and failing that
  it discards the output. It checks before redirecting, because in `sh` a
  failed `exec >file` exits the script. **A logging failure never stops
  `apply`.**
- **`;series`**, following KMC's `kpm.sh`: sleep a second so the keyboard
  closes, run the binary with the typed arguments (no arguments means
  `apply`, so `;series status` works too), show each line with FBInk
  (falling back to `eips`), wait a few seconds, then
  `/usr/bin/xrefresh -d :0.0` to repaint. It's already root, so it doesn't
  need KMC's `su`.
- **No listener.** The README says: if the Group series switch
  disappears, reboot or type `;series`.

## Alternatives considered
- **A listener daemon** on `dynconfig`'s `dynamicConfigUpdated` event:
  catches a push at once, but it's an always-running process for an event
  never seen to matter. The developer chose not to (2026-10-20). The
  design is in the research note if it's ever needed.
- **Before the framework** (`start on starting kppmainapp`, blocking it):
  the first draw would already have the switch, but nothing reads it that
  early, and a slow database would delay the UI.
- **The binary works out where to log** (a terminal, an environment
  variable): guesswork. Each entry point already knows.
- **Linking FBInk into the binary:** more build work, for output a shell
  loop can show.
- **Log only in `/var/local`:** always writable, but not reachable over
  USB, which makes it harder for users to send it with a bug report. It's
  the fallback instead. The log only has the values `status` shows.
- **Wait for `mounted_userstore`** (`start on framework_ready and
  mounted_userstore`): if the userstore fails to mount, the event never
  comes, so `apply` would never run, all for the sake of a log. And it
  says nothing about the storage being full or read-only later.

## Consequences
- A push that drops the override is fixed at the next boot, or when the
  user types `;series`. Until then the switch is hidden, but grouping keeps
  whatever `enabled` says.
- The boot run writes nothing when the override is already in place.
- If `/mnt/us` is stuck rather than missing (a hung FUSE daemon), the
  check could block. Since nothing waits for our task, that holds up
  nothing else.
- `;series` only works after KPPMainApp restarts once after install (ADR
  0003).
- Needs a root filesystem write for the job and the command (ADR 0003).
