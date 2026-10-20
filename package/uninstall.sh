#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-or-later
# KPM uninstall hook. KPM runs it from the package directory.
# Draft (PLAN Phase 0): what uninstalling should restore is decided in
# Phase 1 and 2.

# On upgrade KPM runs this before installing the new version, which
# applies the settings again: leave them.
[ "$1" = upgrade ] && exit 0

# Any failure exits non-zero, so KPM keeps the package and the user can retry.
set -e

PKG=$(cd "$(dirname "$0")" && pwd)

echo "Turning off series grouping"
sh "$PKG/series-grouping.sh" disable
echo "Uninstalled."
