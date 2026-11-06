#!/usr/bin/env bash
# Run from the repo root.
# Usage: scripts/run-firebase-perf.sh model=redfin,version=33

set -euo pipefail

device="$1"
stamp="$(date -u +%Y%m%dT%H%M%SZ)"
results_dir="meowmoji-${stamp}"
mkdir -p out
(cd android && ./gradlew :benchmark:assembleRelease)
apk="android/benchmark/build/outputs/apk/release/benchmark-release.apk"

gcloud firebase test android run \
  --type game-loop \
  --app "$apk" \
  --device "${device},locale=en,orientation=portrait" \
  --timeout 5m \
  --directories-to-pull /sdcard/Android/data/com.litert.benchmark/files \
  --results-dir "$results_dir" \
  2>&1 | tee "out/firebase-${stamp}.log"

bucket_path="$(sed -n 's/.*storage\/browser\/\([^]]*\).*/\1/p' "out/firebase-${stamp}.log" | head -1)"
bucket_path="${bucket_path%/}"
gcloud storage cp -r "gs://${bucket_path}" "out/firebase-${stamp}"
