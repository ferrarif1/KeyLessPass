#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT/flutter_app"

flutter config --enable-macos-desktop --enable-windows-desktop --enable-linux-desktop
flutter create --platforms=macos,windows,linux .
flutter pub get
