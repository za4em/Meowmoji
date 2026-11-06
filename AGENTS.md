# Running the benchmark

Run every command from the repo root. 

The app is `com.litert.benchmark`. It is a release build, arm64-v8a only. It times a match of "cat" on CPU, then on GPU, and writes one line per chip. A success line means the result was the cat emoji. An `error=` line means that chip did not produce a time.

## Connected phone

```bash
scripts/run-device-perf.sh
```

This builds `:benchmark:assembleRelease`, installs the APK, starts `com.litert.benchmark/.MainActivity`, and pulls the file to `out/meowmoji-perf-<stamp>.txt`. The stamp is UTC `YYYYMMDDTHHMMSSZ`. Each run is a new file. The phone overwrites its own copy at:

```text
/storage/emulated/0/Android/data/com.litert.benchmark/files/meowmoji-perf.txt
```

`adb` must already see the phone. `am start -W` can print success when the process died before writing the file. Trust the pulled file.

## Firebase Test Lab

The project is `meowmoji`. `gcloud` must be logged in (`gcloud auth list`). These APIs must be enabled: `testing.googleapis.com` and `toolresults.googleapis.com`. Billing must be on. Without billing the lab allows about five physical phones a day, and a larger matrix is rejected before it starts.

One phone, using the script:

```bash
scripts/run-firebase-perf.sh model=redfin,version=33
```

The argument is `model=<id>,version=<api>`. The script adds `locale=en,orientation=portrait`, runs a game-loop test, and downloads the lab directory to `out/firebase-<stamp>/`.

Several phones are one `gcloud` command with one `--device` per phone. Do not call the script once per phone. That rebuilds and uploads the APK every time.

```bash
stamp="$(date -u +%Y%m%dT%H%M%SZ)"
apk="android/benchmark/build/outputs/apk/release/benchmark-release.apk"
(cd android && ./gradlew :benchmark:assembleRelease)
gcloud firebase test android run \
  --type game-loop \
  --app "$apk" \
  --device model=a05s,version=35,locale=en,orientation=portrait \
  --device model=redfin,version=30,locale=en,orientation=portrait \
  --timeout 2m \
  --directories-to-pull /sdcard/Android/data/com.litert.benchmark/files \
  --results-dir "meowmoji-${stamp}"
```

`--timeout 2m` is how long the benchmark may run after the phone is ready. The match loop itself is a few seconds, including the slowest GPU compile we have seen. The limit does not cover waiting in line for a free phone.

List real phones and their API levels with `gcloud firebase test android models list`. Before adding a phone, check it with `gcloud firebase test android models describe <id>` and keep it only when `supportedAbis` contains `arm64-v8a` and `form` is `PHYSICAL`. A 32-bit phone cannot install this APK. A virtual device is not a GPU measurement.

Use the highest listed API for that model. Known 32-bit phones that cannot run this APK include Galaxy A10 (`a10`), Galaxy A13 (`a13`), moto e20 (`aruba`), and TECNO POP 7 (`TECNO-BF6`).

The lab prints a storage URL like `https://console.developers.google.com/storage/browser/<bucket>/meowmoji-<stamp>/`. The result file for each phone is:

```text
gs://<bucket>/meowmoji-<stamp>/<model>-<version>-en-portrait/artifacts/sdcard/Android/data/com.litert.benchmark/files/meowmoji-perf.txt
```

Pull the text files, not the whole tree. A full `gcloud storage cp -r` also copies `video.mp4`, `logcat`, and the APK, and that copy can fail a hash check after the text files are already in the bucket.

```bash
gcloud storage cp \
  "gs://<bucket>/meowmoji-<stamp>/<model>-<version>-en-portrait/artifacts/sdcard/Android/data/com.litert.benchmark/files/meowmoji-perf.txt" \
  "out/firebase-<stamp>/<model>-<version>-en-portrait/meowmoji-perf.txt"
```

Finished phones upload that file while other phones are still queued. Read those files instead of waiting for a phone that stays `Pending`. One execution can die with `Internal System Error` before the app starts. Say which phone failed. Do not treat that as a benchmark result.

## Report

```bash
python3 scripts/report.py
```

`out/` is the download. The script copies every `meowmoji-perf.txt` and `meowmoji-perf-*.txt` into `benchmark/runs/`. A phone file lands at `benchmark/runs/<stamp>.txt`. A lab phone lands at `benchmark/runs/<stamp>/<model>-<version>.txt`. The script then writes `benchmark/report/index.html` from those copies. Styles are in `report.css` beside the page. Open that file. The script does not open a browser.

Each phone run is one group of rows. CPU is above GPU. The columns are:

- **Typical match** — median of 40 matches after 5 warmup matches. The number under it is the model's confidence.
- **Since last run** — change in that median against the previous run of the same phone and chip. Faster is green. Slower is brown.
- **First match** — the first match, which includes compiling the model.
- **Slow matches** — the time 95% of the timed matches finished within.
- **Memory left** — what `close` did not free. App is the whole process. Native heap is the C memory.

A GPU row that says **No time** failed to compile. The text under it is the error, and the memory cells are still real.

The word-vector gather cannot run on this GPU. A GPU compile asks LiteRT for GPU and CPU, so the refused ops stay on the CPU. The perf file does not say how many ops ran on each chip. That sentence is only in logcat (`7 operations will run on the GPU...`). The lab stores logcat next to the pulled files. Do not parse it into the report unless asked. The app cannot read logcat itself.
