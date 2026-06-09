# Skill freshness audit — first run (2026-06-09)

Closes the audit-tooling half of #626 (Flight 14 / #174 skill drift).

`scripts/skill-freshness-audit.sh` was added and run for the first time against
the 16 skills under `.claude/skills/<slug>/SKILL.md`. The script checks, per
skill: cited path existence (BLOCKING), symbol existence via repo-wide
`git grep` (WARN), and `file.rs:NNN` line cites (WARN). It exits non-zero on any
BLOCKING path miss so it can gate CI. See the script header for the full
false-positive-mode catalogue.

## First-run result (before fixes)

```
Audited 16 skill(s): 4 BLOCKING, 10 WARN
```

### BLOCKING (stale paths — all genuine drift)

| Skill | Stale cite | Reality |
|-------|-----------|---------|
| miniextendr-ffi | `miniextendr-api/src/ffi.rs` (×4 incl. Key files + nonapi) | renamed to `miniextendr-api/src/sys.rs` |
| miniextendr-ffi | `miniextendr-api/src/preserve.rs` | no `preserve` module; primitives live in `gc_protect.rs` (`Root`) |
| miniextendr-worker | `miniextendr-api/src/ffi.rs` | `miniextendr-api/src/sys.rs` |
| miniextendr-rv | `journal/2026-05-17-pr2-release-workflow-cran-pins.md` (×2) | file never existed; pin source of truth is `rproject.toml` + `rpkg/DESCRIPTION` `Config/roxygen2/version` |

### WARN (real symbol/line drift — also fixed)

| Skill | Stale cite | Reality |
|-------|-----------|---------|
| miniextendr-ffi | line block list `2092, 2634, …, 3790` for the `#[r_ffi_checked]` blocks | now at `248, 787, 956, 1126, 1509, 1572, 1681, 1955` in `sys.rs` |
| miniextendr-class-systems | `build_r6_return` (×2) | renamed to `build_r6_body` (`method_return_builder.rs:260`) |
| miniextendr-connections | `check_runtime_connections_support` (×2) | actual fn is `check_connections_runtime` (`connection.rs:131`) |
| miniextendr-architecture | prose abbrev `write_wrappers` | expanded to `miniextendr_write_wrappers` for precision |

### WARN (documented false positives — left as-is)

- `.cargo/config.toml`, `R/miniextendr-wrappers.R` etc. — generated / gitignored;
  absent in a clean tree by design.
- `R_ext/Connections.h` — external R header, lives in gitignored `background/`.
- `src/rust/src/lib.rs` — describes the *scaffolded end-user package* layout, not
  a path inside this repo (this repo's example is `rpkg/src/rust/lib.rs`).

## After fixes

```
Audited 16 skill(s): 0 BLOCKING, 7 WARN   (exit 0)
```

The 7 remaining WARNs are all the documented false-positive modes above — no real
drift remains.

## Cadence

Documented in `CLAUDE.md` and `AGENTS.md` under "Skill freshness audit
(quarterly)": run `bash scripts/skill-freshness-audit.sh` once a quarter and
repair drift in the same pass (source wins).
