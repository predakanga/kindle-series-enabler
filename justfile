# Kindle Series Enabler - build, test, package and release

kindle_target := "armv7-unknown-linux-musleabihf"
# SSH host for the device recipes. `kindle` is the reference device; set
# KINDLE_HOST to use another, e.g. `kindle-emu` for an eink-emulator guest.
kindle_host := env("KINDLE_HOST", "kindle")
# On the device: persistent, writable, not exposed over USB (ADR 0003).
kindle_dir := "/var/local/kindle-series-enabler"
package_id := "kindle-series-enabler"

default:
    @just --list

# Build for the host
build:
    cargo build

# Build the static Kindle binary (Zig compiles the bundled SQLite; ADR 0004)
build-kindle:
    cargo zigbuild --release --target {{kindle_target}}

# Run all tests
test:
    cargo test

# Run tests whose name matches a filter, e.g. `just test-one restore`
test-one filter:
    cargo test {{filter}}

# Format, lint, and check every shell script
check:
    cargo fmt --check
    cargo clippy --all-targets -- -D warnings
    shellcheck -x package/*.sh tools/*.sh

fmt:
    cargo fmt

# Show the Kindle binary's size
size-kindle: build-kindle
    @ls -l target/{{kindle_target}}/release/kindle-series-enabler | awk '{print $5 " bytes"}'

# Copy the binary and scripts to the device (writable storage; no root remount)
deploy: build-kindle
    ssh {{kindle_host}} "mkdir -p {{kindle_dir}}"
    # Copy under a temporary name, then rename over the old binary.
    scp -O -q target/{{kindle_target}}/release/kindle-series-enabler {{kindle_host}}:{{kindle_dir}}/kindle-series-enabler.new
    # One file per scp: the device's dropbear aborts on a multi-file copy here.
    scp -O -q package/boot.sh {{kindle_host}}:{{kindle_dir}}/boot.sh
    scp -O -q package/series.sh {{kindle_host}}:{{kindle_dir}}/series.sh
    ssh {{kindle_host}} "chmod 755 {{kindle_dir}}/kindle-series-enabler.new {{kindle_dir}}/boot.sh {{kindle_dir}}/series.sh && mv -f {{kindle_dir}}/kindle-series-enabler.new {{kindle_dir}}/kindle-series-enabler"

# Run the deployed binary on the device, e.g. `just kse status`
kse *args:
    ssh {{kindle_host}} "{{kindle_dir}}/kindle-series-enabler {{args}}"

# Keep the Kindle awake for long tests (Wi-Fi drops with the screensaver; a reboot turns it back on)
screensaver-off:
    ssh {{kindle_host}} "lipc-set-prop -i com.lab126.powerd preventScreenSaver 1 && lipc-get-prop com.lab126.powerd preventScreenSaver"

# Let the Kindle sleep again
screensaver-on:
    ssh {{kindle_host}} "lipc-set-prop -i com.lab126.powerd preventScreenSaver 0 && lipc-get-prop com.lab126.powerd preventScreenSaver"

# Build the KPM package: target/package/kindle-series-enabler.kpkg
package: build-kindle
    mkdir -p target/package
    sh tools/package.sh target/{{kindle_target}}/release/kindle-series-enabler target/package/{{package_id}}.kpkg

# Prepare a release: bump Cargo.toml, date the changelog, commit (no push)
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
