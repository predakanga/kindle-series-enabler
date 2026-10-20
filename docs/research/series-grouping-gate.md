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
    `series.grouping.enabled = false`. That `dcc` row sat among rows written
    by a server config fetch, so it was probably set by Amazon's server
    rather than by the device.
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

From the disassembly, the Settings switch writes the UserDefaults row as
well as the `dcc` value, which is harmless.

Undo (back to how it was found on the reference device):

```sh
sqlite3 /var/local/appreg.db "update properties set value='false' where handlerId='dcc' and name='series.grouping.enabled';"
sqlite3 /var/local/appreg.db "delete from properties where handlerId='dcc' and name='series.grouping.supported.marketplaces';"
```

## What might reset it (unconfirmed)

- `dcc` rows are written from the server's dynamic config, so a refresh
  could set `series.grouping.enabled` back to `false` or drop the
  marketplace override. What triggers a refresh, and which process writes
  `dcc` rows, hasn't been looked at.
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
