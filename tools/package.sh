#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-or-later
# Builds the KPM package from package/. Used by `just package` on macOS and
# by the release workflow on Linux, so it handles both bsdtar and GNU tar.
# Modelled on Kindle Wireless Sync's tools/package.sh.
#
# Usage: tools/package.sh OUT.kpkg
set -eu

out=$1
stage=$(dirname "$out")/kpkg
files="install.sh uninstall.sh series-grouping.sh"

rm -rf "$stage"
mkdir -p "$stage"

version=$(tr -d '[:space:]' < VERSION)
echo "$version" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$' || { echo "VERSION isn't X.Y.Z" >&2; exit 1; }
sed "s|@VERSION@|$(echo "$version" | tr . ,)|" package/manifest.json.in > "$stage/manifest.json"
jq -e . "$stage/manifest.json" >/dev/null || { echo "manifest.json isn't valid JSON" >&2; exit 1; }

for f in $files; do
    cp "package/$f" "$stage/$f"
    chmod 755 "$stage/$f"
done
chmod 644 "$stage/manifest.json"

# KPM needs manifest.json at the archive root (no "./"), so name every entry.
# Root-owned entries, and no macOS metadata (._* files, xattrs).
if tar --version 2>/dev/null | grep -q bsdtar; then
    owner="--uid 0 --gid 0 --no-xattrs --no-acls --no-mac-metadata"
else
    owner="--owner=0 --group=0 --numeric-owner --no-xattrs --no-acls"
fi
# shellcheck disable=SC2086 # $owner and $files are lists
COPYFILE_DISABLE=1 tar $owner -czf "$out" -C "$stage" manifest.json $files

tar -tzf "$out" | grep -qx manifest.json || { echo "manifest.json is not at the archive root" >&2; exit 1; }
ls -l "$out"
