#!/usr/bin/env python3
"""Read out/ and write scripts/report/runs.js. Run from the repo root."""

import json
import re
from pathlib import Path

STAMP = re.compile(r"(\d{8}T\d{6}Z)")
OUT = Path("out")
DATA = Path("scripts/report/runs.js")


def main() -> int:
    OUT.mkdir(exist_ok=True)
    runs = load_runs(OUT)
    DATA.write_text("window.runs = " + json.dumps(runs) + "\n")
    print(DATA)
    return 0


def load_runs(root: Path) -> list[dict]:
    runs = []
    for path in root.rglob("*"):
        if not path.is_file():
            continue
        if path.name != "meowmoji-perf.txt" and not path.name.startswith("meowmoji-perf-"):
            continue
        runs.append(
            {
                "stamp": stamp_of(path),
                "source": "Firebase" if any(part.startswith("firebase-") for part in path.parts) else "Phone",
                "records": parse(path.read_text(errors="replace")),
            }
        )
    runs.sort(key=lambda run: run["stamp"], reverse=True)
    attach_speed_change(runs)
    return runs


def stamp_of(path: Path) -> str:
    for part in (path.name, *path.parts):
        match = STAMP.search(part)
        if match:
            return match.group(1)
    return ""


def parse(text: str) -> list[dict[str, str]]:
    records = []
    seen = set()
    for line in text.splitlines():
        start = line.find("accelerator=")
        if start < 0 or "device=" not in line:
            continue
        body = line[start:].strip()
        if body in seen:
            continue
        seen.add(body)
        records.append(dict(pair.split("=", 1) for pair in body.split(" ") if "=" in pair))
    return records


def attach_speed_change(runs: list[dict]) -> None:
    previous: dict[tuple[str, str], str] = {}
    for run in sorted(runs, key=lambda item: item["stamp"]):
        for record in run["records"]:
            key = (record.get("device", ""), record.get("accelerator", ""))
            current = record.get("steadyMedianMs")
            earlier = previous.get(key)
            if current is not None and earlier is not None:
                record["speedChangeMs"] = f"{float(current) - float(earlier):+.2f}"
            if current is not None:
                previous[key] = current


if __name__ == "__main__":
    raise SystemExit(main())
