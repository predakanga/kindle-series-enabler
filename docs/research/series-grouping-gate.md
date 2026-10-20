# The series grouping gate (FW 5.20.1)

How the Kindle decides whether to group series, and the smallest change
that turns grouping on where Amazon doesn't offer it. Researched on the
reference device (Paperwhite 12th Gen, FW 5.20.1, AU marketplace) from
2026-10-16 to 2026-10-17, for
[Kindle Wireless Sync](https://github.com/predakanga/kindle-wireless-sync)'s
native series (its ADR 0016), and moved here on 2026-10-20.

Everything here was read from Amazon's binaries on the device
(disassembled) or observed on it. It's Amazon's code, so nothing is copied
here: only names, strings and behaviour.

## Background: grouping is gated

On the reference device even Amazon series (for example a 4-book one) were
**not grouped**, and there was no switch for it. From `KPPMainApp`'s strings:

- The switch is meant to be a **Quick Settings** tile (`group_series_library`,
  `QuickSettings::GroupSeriesWidgetController`), and it isn't offered on this
  device.
- Availability depends on remote config (`series.grouping.supported`,
  `series.grouping.supported.marketplaces`) and a weblab,
  `BOOKS_EINK_SAGA_LSG_431259` (`isSeriesGroupingWeblabEnabled`). This
  device is on the **AU** marketplace.
- The user's choice was moved from dynamic config (`series.grouping.enabled`,
  `IGNITE_SERIES_GROUPING_ENABLED_KEY`) into "UserDefaults" (marker file
  `/var/local/SERIES_GROUPING_MIGRATION_DONE`, 2026-10-13). On this device
  `/var/local/user_defaults.db` (table `user_defaults`, columns
  `ID|Key|Value`) had no series-grouping key.
- Weblab state is in `/var/local/weblab/` (`allocations/*.alloc`, a large
  read-only key/value DB, plus `overrides/` and `local_overrides/` folders,
  and `/var/local/weblab_overrides`). Their formats haven't been looked at.

## The gate logic

Read from `KPPMainApp`'s `KPPSettings::ServiceProvider::GroupSeriesServiceProvider`
(`/app/bin/KPPMainApp`, a stripped ARM Thumb-2 ELF with a dynamic symbol
table):

- Every check first asks `IgniteUtils::IsLibraryRearchitectureEnabled()`
  (the Ignite "library rearch" module). If that's true, the switch is
  always offered and its value lives in **UserDefaults**. Otherwise:
  - `shouldWidgetBeDisplayed` = dynconfig `series.grouping.supported ==
    "true"` **and** `series.grouping.supported.marketplaces` contains
    dynconfig `marketplace.obfuscated.id` (a substring match).
  - `getGroupBySeriesConfig` = dynconfig `series.grouping.enabled == "true"`.
  - `setGroupBySeriesConfig` writes both dynconfig and UserDefaults.
  - `isSeriesGroupingWeblabEnabled` (`BOOKS_EINK_SAGA_LSG_431259 == T1`)
    has no direct callers, so the weblab doesn't seem to gate it.
- The constructor migrates the dynconfig value to UserDefaults once (the
  marker file above).

## Where the values live

- **Dynconfig** (`/usr/lib/libdynconf.so`) lives in `/var/local/appreg.db`,
  table `properties (handlerId, name, value)`, primary key `(handlerId,
  name) ON CONFLICT REPLACE`. Handler `dcd` holds the defaults and `dcc`
  the current (server-written) values, which win. On the reference device:
  - `dcd`: `series.grouping.supported = true`, `series.grouping.enabled =
    true`, `series.grouping.supported.marketplaces =
    A1VC38T7YXB528,ATVPDKIKX0DER,A1F83G8C2ARO7P,A1PA6795UKMFR9` (JP, US,
    UK, DE).
  - `dcc`: `marketplace.obfuscated.id = A39IBJ37TRP1C6` (AU) and
    `series.grouping.enabled = false`. That row sat among `RAMP` rows, which
    first suggested a server config fetch. It turned out to be a batch
    written at boot (see "What refreshes `dcc`"), so where the `false` came
    from is unknown.
- **UserDefaults** (`/app/lib/libEReaderAppSDKShared.so`,
  `esdk::UserDefaultsImpl`) stores rows in `/var/local/user_defaults.db`,
  table `user_defaults`, with `Key = <namespace>.<key>`. The grouping flag
  is `IGNITE_LIBRARY_NAMESPACE.IGNITE_SERIES_GROUPING_ENABLED_KEY`. Existing
  rows use `ID = 1`, and bools are stored as `1`/`0`.

## First attempt (2026-10-16)

