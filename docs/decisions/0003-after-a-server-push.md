# 0003: After a server config push

- Status: Proposed
- Date: 2026-10-20

## Context
From the research note ("What refreshes `dcc`", "Telling a push from the
user"):
- Only the `dynconfig` daemon applies server config. It does so when
  Amazon queues a config ToDo for the device, and the device can't
  request one.
- A push **merges** keys into `dcc`. Our rows change only if the pushed
  file sets those exact keys.
- After each push, dynconfig sends the LIPC event `com.lab126.dynconfig`
  `dynamicConfigUpdated`, naming the keys it set. The Settings switch
  doesn't send it, and it writes UserDefaults too, which a push doesn't.
  So a push can be told apart from the user.
- No push has been seen on the reference device since 2026-10-16. A
  listener has been logging these events since 2026-10-20.
- Both values survived two boots. A write at boot re-writes `enabled`
  with its current value, from something that runs before KPPMainApp
  (not traced; restarting KPPMainApp doesn't do it).

Reacting to the event needs a process running all the time, which means
an upstart job and so a root filesystem write. Wireless Sync does this
(its ADRs 0004 and 0012), but it's a lot of machinery for something not
yet seen to happen.

## Decision
**No daemon, for now.** The package doesn't watch for pushes. Instead:
- `kpm launch kindle-series-enabler` (the package's `launch.sh`)
  re-applies the install changes (ADR 0001), and keeps the original
  record (ADR 0002). After a push, a user who notices runs that, or
  reinstalls. It always puts the override back. It touches `enabled`
  only if the evidence says a push changed it: `dcc` disagrees with the
  UserDefaults key, which the Settings switch keeps in step. Then it sets
  `dcc` back to the UserDefaults value, the user's last choice, which may
  be off. If they agree, the user's choice stands.
- The README tells users what to do if grouping turns itself off.
- Keep the event listener running on the reference device, and look at
  the log from time to time. **Revisit this decision if a push is ever
  seen setting our keys.** The design for the listener is already known:
  subscribe to `dynamicConfigUpdated`, and on an event that names our keys
  (or is cut off at the size limit), apply the same re-apply logic.

## Alternatives considered
- **A listener daemon** (an upstart job running `lipc-wait-event` or our
  binary). Handles pushes automatically, but needs a root filesystem
  write on install and uninstall, an upstart job to maintain, and an
  always-running process for an event never yet seen to matter.
- **Re-apply at every boot** (an upstart job that runs once): catches a
  push at the next restart, not when it happens. It also needs the
  root filesystem write. And a push that happened was the server's
  choice, which only the user should override.
- **Nothing at all** (the user reinstalls): works, but reinstalling to
  re-apply is clumsy, and re-installing from a local file doesn't work
  in KPM 0.2.2.

## Consequences
- The binary gets a re-apply command that `launch.sh` runs, using the
  `dcc`/UserDefaults comparison. If the UserDefaults key is missing (wiped
  on deregistration, or by Ignite's cleanup), it can't tell, so it treats
  the package's own value (`true`) as the choice and writes both.
- If pushes turn out to reset the setting, a later version adds the
  listener. This ADR would then be superseded.
- The package needs no `mntroot` and no upstart jobs.
