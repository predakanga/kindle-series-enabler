#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-or-later
# Tests package/series-grouping.sh and the KPM hooks against a fixture
# appreg.db with the reference device's rows (FW 5.20.1, AU marketplace).
# Needs the sqlite3 CLI. Usage: tests/series-grouping.sh
set -eu

cd "$(dirname "$0")/.."
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
APPREG=$tmp/appreg.db
export APPREG
failures=0

US=ATVPDKIKX0DER
AU=A39IBJ37TRP1C6
DEFAULTS=A1VC38T7YXB528,$US,A1F83G8C2ARO7P,A1PA6795UKMFR9

# The reference device's schema and values (docs/research/series-grouping-gate.md).
fixture() {
    rm -f "$APPREG"
    sqlite3 "$APPREG" "
        CREATE TABLE handlerIds ( handlerId TEXT PRIMARY KEY NOT NULL ON CONFLICT IGNORE );
        CREATE TABLE properties ( handlerId TEXT NOT NULL REFERENCES handlerIds( handlerId ),
            name TEXT NOT NULL, value TEXT, PRIMARY KEY (handlerId, name) ON CONFLICT REPLACE );
        INSERT INTO handlerIds VALUES ('dcc'), ('dcd');
        INSERT INTO properties VALUES
            ('dcd', 'marketplace.obfuscated.id', '$US'),
            ('dcd', 'series.grouping.enabled', 'true'),
            ('dcd', 'series.grouping.supported', 'true'),
            ('dcd', 'series.grouping.supported.marketplaces', '$DEFAULTS'),
            ('dcc', 'marketplace.obfuscated.id', '${1:-$AU}'),
            ('dcc', 'series.grouping.enabled', 'false');"
}

dcc() { sqlite3 "$APPREG" "select value from properties where handlerId='dcc' and name='$1';"; }

check() {
    if [ "$2" = "$3" ]; then
        echo "ok   $1"
    else
        echo "FAIL $1: expected '$3', got '$2'"
        failures=$((failures + 1))
    fi
}

grouping() { sh package/series-grouping.sh "$@"; }

fixture
check "status as found" "$(grouping)" "grouping enabled: false
settings switch:  no (marketplace $AU)"

grouping enable >/dev/null
check "enable sets enabled" "$(dcc series.grouping.enabled)" true
check "enable adds the marketplace" "$(dcc series.grouping.supported.marketplaces)" "$DEFAULTS,$AU"
check "status after enable" "$(grouping status)" "grouping enabled: true
settings switch:  yes (marketplace $AU)"

grouping enable >/dev/null
check "enable twice adds it once" "$(dcc series.grouping.supported.marketplaces)" "$DEFAULTS,$AU"

grouping disable >/dev/null
check "disable sets enabled false" "$(dcc series.grouping.enabled)" false
check "disable removes the override" "$(dcc series.grouping.supported.marketplaces)" ""
check "status after disable" "$(grouping)" "grouping enabled: false
settings switch:  no (marketplace $AU)"

fixture "$US"
grouping enable >/dev/null
check "a supported marketplace needs no override" "$(dcc series.grouping.supported.marketplaces)" ""
check "a supported marketplace is enabled" "$(dcc series.grouping.enabled)" true

fixture "A39 IBJ-37"
if grouping enable >/dev/null 2>&1; then rc=0; else rc=$?; fi
check "an odd marketplace id is refused" "$rc" 1
check "and nothing is changed" "$(dcc series.grouping.enabled)" false

fixture
if grouping bogus >/dev/null 2>&1; then rc=0; else rc=$?; fi
check "an unknown command fails" "$rc" 1

# The hooks grouping from the package directory, as KPM runs them.
fixture
sh package/install.sh >/dev/null
check "install enables" "$(grouping)" "grouping enabled: true
settings switch:  yes (marketplace $AU)"
sh package/uninstall.sh upgrade >/dev/null
check "uninstall for an upgrade keeps it" "$(dcc series.grouping.enabled)" true
sh package/uninstall.sh >/dev/null
check "uninstall disables" "$(grouping)" "grouping enabled: false
settings switch:  no (marketplace $AU)"

if [ "$failures" -gt 0 ]; then
    echo "$failures failed"
    exit 1
fi
echo "all passed"
