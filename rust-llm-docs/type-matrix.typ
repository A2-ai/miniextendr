// miniextendr type matrix: the two tables of generated/type-matrix.json.
//
//   typst compile rust-llm-docs/type-matrix.typ rust-llm-docs/generated/type-matrix.pdf
//   typst compile rust-llm-docs/type-matrix.typ 'rust-llm-docs/generated/type-matrix-{p}.svg'
//
// `just type-matrix` builds the JSON first and runs both. Another data file:
// `--input data=<path relative to this file>`.

#let data = json(sys.inputs.at("data", default: "generated/type-matrix.json"))

#set page(width: auto, height: auto, margin: 18pt)
#set text(size: 7.5pt)

#let palette = (
  "yes": (fill: rgb("#d7f0de"), ink: rgb("#12592b")),
  "partial": (fill: rgb("#fcebc2"), ink: rgb("#6f4a00")),
  "gap": (fill: rgb("#f9d5d1"), ink: rgb("#94190f")),
  "no": (fill: rgb("#e9e9e7"), ink: rgb("#55585c")),
  "n/a": (fill: none, ink: rgb("#a3a6aa")),
)
#let glyphs = data.legend.glyphs
#let muted = rgb("#5f6368")
#let rule-color = rgb("#d9d9d6")

#let chip(status, label) = {
  let p = palette.at(status)
  box(
    fill: p.fill,
    inset: (x: 3pt, y: 1.5pt),
    radius: 5pt,
    text(fill: p.ink, weight: if status == "gap" { "bold" } else { "regular" })[#glyphs.at(status) #label],
  )
}

#let short-crate(name) = name.replace("miniextendr-", "")

#let header-block = {
  text(size: 16pt, weight: "bold")[miniextendr type matrix]
  h(10pt)
  text(fill: muted)[commit #raw(data.commit.slice(0, 10)), #data.generated_at]
  if data.dirty { text(fill: palette.gap.ink)[ (uncommitted changes)] }
  v(4pt)
  block(width: 980pt)[
    #for status in ("yes", "partial", "gap", "no", "n/a") [
      #chip(status, status) #data.legend.statuses.at(status) #h(10pt)
    ]
  ]
}

#let legend-block = {
  let labels = (:)
  for c in data.cell_columns { labels.insert(c.id, c.label) }
  block(width: 980pt, inset: (top: 6pt))[
    #set par(spacing: 4pt)
    #for (name, rule) in data.legend.rows [
      *#name:* #text(fill: muted, rule) \
    ]
    #v(3pt)
    #for (id, rule) in data.legend.columns [
      *#labels.at(id):* #text(fill: muted, rule) \
    ]
  ]
}

#let matrix-table(tab) = {
  let info = if tab.id == "types" { ("crate", "kind", "feature") } else { ("origin", "kind", "feature") }
  let cols = data.cell_columns
  let groups = ()
  for c in cols {
    if groups.len() == 0 or groups.last().at(0) != c.group {
      groups.push((c.group, 1))
    } else {
      let last = groups.pop()
      groups.push((last.at(0), last.at(1) + 1))
    }
  }
  let counts(col) = {
    let s = tab.summary.at(col)
    ("yes", "partial", "gap", "no")
      .filter(st => s.at(st, default: 0) > 0)
      .map(st => str(s.at(st)) + glyphs.at(st))
      .join(" ")
  }
  let name-of(row) = if tab.id == "types" {
    row.path.split("::").slice(1).join("::")
  } else { row.name }
  let cells = ()
  for row in tab.rows {
    cells.push(raw(name-of(row)))
    for i in info {
      let v = row.info.at(i)
      cells.push(text(fill: muted, if i == "crate" { short-crate(v) } else { v }))
    }
    for c in cols {
      let cell = row.cells.at(c.id)
      cells.push(chip(cell.status, if cell.status == "n/a" { "" } else { cell.label }))
    }
  }
  heading(level: 1, outlined: false)[#tab.title (#tab.rows.len() rows)]
  table(
    columns: 1 + info.len() + cols.len(),
    stroke: (x, y) => (bottom: 0.4pt + rule-color),
    inset: (x: 3pt, y: 2pt),
    align: left + horizon,
    table.header(
      table.cell(colspan: 1 + info.len())[],
      ..groups.map(g => table.cell(colspan: g.at(1), text(fill: muted, size: 6.5pt, upper(g.at(0))))),
      [*#if tab.id == "types" [type] else [shape]*],
      ..info.map(i => [*#i*]),
      ..cols.map(c => [*#c.label* \ #text(fill: muted, size: 6pt, counts(c.id))]),
    ),
    ..cells,
  )
}

#header-block
#legend-block
#for (i, tab) in data.tabs.enumerate() {
  if i > 0 { pagebreak() }
  matrix-table(tab)
}
