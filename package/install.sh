#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-or-later
# KPM install hook (ADR 0003). KPM runs it from the extracted package
# (/mnt/us/kmc/kpm/packages/kindle-series-enabler/) on install and upgrade.
# Every step is safe to repeat: if this fails, KPM keeps no record, and the
# fix is to install again.
set -e
# KPM runs this with the umask of the shell that ran `kpm`, which can be 077.
umask 022

PKG=$(cd "$(dirname "$0")" && pwd)
DIR=/var/local/kindle-series-enabler
JOB=/etc/upstart/kindle-series-enabler.conf
# shellcheck source=package/rootfs.sh
. "$PKG/rootfs.sh"

# Copy under a temporary name, then rename.
put() {
    cp "$1" "$2.new"
    chmod "$3" "$2.new"
    mv -f "$2.new" "$2"
}

fresh=1
[ -f "$JOB" ] && fresh=0

echo "Installing to $DIR"
mkdir -p "$DIR"
chmod 755 "$DIR"
for f in kindle-series-enabler boot.sh series.sh; do
    put "$PKG/$f" "$DIR/$f" 755
done

# Before anything on the root filesystem, so that on a Kindle where it
# can't work nothing else is changed.
if ! "$DIR/kindle-series-enabler" apply; then
    [ "$fresh" = 1 ] && rm -rf "$DIR"
    echo "Not installed. Nothing else was changed." >&2
    exit 1
fi

rootfs_rw
echo "Installing the boot job and the ;series search command"
put "$PKG/kindle-series-enabler.conf" "$JOB" 644
changes=$("$DIR/kindle-series-enabler" register)
echo "$changes"
/sbin/initctl reload-configuration 2>/dev/null || true

# KPPMainApp reads the search commands only at startup.
case $changes in
    *": added"*) echo "Installed. ;series works in the search bar after the next restart." ;;
    *) echo "Installed." ;;
esac
