# Changelog

All notable changes to Kindle Series Enabler. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org/). The release workflow publishes
each version's section as its GitHub release notes, so every release needs
one.

## [Unreleased]

## [1.0.0] - 2026-10-20

### Added
- Shows the Library's **Group series** switch (Settings > Home and Library
  > Library) in marketplaces where Amazon doesn't offer it, by adding this
  Kindle's marketplace to the list of supported ones. Turn grouping on or
  off with that switch.
- Puts the switch back at every boot, in case an Amazon config update
  removed it.
- Type `;series` in the search bar to put it back by hand, with the result
  shown on the screen, or `;series status` to see the values involved.
  `;series` works from the first restart after installing.
- Uninstalling removes the switch, and leaves grouping on or off as it was
  set.

[Unreleased]: https://github.com/predakanga/kindle-series-enabler/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/predakanga/kindle-series-enabler/releases/tag/v1.0.0
