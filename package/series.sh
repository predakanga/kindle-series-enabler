#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-or-later
# Run by the ;series search-bar command (ADR 0002), as root, by KPPMainApp.
# The rest of the typed line arrives as arguments: none means apply,
# `;series status` shows the status. Shows the output on the screen, then
# repaints. Follows KMC's /var/local/kmc/sbin/kpm.sh.
DIR=/var/local/kindle-series-enabler
FBINK=/var/local/kmc/bin/fbink

sleep 1 # let the keyboard close

out=$("$DIR/kindle-series-enabler" "$@" 2>&1)

# One argument per line of output.
set -f
IFS='
'
# shellcheck disable=SC2086 # split into lines on purpose
set -- $out
unset IFS
set +f
[ $# -gt 0 ] || set -- "kindle-series-enabler printed nothing"

if [ -x "$FBINK" ]; then
    # FBInk wraps long lines and puts each argument on its own line.
    "$FBINK" -q -x 1 -y 2 -S 3 "$@"
else
    row=2
    for line in "$@"; do
        eips 1 "$row" "$line" >/dev/null
        row=$((row + 1))
    done
fi

# Long enough to read, then repaint over the text, as kpm.sh does.
sleep $((4 + $#))
/usr/bin/xrefresh -d :0.0
