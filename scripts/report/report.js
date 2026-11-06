const list = document.querySelector("#runs")
const runs = window.runs || []
const chipOrder = ["CPU", "GPU", "NPU"]
const metrics = [
  ["Typical match", "median of 40"],
  ["Since last run", "same phone and chip"],
  ["First match", "includes compile"],
  ["Slow matches", "95% finished within"],
  ["Memory left", "app, then native heap"],
]

if (runs.length === 0) {
  list.innerHTML = '<p class="empty">No runs in out/.</p>'
} else {
  list.append(table(runs))
}

function table(runs) {
  const wrap = document.createElement("div")
  const table = document.createElement("table")
  wrap.className = "table-wrap"
  table.append(head(), body(runs))
  wrap.append(table)
  return wrap
}

function head() {
  const head = document.createElement("thead")
  const names = document.createElement("tr")
  for (const title of ["Device", "Accelerator"]) {
    const cell = document.createElement("th")
    cell.textContent = title
    names.append(cell)
  }
  for (const [name, hint] of metrics) {
    const cell = document.createElement("th")
    const note = document.createElement("small")
    cell.className = "metric"
    cell.append(name)
    note.textContent = hint
    cell.append(note)
    names.append(cell)
  }
  head.append(names)
  return head
}

function body(runs) {
  const body = document.createElement("tbody")
  for (const run of runs) body.append(...rows(run))
  return body
}

function rows(run) {
  const records = [...run.records].sort((left, right) => rank(left.accelerator) - rank(right.accelerator))
  if (records.length === 0) {
    const tr = document.createElement("tr")
    tr.className = "run"
    tr.append(deviceCell(run), textCell("—", "accelerator"), ...metrics.map(() => textCell("—", "num")))
    return [tr]
  }
  return records.map((record, index) => {
    const tr = document.createElement("tr")
    if (index === 0) {
      const device = deviceCell(run)
      device.rowSpan = records.length
      tr.className = "run"
      tr.append(device)
    }
    tr.append(acceleratorCell(record), ...recordCells(record))
    return tr
  })
}

function acceleratorCell(record) {
  const cell = document.createElement("td")
  cell.className = "accelerator"
  cell.textContent = record.accelerator || "—"
  return cell
}

function deviceCell(run) {
  const cell = document.createElement("td")
  const name = document.createElement("strong")
  const detail = document.createElement("small")
  const record = run.records[0] || {}
  cell.className = "device"
  name.textContent = [label(record.device) || "Unknown phone", label(record.soc)].filter(Boolean).join(" · ")
  detail.textContent = clock(run.stamp)
  cell.append(name, detail)
  return cell
}

function recordCells(record) {
  if (!record) return metrics.map(() => textCell("—", "num"))
  if (record.error) {
    return [
      failCell(label(record.error)),
      textCell("—", "num"),
      textCell("—", "num"),
      textCell("—", "num"),
      memoryCell(record),
    ]
  }
  return [
    timeCell(record),
    changeCell(record.speedChangeMs),
    textCell(ms(record.firstMatchMs), "num"),
    textCell(ms(record.steadyP95Ms), "num"),
    memoryCell(record),
  ]
}

function timeCell(record) {
  const cell = document.createElement("td")
  const value = document.createElement("strong")
  const note = document.createElement("small")
  cell.className = "num"
  value.textContent = ms(record.steadyMedianMs)
  note.className = "sub"
  note.textContent = record.score ? `confidence ${record.score}` : ""
  cell.append(value, note)
  return cell
}

function failCell(message) {
  const cell = document.createElement("td")
  const value = document.createElement("strong")
  const note = document.createElement("small")
  cell.className = "num"
  value.textContent = "No time"
  note.className = "sub fail"
  note.title = message
  note.textContent = message.split(": ").pop()
  cell.append(value, note)
  return cell
}

function changeCell(value) {
  const cell = textCell("—", "num")
  const change = speedChange(value)
  if (!change) return cell
  cell.textContent = ""
  const note = document.createElement("span")
  note.className = `change ${change.kind}`
  note.textContent = change.text
  cell.append(note)
  return cell
}

function memoryCell(record) {
  const cell = document.createElement("td")
  cell.className = "mem"
  cell.append(memoryLine(record, "pss", "app"), memoryLine(record, "native", "native heap"))
  return cell
}

function memoryLine(record, prefix, name) {
  const line = document.createElement("div")
  const before = Number(record[`${prefix}BeforeKb`])
  const closed = Number(record[`${prefix}AfterCloseKb`])
  if (!Number.isFinite(before) || !Number.isFinite(closed)) {
    line.textContent = `— ${name}`
    return line
  }
  const delta = closed - before
  if (Math.abs(delta) < 50) {
    line.textContent = `none ${name}`
    return line
  }
  const amount = `${(Math.abs(delta) / 1024).toFixed(1)} MB ${name}`
  line.textContent = delta > 0 ? amount : `${amount} freed`
  return line
}

function textCell(text, className) {
  const cell = document.createElement("td")
  cell.className = className
  cell.textContent = text
  return cell
}

function speedChange(value) {
  if (value == null || value === "") return null
  const amount = Number(value)
  if (!Number.isFinite(amount)) return null
  if (Math.abs(amount) < 0.005) return { text: "Same", kind: "same" }
  const ms = Math.abs(amount).toFixed(2)
  return amount < 0
    ? { text: `${ms} ms faster`, kind: "faster" }
    : { text: `${ms} ms slower`, kind: "slower" }
}

function rank(name) {
  const index = chipOrder.indexOf(name)
  return index < 0 ? chipOrder.length : index
}

function ms(value) {
  return value == null || value === "" ? "—" : `${value} ms`
}

function label(value) {
  return (value || "").replaceAll("_", " ")
}

function clock(stamp) {
  const match = /^(\d{4})(\d{2})(\d{2})T(\d{2})(\d{2})/.exec(stamp || "")
  if (!match) return stamp || ""
  const [, year, month, day, hour, minute] = match
  return `${year}-${month}-${day} ${hour}:${minute} UTC`
}
