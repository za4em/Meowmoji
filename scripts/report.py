#!/usr/bin/env python3
"""Copy result lines from out/ into benchmark/runs/ and write benchmark/report/index.html."""

import html
import re
from pathlib import Path

STAMP = re.compile(r"(\d{8}T\d{6}Z)")
CLOCK = re.compile(r"(\d{4})(\d{2})(\d{2})T(\d{2})(\d{2})")
DEVICE = re.compile(r"^(.+)-en-portrait$")
OUT = Path("out")
RUNS = Path("benchmark/runs")
PAGE = Path("benchmark/report/index.html")
CHIPS = ("CPU", "GPU", "NPU")
METRICS = (
    ("Typical match", "median of 40"),
    ("Since last run", "same phone and chip"),
    ("First match", "includes compile"),
    ("Slow matches", "95% finished within"),
    ("Memory left", "app, then native heap"),
)


def main() -> int:
    copy_results(OUT)
    PAGE.parent.mkdir(parents=True, exist_ok=True)
    PAGE.write_text(render(load_runs(RUNS)))
    print(PAGE)
    return 0


def copy_results(src: Path) -> None:
    if not src.is_dir():
        return
    for path in sorted(src.rglob("*")):
        if not path.is_file():
            continue
        if path.name != "meowmoji-perf.txt" and not path.name.startswith("meowmoji-perf-"):
            continue
        target = kept_path(path)
        if target is None:
            continue
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(path.read_bytes())


def kept_path(path: Path) -> Path | None:
    stamp = stamp_of(path)
    if not stamp:
        return None
    for part in path.parts:
        match = DEVICE.match(part)
        if match:
            return RUNS / stamp / f"{match.group(1)}.txt"
    return RUNS / f"{stamp}.txt"


def load_runs(root: Path) -> list[dict]:
    runs = []
    for path in root.rglob("*"):
        if not path.is_file():
            continue
        if path.suffix != ".txt":
            continue
        runs.append({"stamp": stamp_of(path), "records": parse(path.read_text(errors="replace"))})
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


def render(runs: list[dict]) -> str:
    body = '<p class="empty">No runs in out/.</p>' if not runs else table(runs)
    return f"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Meowmoji runs</title>
<link rel="stylesheet" href="report.css">
</head>
<body>
<main>
  <h1>Meowmoji runs</h1>
  {body}
  <dl class="intro">
    <div><dt>Typical match</dt><dd>Median time of 40 matches of “cat”, after 5 warmup matches. The number under it is how sure the model is.</dd></div>
    <div><dt>Since last run</dt><dd>Change in that typical time from the previous run of the same phone and chip.</dd></div>
    <div><dt>First match</dt><dd>Time of the first match, including compiling the model.</dd></div>
    <div><dt>Slow matches</dt><dd>Time within which 95% of the timed matches finished.</dd></div>
    <div><dt>Memory left</dt><dd>Memory still held after close. App is the whole process. Native heap is the C memory.</dd></div>
  </dl>
