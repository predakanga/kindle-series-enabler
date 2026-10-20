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

KindleModding's "Debug Commands" page (kindlemodding.org,
`content/kindle-hacking/debug-commands.md` in
github.com/KindleModding/kindlemodding.github.io) lists a third file,
`/usr/share/webkit-1.0/pillow/debug_cmds.json`, from older firmware's
pillow UI, and says it no longer exists. It isn't on the reference device.

The KMC jailbreak adds `;kpm`, `;log` and `;kmclog` to **every one of the
three files that exists**, and skips the rest. Its
`/var/local/kmc/bin/kmc_system_patcher` (source:
github.com/KindleModding/jb.sh, `system_patcher/main.c`) parses each file
with cJSON, deletes and re-adds its three keys, prints the result
(tab-indented) and writes it back in place (`r+`, then `ftruncate`). If a
file doesn't parse, it prints it and moves on. KMC's
`system_patches/patch_system.sh` runs the patcher, and the hotfix runs it
again after an update. So a third party's key survives KMC re-patching, as
long as the file stays valid JSON. **A broken file would break KMC's
patcher, and probably every search command.** (KMC's older hotfix job,
`install_debug_commands_kpp_app.sh` in github.com/KindleModding/Hotfix,
inserted `;log` into `/app/kpp_app_cmds.json` with `sed` instead.)

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
- **A missing target is harmless (tested 2026-10-20).** With
  `";ksesh": "/bin/sh /var/local/kse-missing/series.sh"` (a missing
  script) and `";ksebin": "/var/local/kse-missing/series"` (a missing
  executable), typing either just went back to the Home screen. There was
  no error, no store search, the same KPPMainApp process kept running,
  and it had no child processes left over. A short-lived second
  `KPPMainApp` process (the fork that runs the command) appeared and
  exited. This is what a `;series` left behind after uninstall looks like.
- **Restarting the `framework` job doesn't avoid the "Application Error"
  dialog** that `initctl restart kppmainapp` shows, and KPPMainApp takes
  a while to come back.
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

- **Every framework start makes `/var/local` group-writable**
  (`/etc/upstart/framework.conf`, seen 2026-10-20): for `/var/local`,
  `/var/log`, `/var/lock` and `/var/run` it runs `chgrp -R javausers` and
  `chmod -R g=u`, and sets `g+s` on every directory. Our files went from
  `755 root:root` at install to `775 root:javausers` at the next boot (`644`
  to `664`). It isn't the umask: the install left the right modes, and the
  ctime is the boot. Immutable files (KMC's) escape it, and the same job
  resets `/var/local/root` (Amazon's root-only folder, holding
  `sysreg.db`) to group `root` straight afterwards. The framework user
  (uid 9000) is in `javausers`. It doesn't matter on a KMC jailbreak:
  KMC's `/var/local/kmc/bin/gandalf` (`su`) is setuid root and executable
  by anyone, so a `javausers` process can already become root. Kindle
  Wireless Sync's files get the same treatment.
- **Core dumps go to Amazon's crash handler** (`core_pattern` is
  `|/usr/bin/system_crash_handler %p %s %e %i`). For our aborts (a
  `println!` into a closed pipe, 2026-10-20) it logged `Skipping:
  kindle-series-enabler (no libstackdump.so)` and wrote nothing, but
  aborting is still to be avoided (the binary is built with
  `panic = "abort"`), so output errors are ignored.
- **The device's dropbear aborts on a multi-file `scp -O`** into
  `/var/local/kindle-series-enabler/` (2026-10-20: "Connection closed by
  remote host", and `system_crash_handler … exe=dropbear signal=6`).
  Single-file copies work, so `just deploy` copies one file at a time.

- `/mnt/us` is a FUSE mount (`fsp`) over the userstore. Upstart's
  `filesystems_userstore` mounts it (and emits `mounted_userstore`) early,
  long before `framework_ready`, but nothing makes a `start on
  framework_ready` job wait for it. It can also be full, read-only, or
  stuck if the FUSE daemon is. A boot job shouldn't need it.

- KMC's `/etc/upstart/kmc.conf` is `start on framework_ready`, and only
  runs `/mnt/us/emergency.sh` if it exists. KMC says update persistence
  "can't be done via upstart", so it registers its hotfix launcher in
  `appreg.db` instead. Upstart jobs in `/etc/upstart` are lost on a
  firmware update.
- Kindle Wireless Sync's `kindle-wireless-sync-autostart.conf` is a
  `task` job, `start on framework_ready`, `console none`, that `exec`s a
  script on `/var/local` if it's there. Its comment says it never blocks
  boot.
