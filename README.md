# Kindle Series Enabler

A [KPM](https://kindlemodding.org/jailbreaking/whats-next/installing-homebrew.html#kpm)
package for jailbroken Kindles that turns on the built-in Library's
**series grouping** (one tile per series, in order) in countries where
Amazon doesn't offer it, such as Australia.

**Status: in development, not released.** See [`docs/PLAN.md`](docs/PLAN.md).

## What it changes

Series grouping is controlled by two values in the Kindle's own settings
store, `/var/local/appreg.db`, which Amazon's servers normally set. This
package changes them. Details, and how to undo the change by hand, are in
[`docs/research/series-grouping-gate.md`](docs/research/series-grouping-gate.md).

This changes an Amazon-side setting. It's possible that Amazon resets it,
or that it has effects nobody has seen yet.

It pairs well with
[Kindle Wireless Sync](https://github.com/predakanga/kindle-wireless-sync),
which sends Calibre series to the Kindle as native series.

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

