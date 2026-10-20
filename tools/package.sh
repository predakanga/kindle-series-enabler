#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-or-later
# Builds the KPM package from an already built binary and package/. Used by
# `just package` on macOS and by the release workflow on Linux, so it
# handles both bsdtar and GNU tar. Modelled on Kindle Wireless Sync's
# tools/package.sh.
#
# Usage: tools/package.sh BINARY OUT.kpkg
set -eu

bin=$1
out=$2
scripts="install.sh uninstall.sh launch.sh rootfs.sh boot.sh series.sh"
stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT

version=$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -1)
echo "$version" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$' || { echo "no X.Y.Z version in Cargo.toml" >&2; exit 1; }
sed "s|@VERSION@|$(echo "$version" | tr . ,)|" package/manifest.json.in > "$stage/manifest.json"
jq -e . "$stage/manifest.json" >/dev/null || { echo "manifest.json isn't valid JSON" >&2; exit 1; }

cp "$bin" "$stage/kindle-series-enabler"
cp package/upstart/kindle-series-enabler.conf "$stage/"
for f in $scripts; do
    cp "package/$f" "$stage/$f"
done
chmod 755 "$stage/kindle-series-enabler" "$stage"/*.sh
chmod 644 "$stage/manifest.json" "$stage/kindle-series-enabler.conf"

# KPM needs manifest.json at the archive root (no "./"), so name every entry.
# Root-owned entries, and no macOS metadata (._* files, xattrs).
if tar --version 2>/dev/null | grep -q bsdtar; then
    owner="--uid 0 --gid 0 --no-xattrs --no-acls --no-mac-metadata"
else
    owner="--owner=0 --group=0 --numeric-owner --no-xattrs --no-acls"
fi
# shellcheck disable=SC2086 # $owner and $scripts are lists
COPYFILE_DISABLE=1 tar $owner -czf "$out" -C "$stage" \
    manifest.json kindle-series-enabler kindle-series-enabler.conf $scripts

tar -tzf "$out" | grep -qx manifest.json || { echo "manifest.json is not at the archive root" >&2; exit 1; }
ls -l "$out"
