#!/usr/bin/env python3
"""
Render ``type-matrix.json`` as one self-contained interactive HTML page.

The page embeds the JSON, its CSS and its JavaScript inline: no external
scripts, stylesheets or fonts. Two tabs (miniextendr types, converted types),
a sticky header row and first column, sortable columns, filters, per-column
counts, and a side panel with each cell's evidence, linked to the sources at
the matrix's commit.

Usage:
    python3 rustdoc_type_matrix_html.py generated/type-matrix.json generated/type-matrix.html
"""

from __future__ import annotations

import html
import json
import sys
from pathlib import Path

CSS = r"""
:root {
  color-scheme: light;
  --bg: #fbfbfa;
  --surface: #ffffff;
  --surface-2: #f2f2f0;
  --text: #1d1f21;
  --muted: #5f6368;
  --border: #d9d9d6;
  --accent: #2b5fb4;
  --yes-bg: #d7f0de; --yes-fg: #12592b;
  --partial-bg: #fcebc2; --partial-fg: #6f4a00;
  --gap-bg: #f9d5d1; --gap-fg: #94190f;
  --no-bg: #e9e9e7; --no-fg: #55585c;
  --na-bg: transparent; --na-fg: #a3a6aa;
  --shadow: 0 8px 28px rgba(0, 0, 0, 0.18);
}
@media (prefers-color-scheme: dark) {
  :root:not([data-theme="light"]) {
    color-scheme: dark;
    --bg: #15171a;
    --surface: #1c1f23;
    --surface-2: #24282d;
    --text: #e6e7e9;
    --muted: #9aa0a6;
    --border: #353a40;
    --accent: #8ab4f8;
    --yes-bg: #143d24; --yes-fg: #93e2ad;
    --partial-bg: #47380f; --partial-fg: #f3d17c;
    --gap-bg: #4d1b17; --gap-fg: #ffaba3;
    --no-bg: #2b2f34; --no-fg: #a6abb1;
    --na-bg: transparent; --na-fg: #5c6167;
    --shadow: 0 8px 28px rgba(0, 0, 0, 0.55);
  }
}
:root[data-theme="dark"] {
  color-scheme: dark;
  --bg: #15171a;
  --surface: #1c1f23;
  --surface-2: #24282d;
  --text: #e6e7e9;
  --muted: #9aa0a6;
  --border: #353a40;
  --accent: #8ab4f8;
  --yes-bg: #143d24; --yes-fg: #93e2ad;
  --partial-bg: #47380f; --partial-fg: #f3d17c;
  --gap-bg: #4d1b17; --gap-fg: #ffaba3;
  --no-bg: #2b2f34; --no-fg: #a6abb1;
  --na-bg: transparent; --na-fg: #5c6167;
  --shadow: 0 8px 28px rgba(0, 0, 0, 0.55);
}
* { box-sizing: border-box; }
html, body { margin: 0; }
body {
  background: var(--bg);
  color: var(--text);
  font: 13px/1.4 system-ui, -apple-system, "Segoe UI", Roboto, sans-serif;
  overflow-x: hidden;
}
main {
  display: flex;
  flex-direction: column;
  height: 100dvh;
  padding: 12px 16px 16px;
  gap: 10px;
}
header { flex: none; display: flex; flex-wrap: wrap; align-items: baseline; gap: 6px 16px; }
h1 { font-size: 18px; margin: 0; font-weight: 650; }
.meta { color: var(--muted); }
.meta a { color: var(--accent); }
.dirty { color: var(--gap-fg); background: var(--gap-bg); padding: 0 6px; border-radius: 4px; }
details.legend { flex: none; background: var(--surface); border: 1px solid var(--border); border-radius: 8px; padding: 6px 10px; }
details.legend summary { cursor: pointer; font-weight: 600; }
.legend-statuses { display: flex; flex-wrap: wrap; gap: 6px 14px; margin: 8px 0; }
.legend dl { display: grid; grid-template-columns: max-content 1fr; gap: 4px 12px; margin: 8px 0 4px; max-height: 40vh; overflow: auto; }
.legend dt { font-weight: 600; }
.legend dd { margin: 0; color: var(--muted); }
.controls { flex: none; display: flex; flex-wrap: wrap; gap: 8px; align-items: center; }
.tabs { display: flex; gap: 4px; }
.tabs button {
  font: inherit; padding: 5px 12px; border: 1px solid var(--border); border-radius: 6px;
  background: var(--surface); color: var(--text); cursor: pointer;
}
.tabs button[aria-selected="true"] { background: var(--accent); border-color: var(--accent); color: var(--bg); font-weight: 600; }
.controls select, .controls input[type="search"] {
  font: inherit; padding: 4px 6px; border: 1px solid var(--border); border-radius: 6px;
  background: var(--surface); color: var(--text); max-width: 100%;
}
.controls input[type="search"] { width: 16em; }
.controls label { display: inline-flex; gap: 4px; align-items: center; color: var(--muted); }
.count { color: var(--muted); }
.table-wrap {
  flex: 1 1 auto; min-height: 240px; overflow: auto; max-width: 100%;
  border: 1px solid var(--border); border-radius: 8px; background: var(--surface);
}
table { border-collapse: separate; border-spacing: 0; }
th, td { border-bottom: 1px solid var(--border); padding: 3px 6px; white-space: nowrap; }
thead th {
  position: sticky; top: 0; z-index: 2; background: var(--surface-2); text-align: left;
  vertical-align: bottom; cursor: pointer; user-select: none; font-weight: 600;
}
thead th:focus-visible, .chip:focus-visible, .rowname:focus-visible { outline: 2px solid var(--accent); outline-offset: -2px; }
th .grp { display: block; font-size: 10px; text-transform: uppercase; letter-spacing: 0.04em; color: var(--muted); font-weight: 500; }
th .sum { display: block; font-size: 10px; color: var(--muted); font-weight: 400; }
th[aria-sort="ascending"] .lbl::after { content: " \25B2"; font-size: 9px; }
th[aria-sort="descending"] .lbl::after { content: " \25BC"; font-size: 9px; }
th.first, td.first { position: sticky; left: 0; z-index: 1; background: var(--surface); border-right: 1px solid var(--border); }
thead th.first { z-index: 3; background: var(--surface-2); }
td.first { max-width: 22em; overflow: hidden; text-overflow: ellipsis; }
td.info { color: var(--muted); max-width: 14em; overflow: hidden; text-overflow: ellipsis; }
th.grp-start, td.grp-start { border-left: 2px solid var(--border); }
tbody tr:hover td { background: var(--surface-2); }
.rowname { font: 12px ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; color: var(--text); background: none; border: 0; padding: 0; cursor: pointer; text-align: left; }
.chip {
  display: inline-flex; align-items: center; gap: 3px; min-width: 4.2em; justify-content: flex-start;
  font: 11px/1.5 system-ui, sans-serif; padding: 0 6px; border-radius: 10px; border: 0; cursor: pointer;
}
.chip .g { font-size: 10px; }
.s-yes { background: var(--yes-bg); color: var(--yes-fg); }
.s-partial { background: var(--partial-bg); color: var(--partial-fg); }
.s-gap { background: var(--gap-bg); color: var(--gap-fg); font-weight: 600; }
.s-no { background: var(--no-bg); color: var(--no-fg); }
.s-na { background: var(--na-bg); color: var(--na-fg); }
aside {
  position: fixed; top: 0; right: 0; bottom: 0; width: min(440px, 100vw); z-index: 10;
  background: var(--surface); border-left: 1px solid var(--border); box-shadow: var(--shadow);
  padding: 14px 16px; overflow: auto; transform: translateX(105%); transition: transform 0.15s ease;
}
aside.open { transform: none; }
aside h2 { font-size: 15px; margin: 0 28px 4px 0; word-break: break-word; }
aside h3 { font-size: 13px; margin: 14px 0 4px; }
aside .close { position: absolute; top: 10px; right: 12px; font: inherit; font-size: 18px; background: none; border: 0; color: var(--muted); cursor: pointer; }
aside .rule { color: var(--muted); margin: 6px 0; }
aside ul { padding-left: 18px; margin: 4px 0; }
aside li { margin: 3px 0; word-break: break-word; }
aside code, aside .mono { font: 12px ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; }
aside a { color: var(--accent); }
.empty { padding: 24px; color: var(--muted); }
@media (max-width: 640px) {
  main { padding: 10px 16px 12px; }
  .controls input[type="search"] { width: 100%; }
  td.first { max-width: 10em; }
}
"""

