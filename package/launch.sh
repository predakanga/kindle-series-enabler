#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-or-later
# KPM launch hook: `kpm launch kindle-series-enabler` runs apply (ADR 0002),
# printing to stdout.
exec /var/local/kindle-series-enabler/kindle-series-enabler "$@"
