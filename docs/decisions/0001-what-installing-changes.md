# 0001: What installing changes

- Status: Proposed
- Date: 2026-10-20

## Context
Grouping on 5.20.1 depends on three values (research note
`series-grouping-gate.md`, "Which change does it" and "Telling a push from
the user"):
- `dcc series.grouping.enabled` in `/var/local/appreg.db` is the on/off
  switch. With it `true`, series are grouped even where the Settings
  switch is hidden.
- A `dcc` override of `series.grouping.supported.marketplaces` that lists
  the device's marketplace shows the Settings switch ("Group series",
  Settings > Home and Library > Library tab). It has no effect on
  grouping itself.
- The UserDefaults key
  `IGNITE_LIBRARY_NAMESPACE.IGNITE_SERIES_GROUPING_ENABLED_KEY` in
  `/var/local/user_defaults.db` has no effect on the reference device. But
  on devices with the library re-architecture it is the value, and the
  switch is always shown there. The Settings switch writes it as well as
  `dcc`.

Everything applies live: the Library picks it up when it next redraws.

The package has no UI, so installing and uninstalling are its controls.
Kindle Wireless Sync's plan (its Later 4) wanted a script that required
the device's UUID as an argument, as a deliberate confirmation.

## Decision
**Installing turns grouping on and shows the switch.** `install.sh`:
1. Adds the override, unless the device's marketplace is already in the
   `dcd` list. The override is the `dcd` list with the marketplace added.
2. Sets `dcc series.grouping.enabled` to `true`.
3. Sets the UserDefaults key to `1`, as the Settings switch does.

After that, the user turns grouping off and on with the Settings switch,
like any other setting. The package doesn't touch it again, except as ADR
0003 allows.

If `series.grouping.supported` isn't `true`, the tool says so and changes
nothing else. That hasn't been seen, so what it would take is unknown.

**No confirmation beyond installing.** Installing a KPM package is
already a deliberate act. The README and the KPM description say plainly
that this changes an Amazon-side setting.

## Alternatives considered
- **Only show the switch** (the override alone), and let the user turn
  grouping on in Settings. It's the least intrusive option, and the user
  makes the choice in Amazon's own UI. But installing would then seem to
  do nothing until the user finds a switch three levels down. Kept as a
  possible option if the package gains settings later.
- **Only set `enabled`,** with no override: grouping works, but there's
  no way to turn it off except uninstalling. Users lose the choice the
  Settings switch gives them.
- **Not writing UserDefaults:** it has no effect on the reference device,
  so it's arguably outside "only what grouping needs". But on library
  re-architecture devices it's the actual value, and writing it keeps the
  two stores agreeing as the switch would. That agreement is also what
  ADR 0003 uses to tell a server push from the user.
- **A UUID confirmation argument** (Wireless Sync's Later 4): made sense
  for a script inside another package. A dedicated package's install is
  the confirmation.

## Consequences
- The tool writes three rows in two databases: two `dcc` rows in
  `appreg.db`, and one row in `user_defaults.db`.
- The override is a copy of the `dcd` list as it was at install. If a
  later firmware adds marketplaces to `dcd`, the copy hides them. Only the
  device's own marketplace matters, so this is harmless, but `status`
  should show it.
- Untested: devices with the library re-architecture, and firmware other
  than 5.20.1 (PLAN Phase 1, item 4).