JS = r"""
(function () {
  "use strict";
  var data = JSON.parse(document.getElementById("matrix-data").textContent);
  var ORDER = { "yes": 0, "partial": 1, "gap": 2, "no": 3, "n/a": 4 };
  var GLYPH = data.legend.glyphs;
  var STATUSES = ["yes", "partial", "gap", "no", "n/a"];
  var state = { tab: 0, sortCol: null, sortDir: 1, filters: {}, search: "", gapsOnly: false };
  var els = {
    tabs: document.getElementById("tabs"),
    filters: document.getElementById("filters"),
    search: document.getElementById("search"),
    gaps: document.getElementById("gaps"),
    count: document.getElementById("count"),
    wrap: document.getElementById("wrap"),
    panel: document.getElementById("panel"),
    panelBody: document.getElementById("panel-body")
  };
  var built = [];

  function cls(status) { return "s-" + (status === "n/a" ? "na" : status); }
  function el(tag, attrs, text) {
    var e = document.createElement(tag);
    if (attrs) for (var k in attrs) e.setAttribute(k, attrs[k]);
    if (text != null) e.textContent = text;
    return e;
  }
  function link(path, line) {
    var url = data.repo_url + "/blob/" + data.commit + "/" + path + (line ? "#L" + line : "");
    var a = el("a", { href: url, target: "_blank", rel: "noopener" }, path + (line ? ":" + line : ""));
    return a;
  }

  function filterColumns(tab) {
    return tab.info_columns.filter(function (c) { return c.id !== "generics" && c.id !== "bounds"; });
  }

  function buildTabs() {
    data.tabs.forEach(function (tab, i) {
      var b = el("button", { role: "tab", "aria-selected": i === 0 ? "true" : "false", type: "button" },
        tab.title + " (" + tab.rows.length + ")");
      b.addEventListener("click", function () { selectTab(i); });
      els.tabs.appendChild(b);
    });
  }

  function buildFilters() {
    var tab = data.tabs[state.tab];
    els.filters.textContent = "";
    state.filters = {};
    filterColumns(tab).forEach(function (col) {
      var values = {};
      tab.rows.forEach(function (r) { values[r.info[col.id]] = true; });
      var sel = el("select", { "aria-label": col.label });
      sel.appendChild(el("option", { value: "" }, "all " + col.label + "s"));
      Object.keys(values).sort().forEach(function (v) { sel.appendChild(el("option", { value: v }, v)); });
      sel.addEventListener("change", function () { state.filters[col.id] = sel.value; applyFilters(); });
      els.filters.appendChild(sel);
    });
  }

  function buildTable(tabIndex) {
    var tab = data.tabs[tabIndex];
    var table = el("table");
    var thead = el("thead");
    var hr = el("tr");
    var columns = [{ id: "name", label: tab.id === "types" ? "type" : "shape", kind: "name" }];
    tab.info_columns.forEach(function (c) { columns.push({ id: c.id, label: c.label, kind: "info" }); });
    var prevGroup = null;
    data.cell_columns.forEach(function (c) {
      columns.push({ id: c.id, label: c.label, kind: "cell", group: c.group, start: c.group !== prevGroup });
      prevGroup = c.group;
    });
    var sums = {};
    columns.forEach(function (c, ci) {
      var th = el("th", { tabindex: "0", scope: "col" });
      if (ci === 0) th.className = "first";
      if (c.start) th.className = "grp-start";
      if (c.kind === "cell") th.appendChild(el("span", { "class": "grp" }, c.start ? c.group : " "));
      th.appendChild(el("span", { "class": "lbl" }, c.label));
      if (c.kind === "cell") {
        var s = el("span", { "class": "sum" });
        sums[c.id] = s;
        th.appendChild(s);
        th.title = data.legend.columns[c.id] || "";
      }
      th.addEventListener("click", function () { sortBy(c, th); });
      th.addEventListener("keydown", function (ev) { if (ev.key === "Enter" || ev.key === " ") { ev.preventDefault(); sortBy(c, th); } });
      c.th = th;
      hr.appendChild(th);
    });
    thead.appendChild(hr);
    table.appendChild(thead);
    var tbody = el("tbody");
    var rows = tab.rows.map(function (r, ri) {
      var tr = el("tr");
      var td = el("td", { "class": "first" });
      var nb = el("button", { "class": "rowname", type: "button", title: r.path }, r.name);
      nb.addEventListener("click", function () { showRow(tab, r); });
      td.appendChild(nb);
      tr.appendChild(td);
      tab.info_columns.forEach(function (c) {
        var v = r.info[c.id] || "";
        tr.appendChild(el("td", { "class": "info", title: v }, v));
      });
      var hasGap = false;
      columns.forEach(function (c) {
        if (c.kind !== "cell") return;
        var cellData = r.cells[c.id];
        if (cellData.status === "gap") hasGap = true;
        var cd = el("td", c.start ? { "class": "grp-start" } : null);
        var b = el("button", {
          "class": "chip " + cls(cellData.status), type: "button",
          "aria-label": c.label + ": " + cellData.status + " " + cellData.label
        });
        b.appendChild(el("span", { "class": "g", "aria-hidden": "true" }, GLYPH[cellData.status]));
        if (cellData.status !== "n/a") b.appendChild(el("span", null, cellData.label));
        b.addEventListener("click", function () { showCell(tab, r, c); });
        cd.appendChild(b);
        tr.appendChild(cd);
      });
      var hay = [r.name, r.path].concat(r.reexports || []);
      tab.info_columns.forEach(function (c) { hay.push(r.info[c.id] || ""); });
      return { tr: tr, row: r, index: ri, hasGap: hasGap, hay: hay.join(" \u0001 ").toLowerCase() };
    });
    rows.forEach(function (x) { tbody.appendChild(x.tr); });
    table.appendChild(tbody);
    return { table: table, tbody: tbody, rows: rows, columns: columns, sums: sums };
  }

  function selectTab(i) {
    state.tab = i;
    state.sortCol = null;
    Array.prototype.forEach.call(els.tabs.children, function (b, j) { b.setAttribute("aria-selected", j === i ? "true" : "false"); });
    if (!built[i]) built[i] = buildTable(i);
    els.wrap.textContent = "";
    els.wrap.appendChild(built[i].table);
    els.wrap.scrollTop = 0;
    els.wrap.scrollLeft = 0;
    buildFilters();
    applyFilters();
  }

  function applyFilters() {
    var b = built[state.tab];
    var q = state.search.trim().toLowerCase();
    var shown = 0;
    var counts = {};
    b.columns.forEach(function (c) { if (c.kind === "cell") counts[c.id] = {}; });
    b.rows.forEach(function (x) {
      var ok = true;
      for (var k in state.filters) {
        if (state.filters[k] && x.row.info[k] !== state.filters[k]) { ok = false; break; }
      }
      if (ok && state.gapsOnly && !x.hasGap) ok = false;
      if (ok && q && x.hay.indexOf(q) < 0) ok = false;
      x.tr.hidden = !ok;
      if (ok) {
        shown++;
        for (var cid in counts) {
          var st = x.row.cells[cid].status;
          counts[cid][st] = (counts[cid][st] || 0) + 1;
        }
      }
    });
    for (var cid in counts) {
      var parts = [];
      STATUSES.forEach(function (s) { if (counts[cid][s] && s !== "n/a") parts.push(counts[cid][s] + GLYPH[s]); });
      b.sums[cid].textContent = parts.join(" ") || " ";
      b.sums[cid].title = STATUSES.map(function (s) { return s + ": " + (counts[cid][s] || 0); }).join(", ");
    }
    els.count.textContent = shown + " of " + b.rows.length + " rows";
  }

  function sortBy(c, th) {
    var b = built[state.tab];
    if (state.sortCol === c.id) state.sortDir = -state.sortDir;
    else { state.sortCol = c.id; state.sortDir = 1; }
    b.columns.forEach(function (o) { o.th.removeAttribute("aria-sort"); });
    th.setAttribute("aria-sort", state.sortDir > 0 ? "ascending" : "descending");
    var key;
    if (c.kind === "name") key = function (x) { return x.row.name.toLowerCase(); };
    else if (c.kind === "info") key = function (x) { return (x.row.info[c.id] || "").toLowerCase(); };
    else key = function (x) { return ORDER[x.row.cells[c.id].status]; };
    var dir = state.sortDir;
    b.rows.slice().sort(function (p, q) {
      var a = key(p), z = key(q);
      if (a < z) return -dir;
      if (a > z) return dir;
      return p.index - q.index;
    }).forEach(function (x) { b.tbody.appendChild(x.tr); });
  }

  function openPanel() {
    els.panel.classList.add("open");
    els.panel.setAttribute("aria-hidden", "false");
    els.panel.querySelector(".close").focus();
  }
  function closePanel() {
    els.panel.classList.remove("open");
    els.panel.setAttribute("aria-hidden", "true");
  }

  function rowDetails(tab, r, into) {
    into.appendChild(el("h3", null, "Row"));
    var ul = el("ul");
    ul.appendChild(el("li", { "class": "mono" }, r.path));
    if (r.source && r.source.path) {
      var li = el("li", null, "defined at ");
      li.appendChild(link(r.source.path, r.source.line));
      ul.appendChild(li);
    }
    tab.info_columns.forEach(function (c) {
      if (r.info[c.id]) ul.appendChild(el("li", null, c.label + ": " + r.info[c.id]));
    });
    (r.reexports || []).forEach(function (p) { ul.appendChild(el("li", { "class": "mono" }, "also at " + p)); });
    into.appendChild(ul);
  }

  function showCell(tab, r, c) {
    var cellData = r.cells[c.id];
    var body = els.panelBody;
    body.textContent = "";
    body.appendChild(el("h2", { "class": "mono" }, r.name));
    var chip = el("span", { "class": "chip " + cls(cellData.status) });
    chip.appendChild(el("span", { "class": "g", "aria-hidden": "true" }, GLYPH[cellData.status]));
    chip.appendChild(el("span", null, c.label + ": " + cellData.status + (cellData.label && cellData.label !== cellData.status ? " (" + cellData.label + ")" : "")));
    body.appendChild(chip);
    body.appendChild(el("p", { "class": "rule" }, data.legend.columns[c.id] || ""));
    body.appendChild(el("h3", null, "Evidence"));
    if (!cellData.evidence.length) body.appendChild(el("p", { "class": "rule" }, "none"));
    else {
      var ul = el("ul");
      cellData.evidence.forEach(function (e) {
        var li = el("li");
        li.appendChild(el("span", { "class": "mono" }, e.text));
        if (e.path) { li.appendChild(document.createTextNode(" ")); li.appendChild(link(e.path, e.line)); }
        ul.appendChild(li);
      });
      if (cellData.more) ul.appendChild(el("li", null, "+ " + cellData.more + " more not listed"));
      body.appendChild(ul);
    }
    rowDetails(tab, r, body);
    openPanel();
  }

  function showRow(tab, r) {
    var body = els.panelBody;
    body.textContent = "";
    body.appendChild(el("h2", { "class": "mono" }, r.name));
    rowDetails(tab, r, body);
    body.appendChild(el("h3", null, "Cells"));
    var ul = el("ul");
    data.cell_columns.forEach(function (c) {
      var cd = r.cells[c.id];
      var li = el("li");
      var b = el("button", { "class": "chip " + cls(cd.status), type: "button" });
      b.appendChild(el("span", { "class": "g", "aria-hidden": "true" }, GLYPH[cd.status]));
      b.appendChild(el("span", null, c.label + ": " + cd.label));
      b.addEventListener("click", function () { showCell(tab, r, c); });
      li.appendChild(b);
      ul.appendChild(li);
    });
    body.appendChild(ul);
    openPanel();
  }

  els.panel.querySelector(".close").addEventListener("click", closePanel);
  document.addEventListener("keydown", function (ev) { if (ev.key === "Escape") closePanel(); });
  els.search.addEventListener("input", function () { state.search = els.search.value; applyFilters(); });
  els.gaps.addEventListener("change", function () { state.gapsOnly = els.gaps.checked; applyFilters(); });

  buildTabs();
  selectTab(0);
})();
"""


