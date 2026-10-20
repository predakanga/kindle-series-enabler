# Kindle Series Enabler

[![CI](https://github.com/predakanga/kindle-series-enabler/actions/workflows/ci.yml/badge.svg)](https://github.com/predakanga/kindle-series-enabler/actions/workflows/ci.yml)
[![Latest release](https://img.shields.io/github/v/release/predakanga/kindle-series-enabler)](https://github.com/predakanga/kindle-series-enabler/releases/latest)
[![Licence: GPL-3.0-or-later](https://img.shields.io/badge/licence-GPL--3.0--or--later-blue)](LICENSE)

Turn on the Kindle Library's built-in **series grouping** (one tile per
series, with the books in order) in countries where Amazon doesn't offer
it, such as Australia.

The package shows Amazon's own **Group series** switch in Settings, which
is normally hidden outside the countries Amazon has chosen. You then turn
grouping on or off with that switch, like any other setting.

> [!WARNING]
> **Written with AI, and tested on one Kindle.**
>
> - This project was written using AI (Anthropic's Claude, through Claude
>   Code), directed and reviewed by its developer.
> - It has only been tested on the developer's personal **Kindle Paperwhite
>   (12th Generation) on firmware 5.20.1**, in the Australian marketplace.
>   Other models and firmware versions may work, but are untested.
>
> It changes a setting that Amazon's servers normally control, on a
> jailbroken device. Amazon could reset it, and it may have effects nobody
> has seen yet. Use it at your own risk. It comes with no warranty (see
> [Licence](#licence)).

## Requirements

- A jailbroken Kindle with [KPM](https://kindlemodding.org/jailbreaking/whats-next/installing-homebrew.html#kpm)
  (the Kindle Package Manager) installed.

## Install

With Wi-Fi on, in kterm or over SSH on the Kindle, add the repository once,
then install:

```sh
kpm add-repo https://predakanga.github.io/kindle-apps/repo.json
kpm update
kpm install kindle-series-enabler
```

To upgrade, run `kpm update` and `kpm install kindle-series-enabler` again.
Install from the repository rather than from a downloaded `.kpkg` file,
because KPM can't upgrade a package that was installed from a file.

If your marketplace already offers series grouping, there's nothing to do,
and the install says so.

## Use

1. Open **Settings > Home and Library > Library**.
2. Turn on **Group series**.

The Library groups your series the next time it's shown.

**If the Group series switch disappears** (an Amazon update can do that),
restart the Kindle, or type `;series` in the search bar. Either puts it
back, and grouping stays as you set it. `;series status` in the search bar
shows the values involved. The `;series` command works from the first
restart after installing.

**After a firmware update,** reinstall the package. Updates replace the
system files that hold the boot job and the `;series` command.

## Uninstall

```sh
kpm uninstall kindle-series-enabler
```

This hides the Group series switch again. Grouping stays on or off, as it
was set. To turn it off, switch it off in Settings before uninstalling, or
reinstall, switch it off, and uninstall again.

## What it changes

- **One setting:** it adds your Kindle's marketplace to
  `series.grouping.supported.marketplaces` in `/var/local/appreg.db`, as an
  override (`dcc`) of Amazon's default list (`dcd`). That list is what
  decides whether Settings shows the Group series switch. The package
  never changes whether grouping is on (`series.grouping.enabled`): the
  switch does that.
- **A boot job,** `/etc/upstart/kindle-series-enabler.conf`, that adds the
  marketplace again at each boot if something removed it. It runs once and
  exits, and logs to `kindle-series-enabler.log` on the Kindle's USB
  storage.
- **The `;series` search command,** added to the Kindle's search-command
  files (`/app/kpp_app_cmds.json` and `/usr/share/app/kpp_sys_cmds.json` on
  firmware 5.20.1), as the KMC jailbreak adds `;kpm`.
- **Its files,** in `/var/local/kindle-series-enabler/`.

Uninstalling removes all four. It puts the list back as it was, unless it
has changed since (for example, an Amazon update set it), in which case
it's left alone.

The details, and the research behind them, are in
[`docs/research/`](docs/research/) and [`docs/decisions/`](docs/decisions/).

## Development

The code is Rust, built for `armv7-unknown-linux-musleabihf` as one static
binary with SQLite compiled in, with [`just`](https://github.com/casey/just)
recipes for building, testing and deploying to a Kindle over SSH. The
Kindle build needs [Zig](https://ziglang.org/) and
[`cargo-zigbuild`](https://github.com/rust-cross/cargo-zigbuild). Start
with:

- [`CLAUDE.md`](CLAUDE.md): the commands, the code layout and how to work
  with the device.
- [`docs/PLAN.md`](docs/PLAN.md): the phases and what's next.
- [`docs/decisions/`](docs/decisions/): the design decisions (ADRs).
- [`docs/research/`](docs/research/): how the Kindle gates series grouping,
  how search commands work, and KPM's behaviour.

Changes are listed in [`CHANGELOG.md`](CHANGELOG.md).

## Credits

- [Kindle Wireless Sync](https://github.com/predakanga/kindle-wireless-sync),
  which this package was split out of, supplied the tooling, the KPM
  packaging and the boot job's design. It sends Calibre series to the
  Kindle as native series, which this package lets you see.
- The [Kindle Modding Club](https://kindlemodding.org)'s jailbreak and
  [documentation](https://github.com/KindleModding/kindlemodding.github.io)
  showed how search commands are defined. Its system patcher
  ([jb.sh](https://github.com/KindleModding/jb.sh)) was the model for
  adding one to every command file, and its `kpm.sh` for showing output on
  the screen with [FBInk](https://github.com/NiLuJe/FBInk).

## Licence

Copyright (C) 2026 predakanga

This program is free software: you can redistribute it and/or modify it
under the terms of the GNU General Public License as published by the Free
Software Foundation, either version 3 of the License, or (at your option)
any later version.

This program is distributed in the hope that it will be useful, but WITHOUT
ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for
more details.

You should have received a copy of the GNU General Public License along with
this program (see [`LICENSE`](LICENSE)). If not, see
<https://www.gnu.org/licenses/>.

Each source file carries an `SPDX-License-Identifier: GPL-3.0-or-later`
header.
