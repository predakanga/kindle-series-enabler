#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-or-later
# KPM uninstall hook (ADR 0003). KPM runs it from the package directory.

# On upgrade KPM runs this before installing the new version, which
# replaces the files and keeps the record: change nothing.
[ "${1:-}" = upgrade ] && exit 0

# Any failure exits non-zero, so KPM keeps the package and the user can
# retry. Each step is safe to repeat.
set -e
umask 022

PKG=$(cd "$(dirname "$0")" && pwd)
DIR=/var/local/kindle-series-enabler
JOB=/etc/upstart/kindle-series-enabler.conf
# The package's own copy, in case $DIR is already gone.
BIN=$PKG/kindle-series-enabler
# shellcheck source=package/rootfs.sh
. "$PKG/rootfs.sh"

# Puts the list back only if it still holds what apply wrote. Grouping
# itself stays on or off, as the user set it (ADR 0003).
"$BIN" restore --state "$DIR"

rootfs_rw
echo "Removing the boot job and the ;series search command"
"$BIN" unregister
rm -f "$JOB"
/sbin/initctl reload-configuration 2>/dev/null || true

rm -rf "$DIR"
echo "Uninstalled. Grouping stays as it was set in Settings."