</main>
</body>
</html>
"""


def table(runs: list[dict]) -> str:
    return f'<div class="table-wrap"><table>{head()}<tbody>{"".join(rows(run) for run in runs)}</tbody></table></div>'


def head() -> str:
    names = "".join(f"<th>{esc(title)}</th>" for title in ("Device", "Chip", "Accelerator"))
    names += "".join(
        f'<th class="metric">{esc(name)}<small>{esc(hint)}</small></th>' for name, hint in METRICS
    )
    return f"<thead><tr>{names}</tr></thead>"


def rows(run: dict) -> str:
    records = sorted(run["records"], key=lambda record: rank(record.get("accelerator", "")))
    if not records:
        cells = device_cell(run, 1) + chip_cell(run, 1) + '<td class="accelerator">—</td>'
        cells += '<td class="num">—</td>' * len(METRICS)
        return f'<tr class="run">{cells}</tr>'
    lines = []
    for index, record in enumerate(records):
        cells = ""
        if index == 0:
            span = len(records)
            cells = device_cell(run, span) + chip_cell(run, span)
        mark = ' class="run"' if index == 0 else ""
        lines.append(f"<tr{mark}>{cells}{accelerator_cell(record)}{record_cells(record)}</tr>")
    return "".join(lines)


def device_cell(run: dict, span: int) -> str:
    record = run["records"][0] if run["records"] else {}
    name = label(record.get("device", "")) or "Unknown phone"
    return f'<td class="device" rowspan="{span}"><strong>{esc(name)}</strong><small>{esc(clock(run["stamp"]))}</small></td>'


def chip_cell(run: dict, span: int) -> str:
    record = run["records"][0] if run["records"] else {}
    return f'<td class="chip" rowspan="{span}">{esc(label(record.get("soc", "")) or "—")}</td>'


def accelerator_cell(record: dict) -> str:
    return f'<td class="accelerator">{esc(record.get("accelerator") or "—")}</td>'


def record_cells(record: dict) -> str:
    if record.get("error"):
        return fail_cell(label(record["error"])) + num("—") * 3 + memory_cell(record)
    return (
        time_cell(record)
        + change_cell(record.get("speedChangeMs"))
        + num(ms(record.get("firstMatchMs")))
        + num(ms(record.get("steadyP95Ms")))
        + memory_cell(record)
    )


def time_cell(record: dict) -> str:
    score = f'confidence {record["score"]}' if record.get("score") else ""
    return f'<td class="num"><strong>{esc(ms(record.get("steadyMedianMs")))}</strong><small class="sub">{esc(score)}</small></td>'


def fail_cell(message: str) -> str:
    short = message.split(": ")[-1]
    return (
        f'<td class="num"><strong>No time</strong>'
        f'<small class="sub fail" title="{esc(message)}">{esc(short)}</small></td>'
    )


def change_cell(value: str | None) -> str:
    change = speed_change(value)
    if change is None:
        return num("—")
    text, kind = change
    return f'<td class="num"><span class="change {kind}">{esc(text)}</span></td>'


def memory_cell(record: dict) -> str:
    lines = "".join(
        f"<div>{esc(memory_line(record, prefix, name))}</div>"
        for prefix, name in (("pss", "app"), ("native", "native heap"))
    )
    return f'<td class="mem">{lines}</td>'


def memory_line(record: dict, prefix: str, name: str) -> str:
    try:
        before = float(record[f"{prefix}BeforeKb"])
        closed = float(record[f"{prefix}AfterCloseKb"])
    except (KeyError, TypeError, ValueError):
        return f"— {name}"
    delta = closed - before
    if abs(delta) < 50:
        return f"none {name}"
    amount = f"{abs(delta) / 1024:.1f} MB {name}"
    return amount if delta > 0 else f"{amount} freed"


def num(text: str) -> str:
    return f'<td class="num">{esc(text)}</td>'


def speed_change(value: str | None) -> tuple[str, str] | None:
    if value is None or value == "":
        return None
    try:
        amount = float(value)
    except ValueError:
        return None
    if abs(amount) < 0.005:
        return ("Same", "same")
    kind = "faster" if amount < 0 else "slower"
    return (f"{abs(amount):.2f} ms {kind}", kind)


def rank(name: str) -> int:
    try:
        return CHIPS.index(name)
    except ValueError:
        return len(CHIPS)


def ms(value: str | None) -> str:
    return "—" if value is None or value == "" else f"{value} ms"


def label(value: str) -> str:
    return (value or "").replace("_", " ")


def clock(stamp: str) -> str:
    match = CLOCK.match(stamp or "")
    if not match:
        return stamp or ""
    year, month, day, hour, minute = match.groups()
    return f"{year}-{month}-{day} {hour}:{minute} UTC"


def esc(text: str) -> str:
    return html.escape(text, quote=True)


if __name__ == "__main__":
    raise SystemExit(main())
