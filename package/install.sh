#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-or-later
# KPM install hook. KPM runs it from the extracted package
# (/mnt/us/kmc/kpm/packages/kindle-series-enabler/) on install and upgrade.
# Draft (PLAN Phase 0): what installing should do is decided in Phase 2.
# Safe to repeat: if this fails, KPM keeps no record, and the fix is to
# install again.
set -e

PKG=$(cd "$(dirname "$0")" && pwd)

echo "Turning on series grouping"
sh "$PKG/series-grouping.sh" enable
echo "Installed. Series are grouped the next time the Library is shown."
