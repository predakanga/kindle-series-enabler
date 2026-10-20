# Kindle Series Enabler - test, package and release

# SSH host for the device recipes. `kindle` is the reference device; set
# KINDLE_HOST to use another, e.g. `kindle-emu` for an eink-emulator guest.
kindle_host := env("KINDLE_HOST", "kindle")
package_id := "kindle-series-enabler"

default:
    @just --list

# Run the tests (needs the sqlite3 CLI)
test:
    sh tests/series-grouping.sh

# Lint every shell script
check:
    shellcheck package/*.sh tools/*.sh tests/*.sh

# Series grouping on the Kindle: status (default), enable or disable
grouping action="status":
    ssh {{kindle_host}} "sh -s -- {{action}}" < package/series-grouping.sh

# Keep the Kindle awake for long tests (Wi-Fi drops with the screensaver; a reboot turns it back on)
screensaver-off:
    ssh {{kindle_host}} "lipc-set-prop -i com.lab126.powerd preventScreenSaver 1 && lipc-get-prop com.lab126.powerd preventScreenSaver"

# Let the Kindle sleep again
screensaver-on:
    ssh {{kindle_host}} "lipc-set-prop -i com.lab126.powerd preventScreenSaver 0 && lipc-get-prop com.lab126.powerd preventScreenSaver"

# Build the KPM package: target/package/kindle-series-enabler.kpkg
package:
    mkdir -p target/package
    sh tools/package.sh target/package/{{package_id}}.kpkg

# Prepare a release: bump VERSION, date the changelog, commit (no push)
release version:
    sh tools/release.sh prepare {{version}}

# Tag the prepared release and push it, which publishes it (asks first)
publish-release *flags:
    sh tools/release.sh publish {{flags}}

# Install the package with the device's own KPM, as a user would
kpm-install: package
    scp -O -q target/package/{{package_id}}.kpkg {{kindle_host}}:/tmp/{{package_id}}.kpkg
    ssh {{kindle_host}} "/var/local/kmc/bin/kpm -y install file:///tmp/{{package_id}}.kpkg; rc=\$?; rm -f /tmp/{{package_id}}.kpkg; exit \$rc"

# Uninstall the package with the device's own KPM
kpm-uninstall:
    ssh {{kindle_host}} "/var/local/kmc/bin/kpm -y uninstall {{package_id}}"
