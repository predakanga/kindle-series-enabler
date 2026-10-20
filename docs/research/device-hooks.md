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
`/var/local/kmc/sbin/kpm.sh` passes on `$*`, so text typed after the
command reaches the script. `kpm.sh` also:
- `sleep 1`s first, to let the keyboard close;
- runs the real work as root through KMC's `/var/local/kmc/bin/su -c`, so
  the command isn't run as root by KPPMainApp (not checked which user it
  is);
- runs `kpm --fbink` to write progress on the screen;
- runs `/usr/bin/xrefresh -d :0.0` afterwards, to repaint over the text.

**Open:** whether KPPMainApp reads the files once at startup or on every
search. If only at startup, a new command works only after a reboot (or a
`kppmainapp` restart). Testing that needs a root filesystem write.

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
  So the shell can edit the command files as JSON, without `jq` (which
  isn't on the device).
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