By hand, with sqlite3 (backup of `appreg.db` from before at
`/mnt/us/kws-appreg.db.bak` on the reference device):

1. Inserted `dcc | series.grouping.supported.marketplaces |` the `dcd` list
   plus `A39IBJ37TRP1C6`. Quick Settings still showed no tile, even after
   `initctl restart kppmainapp` (that restart also popped an "Application
   Error" dialog from appmgrd, which was harmless).
2. Set `dcc series.grouping.enabled` to `true`, and inserted
   `1 | IGNITE_LIBRARY_NAMESPACE.IGNITE_SERIES_GROUPING_ENABLED_KEY | 1`
   into UserDefaults. The developer then rebooted the device before
   looking.

Result: **Amazon series were grouped**, and a "Group series" switch
appeared in the full **Settings** page, but still not in Quick Settings.

## Which change does it (2026-10-17)

Each setting was switched on its own, with the Settings switch left alone.
The Library was redrawn by going to Home and back, with no restart. Every
change took effect **live**, so the reboot in the first attempt wasn't
needed.

| Marketplace override | `dcc` enabled | UserDefaults | Series grouped | Settings switch |
|---|---|---|---|---|
| – | `false` | – | no | hidden |
| – | `false` | `1` | no | hidden |
| – | `true` | – | **yes** | hidden |
| added | `true` | – | **yes** | shown, on |
| added | `false` | – | no | shown, off |

- **`dcc series.grouping.enabled` is the on/off switch.** Setting it to
  `true` groups series even while the Settings switch is hidden.
- **The marketplace override only shows the switch** ("Group series",
  under Settings > Home and Library > Library tab).
- **The UserDefaults flag has no effect** on this device, which fits
  `IsLibraryRearchitectureEnabled()` being false here.

`package/series-grouping.sh` sets exactly these two values (`just grouping
[status|enable|disable]`). It was tested both ways on the device.

The Settings switch writes the UserDefaults row as well as the `dcc` value
(from the disassembly, and seen on the device on 2026-10-20), which is
harmless.

Undo (back to how it was found on the reference device):

```sh
sqlite3 /var/local/appreg.db "update properties set value='false' where handlerId='dcc' and name='series.grouping.enabled';"
sqlite3 /var/local/appreg.db "delete from properties where handlerId='dcc' and name='series.grouping.supported.marketplaces';"
```

## What refreshes `dcc` (2026-10-20)

Read from `/usr/bin/dynconfig` (30 KB, disassembled) and its upstart job,
and checked against `appreg.db` on the reference device.

**Who writes it.** The `dynconfig` daemon (`/etc/upstart/dynconfig.conf`,
started with `lab126`, respawned if it dies). It does nothing on its own
schedule. Amazon's server queues a **ToDo item** for the device. The ToDo
service (`com.lab126.todo`) fetches it when it checks in, and passes it to
dynconfig through the LIPC property `handleToDoItem`, as JSON with an `id`,
a `topic` and a `payload`. The payload's `action` is `DOWNLOAD`, with a
`url`. dynconfig downloads that file through `com.lab126.transfer` into a
temporary `tmp-scfg-down-XXXXXX` file. (The `dequeue` and `obliterate`
strings are commands it sends to the transfer service, not registry
operations.) So a refresh happens only when Amazon queues one, and the
device can't be made to ask for one.

**How it applies a file: a merge.** The file is `key=value` lines:
1. Copy every `dcc` row to a backup handler, `dcb`.
2. For each line, set that `dcc` property (`RegSetPropertyValue`), and
   collect the key names.
3. If any step fails, copy `dcb` back over `dcc`. Either way, delete
   `dcb` afterwards.
4. Report the ToDo status to the server, and send the update event (below).

Nothing in this path deletes `dcc` rows, so **keys missing from the file
are left as they are**. The marketplace override survives a refresh
unless the server's file itself sets
`series.grouping.supported.marketplaces`. `series.grouping.enabled`
survives unless the file sets that key. Some keys are treated as "known
SCFG" and skip a ToDo verification step: `lang.store`,
`marketplace.obfuscated.id`, `url.myk`, `url.mysn`, `url.store`,
`url.website` and `wan.proxy.non_proxy_hosts.http`/`https`. Those form the
marketplace's base config.

**The notification.** After applying a file, dynconfig sends the LIPC
event **`com.lab126.dynconfig` `dynamicConfigUpdated`**. Its parameters
are an int (the number of keys set), then one string per key name, cut off
when the event reaches LIPC's size limit. KPPMainApp subscribes to it. So
does the shell tool: `lipc-wait-event -s 3 com.lab126.dynconfig
dynamicConfigUpdated` subscribes and times out cleanly (exit 255). A
package could listen for it, check whether either grouping key was in the
list (or just check the values), and put them back.

**Telling a push from the user (2026-10-20).** With the listener running
(below), the developer turned "Group series" off and on again in Settings:
- **No event arrived.** The listener does receive events: one sent with
  `lipc-send-event` from a test source, to a listener started the same
  way, came through with its parameters. So the Settings switch writes
  `dcc` in-process, through `libdynconf`, and sends no
  `dynamicConfigUpdated`.
- `dcc series.grouping.enabled` got a new rowid (7986), and the
  UserDefaults row `IGNITE_LIBRARY_NAMESPACE.IGNITE_SERIES_GROUPING_ENABLED_KEY`
  came back (`1`). It had been deleted during the 2026-10-16 isolation
  tests, and nothing had used the switch since.

So two independent signals tell them apart:

| Signal | Settings switch | Server push |
|---|---|---|
| `dynamicConfigUpdated` event | none | sent, naming the keys |
| UserDefaults key | written to match | untouched |

The second needs no event, so it also covers a key list cut off at the
size limit. If `dcc` says `false` while UserDefaults says `1`, a push
changed it. It only works once the UserDefaults key exists: a user who
never touched the switch has none, and deregistration or Ignite's cleanup
removes it. A package that sets `enabled` could write the key as well, as
the switch does.

**No refresh seen so far.** Because `properties` replaces on conflict,
every write gives the row a new rowid, so rowids show the order of writes:
- The marketplace base config (`lang.store`, `url.*`,
  `marketplace.obfuscated.id`, at rowids 4807 to 5199) has the same rowids
  as in the backup from before any change. It hasn't been pushed again
  since 2026-10-16.
- The override, at 7611, is where `enable` wrote it on 2026-10-16, and
  it's untouched.
- The newest `dcc` row (7974) is `gdpr.dpc.setting.version`.
- The device's logs only go back to the morning of 2026-10-20, and show no
  dynconfig downloads.

A listener has been logging every event from `com.lab126.dynconfig` and
`com.lab126.todo`, with timestamps, to `/mnt/us/kse-config-events.log`
since 2026-10-20 14:52. It stops at the next reboot.

**An unexplained write at boot.** The last write to
`series.grouping.enabled` (rowid 7665) falls inside a batch written at boot
on 2026-10-16 (about 20:41). That batch also rewrote two `RAMP.features.*`
`dcc` rows and the `reg_apps` app registrations. The backup shows the same
batch, the same shape and the same gaps, from an earlier boot, then with
`false`. So something re-writes `enabled` at boot with its current value.
This hasn't been tracked down. `enabled` is the user's choice (the Settings
switch writes it), so this only matters if that boot-time write could ever
change the value. The `RAMP` keys aren't plain strings in any binary under
`/usr`, `/app/bin` or `/app/lib`.

**Other things that could undo it:**
- Firmware updates ship the `dcd` defaults
  (`/opt/var/local/reg/ServerConfig.db` holds the same three `dcd` grouping
  rows). The override lives in `dcc`, so an update leaves it alone. But it's
  a copy of the `dcd` list as it was at install, so it would hide any
  marketplaces a later firmware adds to `dcd`. That only matters for
  whether *this* device's marketplace is listed.
- Both values survived one reboot (2026-10-16).
- UserDefaults are wiped on deregistration, and Ignite's "Cleaning user
  preferences for library namespace" routine clears the UserDefaults key.
  Neither matters here, since the key has no effect on this device. On a
  device with the library re-architecture, it would.

## Quick Settings (not investigated)

The Quick Settings tile still doesn't show with the override in place, so
the tile list has another filter. Where to start in `KPPMainApp`:
`QuickSettingsWidgetProvider::getIconPanelWidgetList` (0x1db4024), its
constructor (0x1db4454, which references `group_series_library`), and
`QuickSettingsWidgetFactory::getWidgetController`. The Hermes JS bundle
(`/app/KPPMainApp/js/KPPMainApp.js.hbc`) may hold the widget list too.

## Device notes

- Navigation with Button Mapper's `tap.sh` (percent of the screen): Library
  tab `76 96`, Home tab `24 96`. Quick Settings: swipe or tap `50 1`, then
  All Settings `67 31`. In Settings: Home and Library `40 58`, then the
  Library tab row `40 17`; back arrow `5 9`. The appmgrd "Application
  Error" dialog's Close button: `75 57`.
- `kppmainapp` (`/app/bin/KPPMainApp`, KPPMainAppV2) is the React Native
  UI. Restarting it isn't needed for these settings.
