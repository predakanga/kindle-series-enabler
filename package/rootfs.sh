# SPDX-License-Identifier: GPL-3.0-or-later
# Sourced by install.sh and uninstall.sh (ADR 0003): rootfs_rw makes the
# root filesystem writable, and on exit, success or not, it's made
# read-only again. If it was writable already, it's left that way.
# shellcheck shell=sh

remounted=0
trap 'if [ "$remounted" = 1 ]; then /usr/sbin/mntroot ro >/dev/null || echo "warning: the root filesystem is still writable" >&2; fi' EXIT
trap 'exit 130' INT TERM HUP

rootfs_rw() {
    [ "$remounted" = 1 ] && return
    # The last entry for / is the one in effect.
    if awk '$2 == "/" { opts = $4 } END { exit !(opts ~ /^rw(,|$)/) }' /proc/mounts; then
        return
    fi
    /usr/sbin/mntroot rw >/dev/null
    remounted=1
}
