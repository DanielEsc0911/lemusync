#!/usr/bin/env bash
# Runs the Secret Service tests (crates/engine/tests/secret_service.rs) on a throwaway session
# bus, never on yours (security §Secret Service activation).
#
#   scripts/isolated-secret-service.sh <parent-dir>
#
# - Everything lives in <parent-dir>/lemusync-ss: HOME, XDG_DATA_HOME, XDG_CONFIG_HOME,
#   XDG_CACHE_HOME and the runtime dir. Socket paths are capped at 108 bytes, so
#   XDG_RUNTIME_DIR is the short symlink /tmp/lemusync-ss-$UID pointing into it.
# - The bus has no standard service dirs: the only activatable org.freedesktop.secrets is a
#   trap that records it was started and exits. A regression shows up as that marker, and
#   nothing real (gnome-keyring, KWallet) can ever be activated.
# - gnome-keyring is started here only as a fixture, with a throwaway keyring.
# - Every process started carries LEMUSYNC_ISOLATED_MARKER in its environment. Cleanup (on
#   exit, Ctrl-C, TERM, and again before each run in case an earlier one crashed) kills only
#   processes with that variable and deletes only folders holding our marker file. Run one
#   copy at a time.
set -euo pipefail

LINK="/tmp/lemusync-ss-$(id -u)"
START="$PWD"
SELF="$(cd "$(dirname "$0")" && pwd)/$(basename "$0")"
cd "$(dirname "$SELF")/.."
MARK_FILE=".lemusync-isolated"

# Our processes: their environment has LEMUSYNC_ISOLATED_MARKER. Nobody else's ever does.
marked_pids() {
    local pid
    for pid in $(pgrep -u "$(id -u)"); do
        [[ "$pid" == "$$" ]] && continue
        if { tr '\0' '\n' <"/proc/$pid/environ"; } 2>/dev/null | grep -q '^LEMUSYNC_ISOLATED_MARKER='; then
            echo "$pid"
        fi
    done
}

remove_base() {
    local base="$1"
    if [[ -f "$base/$MARK_FILE" && "$(basename "$base")" == lemusync-ss ]]; then
        rm -rf -- "$base"
    fi
}

sweep() {
    local pids
    pids="$(marked_pids)"
    if [[ -n "$pids" ]]; then
        echo "isolated-secret-service: stopping leftover processes: $(echo $pids)" >&2
        kill -TERM $pids 2>/dev/null || true
        for _ in 1 2 3 4 5 6 7 8 9 10; do
            [[ -z "$(marked_pids)" ]] && break
            sleep 0.2
        done
        pids="$(marked_pids)"
        [[ -n "$pids" ]] && kill -KILL $pids 2>/dev/null || true
    fi
    if [[ -L "$LINK" ]]; then
        remove_base "$(dirname "$(readlink "$LINK")")"
        rm -f -- "$LINK"
    fi
}

if [[ "${1:-}" == "--inner" ]]; then
    # Inside dbus-run-session, on the throwaway bus.
    base="$LEMUSYNC_ISOLATED_MARKER"
    test_run() {
        cargo test -q -p lemusync-engine --test secret_service --offline -- \
            --ignored --exact --test-threads=1 "$1"
    }
    has_owner() {
        dbus-send --session --print-reply --dest=org.freedesktop.DBus /org/freedesktop/DBus \
            org.freedesktop.DBus.NameHasOwner string:org.freedesktop.secrets | grep -q 'boolean true'
    }

    echo "== no owner: lemusync waits and nothing is activated"
    test_run no_owner_means_wait_and_no_activation

    echo "== an owner appears later: lemusync picks it up"
    (
        set -e
        exec 2>>"$base/fixture.log"
        sleep 2
        printf '%s' lemusync-test | gnome-keyring-daemon --foreground --components=secrets --unlock \
            >"$base/gkd.log" 2>&1 &
        until has_owner; do sleep 0.1; done
        # Fixture only, on this bus: the throwaway keyring is the default, plus another
        # app's item that lemusync must never see or touch.
        alias="$(dbus-send --session --print-reply --dest=org.freedesktop.secrets \
            /org/freedesktop/secrets org.freedesktop.Secret.Service.ReadAlias string:default)"
        if grep -q 'object path "/"' <<<"$alias"; then
            dbus-send --session --print-reply --dest=org.freedesktop.secrets \
                /org/freedesktop/secrets org.freedesktop.Secret.Service.SetAlias \
                string:default objpath:/org/freedesktop/secrets/collection/login >/dev/null
        fi
        printf '%s' other-secret | secret-tool store --label=other \
            service other-app username lemusync-selftest
        touch "$base/fixture-ready"
    ) &
    test_run waits_for_an_owner_then_round_trips || {
        cat "$base/fixture.log" "$base/gkd.log" >&2 2>/dev/null || true
        exit 1
    }

    echo "== the activation trap never fired"
    [[ ! -e "$base/ACTIVATED" ]]
    echo "isolated-secret-service: all checks passed"
    exit 0
fi

parent="${1:?usage: $0 <parent-dir>}"
parent="$(cd "$START" && cd "$parent" && pwd)"
base="$parent/lemusync-ss"
real_bus="${DBUS_SESSION_BUS_ADDRESS:-}"

sweep
trap 'sweep' EXIT INT TERM

mkdir -p "$base"/{home,data,config,cache,runtime,services}
touch "$base/$MARK_FILE"
chmod 700 "$base/runtime"
ln -s "$base/runtime" "$LINK"

cat >"$base/activated.sh" <<EOF
#!/bin/sh
touch "$base/ACTIVATED"
exit 1
EOF
chmod +x "$base/activated.sh"
cat >"$base/services/org.freedesktop.secrets.service" <<EOF
[D-BUS Service]
Name=org.freedesktop.secrets
Exec=$base/activated.sh
EOF
cat >"$base/bus.conf" <<EOF
<!DOCTYPE busconfig PUBLIC "-//freedesktop//DTD D-Bus Bus Configuration 1.0//EN"
 "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
<busconfig>
  <type>session</type>
  <keep_umask/>
  <listen>unix:dir=$LINK</listen>
  <auth>EXTERNAL</auth>
  <servicedir>$base/services</servicedir>
  <policy context="default">
    <allow send_destination="*" eavesdrop="true"/>
    <allow eavesdrop="true"/>
    <allow own="*"/>
  </policy>
</busconfig>
EOF

# Build outside the bus first (no D-Bus involved), so the run itself is quick.
cargo test -q -p lemusync-engine --test secret_service --offline --no-run

# A clean environment: no DISPLAY/WAYLAND_DISPLAY (no dialogs on your screen), no real bus.
env -i \
    PATH="$PATH" \
    CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}" \
    RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}" \
    HOME="$base/home" \
    XDG_DATA_HOME="$base/data" \
    XDG_CONFIG_HOME="$base/config" \
    XDG_CACHE_HOME="$base/cache" \
    XDG_RUNTIME_DIR="$LINK" \
    LEMUSYNC_ISOLATED_MARKER="$base" \
    LEMUSYNC_REAL_BUS="$real_bus" \
    dbus-run-session --config-file="$base/bus.conf" -- "$SELF" --inner
