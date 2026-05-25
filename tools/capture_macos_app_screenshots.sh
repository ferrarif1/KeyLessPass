#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_PATH="${KEYLESSPASS_APP:-$ROOT_DIR/releases/macos/KeyLessPass.app}"
APP_BIN="$APP_PATH/Contents/MacOS/KeyLessPass"
SCREENSHOT_LOCALE="${KEYLESSPASS_SCREENSHOT_LOCALE:-en}"
OUT_DIR="$ROOT_DIR/docs/readme-assets/screenshots"
DOCS_OUT_DIR="$ROOT_DIR/docs/assets/screenshots"
WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/keylesspass-screenshots.XXXXXX")"
CURRENT_PID=""

cleanup() {
  if [[ -n "$CURRENT_PID" ]]; then
    kill "$CURRENT_PID" >/dev/null 2>&1 || true
    wait "$CURRENT_PID" >/dev/null 2>&1 || true
  fi
  rm -rf "$WORK_DIR"
}
trap cleanup EXIT

if [[ ! -x "$APP_BIN" ]]; then
  echo "KeyLessPass app not found or not executable: $APP_BIN" >&2
  exit 1
fi

if [[ "${PRESERVE_EXISTING_KEYLESSPASS:-0}" != "1" ]]; then
  pkill -x KeyLessPass >/dev/null 2>&1 || true
  sleep 0.5
fi

mkdir -p "$OUT_DIR" "$DOCS_OUT_DIR"

SEEDED_HOME="$WORK_DIR/app-home"
EMPTY_HOME="$WORK_DIR/empty-home"
USB_ROOT="$WORK_DIR/usb"
mkdir -p "$SEEDED_HOME" "$EMPTY_HOME" "$USB_ROOT"

(
  cd "$ROOT_DIR/rust_core"
  cargo run --example seed_ui_state -- "$SEEDED_HOME" "$USB_ROOT" >/dev/null
)

keylesspass_window_info() {
  local pid="$1"
  swift - "$pid" <<'SWIFT'
import CoreGraphics
import Foundation

guard CommandLine.arguments.count > 1, let targetPid = Int(CommandLine.arguments[1]) else {
  exit(1)
}

let options = CGWindowListOption(arrayLiteral: [.optionOnScreenOnly, .excludeDesktopElements])
let windows = CGWindowListCopyWindowInfo(options, kCGNullWindowID) as? [[String: Any]] ?? []

for window in windows {
  let ownerPid = window[kCGWindowOwnerPID as String] as? Int ?? -1
  let layer = window[kCGWindowLayer as String] as? Int ?? -1
  let windowId = window[kCGWindowNumber as String] as? Int ?? 0
  let alpha = window[kCGWindowAlpha as String] as? Double ?? 0
  if ownerPid == targetPid && layer == 0 && windowId > 0 && alpha > 0 {
    let bounds = window[kCGWindowBounds as String] as? [String: Any] ?? [:]
    let x = bounds["X"] as? Double ?? 0
    let y = bounds["Y"] as? Double ?? 0
    let width = bounds["Width"] as? Double ?? 0
    let height = bounds["Height"] as? Double ?? 0
    let display = CGMainDisplayID()
    let scale = Double(CGDisplayPixelsWide(display)) / Double(CGDisplayBounds(display).width)
    print("\(windowId),\(x),\(y),\(width),\(height),\(scale)")
    exit(0)
  }
}

exit(1)
SWIFT
}

wait_for_window_info() {
  local pid="$1"
  local info=""
  for _ in {1..60}; do
    info="$(keylesspass_window_info "$pid" 2>/dev/null || true)"
    if [[ -n "$info" ]]; then
      echo "$info"
      return 0
    fi
    sleep 0.25
  done
  return 1
}

capture_window() {
  local pid="$1"
  local output="$2"
  local info window_id x y width height scale full crop_x crop_y crop_width crop_height
  info="$(wait_for_window_info "$pid")"
  IFS=, read -r window_id x y width height scale <<<"$info"

  if screencapture -x -l "$window_id" "$output" >/dev/null 2>&1 && [[ -s "$output" ]]; then
    return 0
  fi

  full="$WORK_DIR/full-$pid.png"
  screencapture -x "$full"
  crop_x="$(awk -v v="$x" -v s="$scale" 'BEGIN { printf "%d", v * s }')"
  crop_y="$(awk -v v="$y" -v s="$scale" 'BEGIN { printf "%d", v * s }')"
  crop_width="$(awk -v v="$width" -v s="$scale" 'BEGIN { printf "%d", v * s }')"
  crop_height="$(awk -v v="$height" -v s="$scale" 'BEGIN { printf "%d", v * s }')"
  sips --cropToHeightWidth "$crop_height" "$crop_width" --cropOffset "$crop_y" "$crop_x" "$full" --out "$output" >/dev/null
}

launch_and_capture() {
  local section="$1"
  local home="$2"
  local output="$3"

  if [[ "${PRESERVE_EXISTING_KEYLESSPASS:-0}" != "1" ]]; then
    pkill -x KeyLessPass >/dev/null 2>&1 || true
    sleep 0.5
  fi
  KEYLESSPASS_HOME="$home" KEYLESSPASS_START_SECTION="$section" KEYLESSPASS_LOCALE="$SCREENSHOT_LOCALE" "$APP_BIN" >/dev/null 2>&1 &
  CURRENT_PID=$!
  sleep 3
  capture_window "$CURRENT_PID" "$output"
  kill "$CURRENT_PID" >/dev/null 2>&1 || true
  wait "$CURRENT_PID" >/dev/null 2>&1 || true
  CURRENT_PID=""
}

launch_and_capture setup "$EMPTY_HOME" "$OUT_DIR/01-enrollment.png"
launch_and_capture records "$SEEDED_HOME" "$OUT_DIR/02-records.png"
launch_and_capture derive "$SEEDED_HOME" "$OUT_DIR/03-derive-password.png"
launch_and_capture rotation "$SEEDED_HOME" "$OUT_DIR/04-rotation.png"
launch_and_capture usb "$SEEDED_HOME" "$OUT_DIR/05-usb-recovery.png"

cp "$OUT_DIR/01-enrollment.png" "$DOCS_OUT_DIR/enrollment.png"
cp "$OUT_DIR/02-records.png" "$DOCS_OUT_DIR/cdr_list.png"
cp "$OUT_DIR/03-derive-password.png" "$DOCS_OUT_DIR/derive_password.png"
cp "$OUT_DIR/04-rotation.png" "$DOCS_OUT_DIR/rotation.png"
cp "$OUT_DIR/05-usb-recovery.png" "$DOCS_OUT_DIR/usb_recovery.png"

echo "Captured real macOS app screenshots into $OUT_DIR"
