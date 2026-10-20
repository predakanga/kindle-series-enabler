#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-or-later
# Run once at boot by /etc/upstart/kindle-series-enabler.conf (ADR 0002).
# Logs to /mnt/us, where users can reach it over USB; if that can't be
# written (not mounted, full, read-only), to /var/local; failing that,
# nowhere. A logging failure never stops apply.
DIR=/var/local/kindle-series-enabler

# A failed `exec >file` would end the script; `command exec` just fails.
command exec >/mnt/us/kindle-series-enabler.log 2>&1 ||
    command exec >"$DIR/boot.log" 2>&1 ||
    exec >/dev/null 2>&1

echo "$(date): apply at boot"
exec "$DIR/kindle-series-enabler" apply
