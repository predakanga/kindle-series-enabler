# 0001: What the package changes: the marketplace override, nothing else

- Status: Proposed
- Date: 2026-10-20

## Context
Grouping on 5.20.1 depends on these values (research note
`series-grouping-gate.md`, "Which change does it" and "Telling a push from
the user"):
- `dcc series.grouping.enabled` in `/var/local/appreg.db` is the on/off
  switch. The Settings switch writes it, along with the UserDefaults key
  `IGNITE_LIBRARY_NAMESPACE.IGNITE_SERIES_GROUPING_ENABLED_KEY`.
- `series.grouping.supported.marketplaces` (`dcd` default, optional `dcc`
  override) decides whether the Settings switch ("Group series",
  Settings > Home and Library > Library tab) is shown. A list that includes
  the device's marketplace shows it.
- `series.grouping.supported` must be `true` (it is, on the reference
  device).

Everything applies live: the Library and Settings pick it up on their next
redraw.

The developer's direction (2026-10-20): the package **makes grouping
available**, by setting the marketplace override, and nothing else.

## Decision
**The package writes one row: the `dcc` override of
`series.grouping.supported.marketplaces`.** `apply`:
1. Reads the marketplace from `marketplace.obfuscated.id` (`dcc`, else
   `dcd`). It refuses an empty ID, or one with characters outside `A-Z0-9`,
   since the list is comma-separated.
2. If `series.grouping.supported` (`dcc`, else `dcd`) isn't `true`, it says
   so and changes nothing. That hasn't been seen, so what it would take is
   unknown.
3. Takes the effective list (the `dcc` override if there is one, else the
   `dcd` default). If the marketplace is already in it, it changes nothing
   and says so.
4. Otherwise it writes the `dcc` override: the effective list with the
   marketplace added at the end. Before writing, it records the old value
   for uninstall (ADR 0003).

It never touches `series.grouping.enabled` or UserDefaults. With the switch
shown, the user turns grouping on or off in Settings, as in a supported
marketplace.

Installing is the confirmation. There's no UUID argument (Wireless Sync's
plan, its Later 4, wanted one for a script inside another package).

## Alternatives considered
- **Also set `enabled` to `true`, and UserDefaults to `1`** (the earlier
  draft of this ADR): installing would turn grouping on at once, but it
  takes the choice from the user and changes two more Amazon values.
  Turning the switch on is one tap.
- **Only set `enabled`**, with no override: grouping works, but the user
  can't turn it off without uninstalling.
- **Override with only the device's marketplace,** not the default list
  plus it: just as effective here, but it changes what the list says for
  every other marketplace. Adding to the list is the smallest change.

## Consequences
- After installing, the user turns on "Group series" in Settings. The
  README says where. On the reference device `enabled` started `false`, so
  this tap is needed there.
- The override is a copy of the effective list at the moment `apply`
  writes it, which is normally at install. Later runs find the marketplace
  already listed and leave the row alone. It's only written again if
  something (a push) replaces it with a list that doesn't include the
  marketplace. So if a later firmware adds marketplaces to `dcd`, the copy
  hides them. Only the device's own marketplace matters, so that's
  harmless, but `status` shows both lists.
- If `enabled` is already `true` (the `dcd` default, with no `dcc` row),
  grouping turns on as soon as the switch can be shown. That's Amazon's
  default, not something the package sets.
- Untested: devices with the library re-architecture, and firmware other
  than 5.20.1 (PLAN Phase 1, item 4).
