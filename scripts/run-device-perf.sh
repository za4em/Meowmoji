#!/usr/bin/env bash
# Run from the repo root.
# Usage: scripts/run-device-perf.sh

set -euo pipefail

stamp="$(date -u +%Y%m%dT%H%M%SZ)"
mkdir -p out
(cd android && ./gradlew :benchmark:assembleRelease)
adb install -r android/benchmark/build/outputs/apk/release/benchmark-release.apk
adb shell am start -W -n com.litert.benchmark/.MainActivity
adb pull /storage/emulated/0/Android/data/com.litert.benchmark/files/meowmoji-perf.txt \
  "out/meowmoji-perf-${stamp}.txt"
