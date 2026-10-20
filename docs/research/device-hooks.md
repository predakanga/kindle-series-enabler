# Search-bar commands, on-screen output and tools (FW 5.20.1)

How a package can add a `;command` to the search bar, write to the screen,
and which tools it can rely on. Seen on the reference device (PW 12th Gen,
5.20.1, KMC jailbreak with KPM 0.2.2) on 2026-10-20.

## Search-bar commands

KPPMainApp's native code (the `KPPChromeSearchModule` area) reads two JSON
files, each one flat object mapping a command to a shell command line:
- `/usr/share/app/kpp_sys_cmds.json`: system commands (`;711`, `;un`,
  `;uns`, `;dm`, `;st`, ...).
- `/app/kpp_app_cmds.json`: app commands (`;framer`, `;debugon`,
  `;syncweblab`, ...).

Both are on the root filesystem (read-only ext3, so writing needs
`mntroot rw`), and a firmware update replaces them.

The KMC jailbreak adds `;kpm`, `;log` and `;kmclog` to **both** files. Its
`/var/local/kmc/bin/kmc_system_patcher` parses each file as JSON, adds its
keys and writes it back. KMC's `system_patches/patch_system.sh` runs the
patcher (it's run again by the hotfix after an update). So a third
party's key survives KMC re-patching, as long as the file stays valid
JSON. **A broken file would break KMC's patcher, and probably every search
command.**

The values include arguments (`/app/tools/switchChrome.sh kpp 2`). KMC's
`/var/local/kmc/sbin/kpm.sh`:
- `sleep 1`s first, to let the keyboard close;
- runs `kpm --fbink $*` through KMC's `/var/local/kmc/bin/su -c`;
- runs `/usr/bin/xrefresh -d :0.0` two seconds later, to repaint over the
  text.

**Tested (2026-10-20)** by adding `";series": "/bin/sh
/mnt/us/kse-cmdtest.sh"` to both files (backups in
`/mnt/us/kse-kpp_*_cmds.json.bak`):
- **KPPMainApp reads the files only at startup.** Typing `;series` in the
  running app just searched the store. After `initctl restart kppmainapp`
  it ran. So a new command needs a framework restart or a reboot. (Restart
  `framework` rather than `kppmainapp`: the latter is reported as an
  "Application Error".)
- **Text after the command is passed as arguments:** `;series foo` ran
  the script with `$1 = foo`. Plain `;series` passed none.
- **It runs as root** (uid 0, gid `video`, groups `javausers`), as a child
  of `/app/bin/KPPMainApp`, in `/`, with a minimal environment (`PATH`,
  `LANG`, `LC_ALL`, `KINDLE_TZ`). So KMC's `su` isn't needed.
- **Nothing repaints afterwards.** FBInk's line stayed on the screen until
  something else redrew it. The first run's line was covered by the store
  search results, the second's stayed. Hence `kpm.sh`'s `xrefresh`.

## Writing to the screen

- **FBInk** is at `/var/local/kmc/bin/fbink` (plus `libfbink.so` in
  `/var/local/kmc/lib/`). KMC ships it, and KPM is part of KMC, so it's
  there wherever the package can be installed. KMC's `dispatch.sh` uses
  `fbink -x 1 -y 0 -S 3 "text"`. It isn't on `PATH`.
- **`eips`** (`/usr/sbin/eips`) is Amazon's own, on every firmware. KMC's
  `jb.sh` uses `eips_v2 text "..." --top N` if it exists and `eips 0 ROW
  "..."` otherwise. It's the fallback if FBInk is missing.

## Tools on the device

- **`sqlite3`** (`/usr/bin/sqlite3`, 3.47.0, 2024-10-21). KMC depends on it:
  `patch_system.sh` registers its shell integration by piping SQL files to
  `sqlite3 /var/local/appreg.db`. So a device with a working KMC install
  has it.
- It has the JSON functions and the CLI's `readfile()`. In `:memory:`,
  `json_set(readfile(F), '$.";series"', V)` adds a key at the end, keeping
  the order of the others. `json_remove` of the same key gives back an
  object equal to `json(readfile(F))`. `json_pretty` (3.46+) is there too.
  So on this device the shell could edit the command files as JSON,
  without `jq` (which isn't on the device). **But don't rely on it:** the
  developer's earlier testing found older firmware whose `sqlite3` was
  built without the JSON functions (2026-10-20).
- `lipc-get-prop` and the other `lipc-*` tools are in `/usr/bin`.
- No `python3` and no `jq`.
- Permissions: `/var/local/appreg.db` and `/var/local/user_defaults.db` are
  `framework:javauser`, mode `775`.

## Boot jobs

- KMC's `/etc/upstart/kmc.conf` is `start on framework_ready`, and only
  runs `/mnt/us/emergency.sh` if it exists. KMC says update persistence
  "can't be done via upstart", so it registers its hotfix launcher in
  `appreg.db` instead. Upstart jobs in `/etc/upstart` are lost on a
  firmware update.
- Kindle Wireless Sync's `kindle-wireless-sync-autostart.conf` is a
  `task` job, `start on framework_ready`, `console none`, that `exec`s a
  script on `/var/local` if it's there. Its comment says it never blocks
  boot.
