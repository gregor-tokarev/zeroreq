#!/bin/bash

set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP="$ROOT/target/debug/Request Eagle.app/Contents/MacOS/request-eagle"
POLL_INTERVAL="${REQUEST_EAGLE_DEV_POLL_INTERVAL:-0.5}"
APP_PID=""

snapshot() {
	{
		for file in Cargo.toml Cargo.lock Makefile; do
			test ! -f "$ROOT/$file" || printf '%s\n' "$file"
		done
		find "$ROOT/crates" "$ROOT/packaging/macos" -type f -print |
			sed "s|^$ROOT/||"
	} |
		LC_ALL=C sort |
		while IFS= read -r file; do
			printf '%s  ' "$file"
			shasum "$ROOT/$file"
		done |
		shasum |
		awk '{print $1}'
}

stop_app() {
	if test -z "$APP_PID"; then
		return
	fi

	if kill -0 "$APP_PID" 2>/dev/null; then
		kill "$APP_PID" 2>/dev/null || true
	fi
	wait "$APP_PID" 2>/dev/null || true
	APP_PID=""
}

shutdown() {
	stop_app
	exit 0
}

trap shutdown INT TERM
trap stop_app EXIT

echo "Watching Cargo and application files. Press Ctrl-C to stop."

LAST_SNAPSHOT=""
while true; do
	CURRENT_SNAPSHOT="$(snapshot)"

	if test "$CURRENT_SNAPSHOT" = "$LAST_SNAPSHOT"; then
		sleep "$POLL_INTERVAL"
		continue
	fi

	LAST_SNAPSHOT="$CURRENT_SNAPSHOT"
	stop_app
	echo
	echo "Change detected; rebuilding Request Eagle..."

	if "${MAKE:-make}" --no-print-directory -C "$ROOT" bundle; then
		# If another edit landed during the build, rebuild once more before launch.
		if test "$(snapshot)" != "$LAST_SNAPSHOT"; then
			LAST_SNAPSHOT=""
			continue
		fi

		echo "Launching Request Eagle..."
		"$APP" &
		APP_PID=$!
	else
		echo "Build failed; waiting for another change."
	fi
done