def legend_html(data: dict) -> str:
    legend = data["legend"]
    glyphs = legend["glyphs"]
    statuses = []
    for status in ["yes", "partial", "gap", "no", "n/a"]:
        css = "s-na" if status == "n/a" else f"s-{status}"
        statuses.append(
            f'<span><span class="chip {css}"><span class="g" aria-hidden="true">{html.escape(glyphs[status])}</span>'
            f"<span>{html.escape(status)}</span></span> {html.escape(legend['statuses'][status])}</span>"
        )
    labels = {c["id"]: c["label"] for c in data["cell_columns"]}
    rules = "".join(
        f"<dt>{html.escape(labels[cid])}</dt><dd>{html.escape(text)}</dd>"
        for cid, text in legend["columns"].items()
    )
    rows = "".join(
        f"<dt>{html.escape(name)}</dt><dd>{html.escape(text)}</dd>" for name, text in legend["rows"].items()
    )
    return (
        '<details class="legend"><summary>Legend and rules</summary>'
        f'<div class="legend-statuses">{"".join(statuses)}</div>'
        f"<dl>{rows}{rules}</dl></details>"
    )


def render_html(data: dict) -> str:
    commit = data["commit"]
    short = commit[:10]
    commit_url = f"{data['repo_url']}/commit/{commit}"
    dirty = ' <span class="dirty">uncommitted changes</span>' if data.get("dirty") else ""
    payload = json.dumps(data, ensure_ascii=False, separators=(",", ":")).replace("</", "<\\/")
    return f"""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>miniextendr type matrix</title>
<style>{CSS}</style>
</head>
<body>
<main>
<header>
<h1>miniextendr type matrix</h1>
<span class="meta">commit <a href="{html.escape(commit_url)}">{html.escape(short)}</a>, {html.escape(data["generated_at"])}{dirty}</span>
</header>
{legend_html(data)}
<div class="controls">
<div class="tabs" id="tabs" role="tablist" aria-label="Tables"></div>
<span id="filters"></span>
<input type="search" id="search" placeholder="Search names, paths, modules" aria-label="Search">
<label><input type="checkbox" id="gaps"> rows with gaps only</label>
<span class="count" id="count"></span>
</div>
<div class="table-wrap" id="wrap" role="region" aria-label="Type matrix table" tabindex="0"></div>
</main>
<aside id="panel" aria-hidden="true" aria-label="Evidence">
<button class="close" type="button" aria-label="Close">&#x2715;</button>
<div id="panel-body"></div>
</aside>
<script type="application/json" id="matrix-data">{payload}</script>
<script>{JS}</script>
</body>
</html>
"""


def main(argv=None) -> int:
    argv = sys.argv[1:] if argv is None else argv
    if len(argv) != 2:
        print(__doc__.strip().splitlines()[-1], file=sys.stderr)
        return 2
    data = json.loads(Path(argv[0]).read_text(encoding="utf-8"))
    Path(argv[1]).write_text(render_html(data), encoding="utf-8")
    print(f"Wrote {argv[1]}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
