// Tier-3 webR Node smoke runner (closes #492).
//
// Boots a webR Node session inside the upstream/mirrored webR base image
// (see Dockerfile.webr's WEBR_BASE), NODEFS-mounts the wasm install
// produced by tier-2 (Phase 2 of `tests/webr-smoke.sh` / the
// `webr-install` GHA job), then drives `library(miniextendr)` to prove
// the side-module actually loads in a real webR runtime, not just at the
// emcc link step.
//
// Expected layout when this script runs:
//   /tmp/wasm-lib/miniextendr/      ← tier-2 R CMD INSTALL output
//   /opt/webr/src/dist/webr.mjs     ← Node-targeted webR bundle (loaded here)
//   /opt/webr/src/dist/R.wasm       ← R runtime + workers + vfs
//   /opt/webr/src/dist/webr-worker.js
//   /opt/webr/src/dist/vfs/
//
// webR's esbuild config (.webr/src/esbuild.ts) emits two distinct
// bundles: a *browser* one at /opt/webr/dist/webr.mjs (which stubs out
// `fs`/`worker_threads`/`url`/etc and can't run in Node), and a *Node*
// one at /opt/webr/src/dist/webr.{cjs,mjs} (the .mjs has a banner
// defining __dirname/__filename/createRequire so the ESM build works in
// Node). The image's Dockerfile wipes both `src/node_modules` and
// `src/dist`; the CI workflow rebuilds them via the upstream Makefile
// `$(PKG_DIST)/webr.mjs` target, which also copies R.wasm/R.js/vfs/
// webr-worker.js out of /opt/webr/dist into /opt/webr/src/dist so the
// bundle resolves all runtime assets via its own __dirname — no
// `baseUrl` override needed. (And we MUST NOT set a `file://` baseUrl:
// Node 18+'s `new Worker(string)` rejects file:// URL strings with
// ERR_WORKER_PATH, so the bundle would crash at init building the
// webr-worker.js worker path off such a baseUrl.)
//
// Hard miniextendr Imports are derived from rpkg/DESCRIPTION (minus base-R
// packages) and fetched from repo.r-wasm.org via `webR.installPackages`
// before the library load. If the network fetch fails the smoke aborts
// loudly — we cannot prove the side-module is healthy without the deps that
// its NAMESPACE pulls in. (A newly added Import must exist as a wasm binary
// on repo.r-wasm.org; this runner surfaces that requirement automatically.)
//
// Opt-in testthat pass (#1255): when SMOKE_TESTTHAT=1, also NODEFS-mounts
// rpkg/tests and runs the testthat suite against the already-loaded wasm
// install (`load_package = "installed"` — no recompile, no pkgload::load_all,
// just `library()` against what's already attached). This is *informational
// only* — many rpkg tests assume fork/thread/worker semantics that don't hold
// under webR's single-threaded, no-fork interpreter, so red counts here are
// expected and never fail the gate. Only a harness-level error (testthat
// itself won't install, the mount fails, R errors before producing counts)
// sets a non-zero exit code. See docs/WEBR.md.

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { WebR } from "file:///opt/webr/src/dist/webr.mjs";

const WASM_LIB_MOUNT = "/wasm-rlib";
const HOST_WASM_LIB = "/tmp/wasm-lib";

// Scaffold leg (#1259, #1271): when set, end-user package(s) freshly
// scaffolded from minirextendr's templates have been wasm-installed into
// HOST_WASM_LIB alongside miniextendr (see the "Scaffold leg" /
// "Monorepo scaffold leg" steps in .github/workflows/webr.yml). Load each and
// drive the template's stock #[miniextendr] functions (add/hello — identical
// in the standalone rpkg and monorepo rpkg templates, renamed
// <pkg>_add/<pkg>_hello at scaffold time, see the #1273 note at the loop
// below). Comma-separated list (#1271) so the monorepo-template package rides
// alongside the standalone one, e.g. SMOKE_SCAFFOLD_PKG=mxsmoke,mxmono.
// Unset/empty → behave exactly as before (e.g. tests/webr-smoke.sh without
// --scaffold).
const SCAFFOLD_PKGS = (process.env.SMOKE_SCAFFOLD_PKG ?? "")
  .split(",")
  .map((s) => s.trim())
  .filter((s) => s.length > 0);

// Informational testthat pass (#1255): opt-in via SMOKE_TESTTHAT=1. The local
// smoke (tests/webr-smoke.sh) defaults it ON; CI keeps per-PR tier-3
// load-only and enables it on main-push / workflow_dispatch. See the header
// comment and informationalTestthat() below for the tolerate-and-report
// semantics.
const SMOKE_TESTTHAT = process.env.SMOKE_TESTTHAT === "1";
const TESTS_MOUNT = "/rpkg-tests";

// Base-R packages ship with webR itself — never install them from the repo.
const R_BASE_PKGS = new Set([
  "base", "compiler", "datasets", "graphics", "grDevices", "grid", "methods",
  "parallel", "splines", "stats", "stats4", "tcltk", "tools", "utils",
]);

// Parse the Imports field out of rpkg/DESCRIPTION so there is no hand-synced
// copy of the list here; the tier-2 workflow step derives its native-install
// list from the same file. DCF format: the field's value runs until the next
// line that starts at column 0.
function hardImportsFromDescription() {
  const descPath = join(
    dirname(fileURLToPath(import.meta.url)), "..", "..", "rpkg", "DESCRIPTION",
  );
  const lines = readFileSync(descPath, "utf8").split("\n");
  const start = lines.findIndex((l) => l.startsWith("Imports:"));
  if (start === -1) throw new Error(`No Imports field found in ${descPath}`);
  let field = lines[start].slice("Imports:".length);
  for (let i = start + 1; i < lines.length && /^[ \t]/.test(lines[i]); i++) {
    field += ` ${lines[i]}`;
  }
  const imports = field
    .split(",")
    .map((s) => s.trim().replace(/\s*\(.*\)$/, ""))
    .filter((s) => s.length > 0 && !R_BASE_PKGS.has(s));
  if (imports.length === 0) {
    throw new Error(`Parsed zero non-base Imports from ${descPath}`);
  }
  return imports;
}

const HARD_IMPORTS = hardImportsFromDescription();

function unwrapScalar(rJsResult) {
  if (rJsResult && Array.isArray(rJsResult.values) && rJsResult.values.length > 0) {
    return rJsResult.values[0];
  }
  return String(rJsResult);
}

// Module-scoped so the top-level teardown can terminate webR's worker even
// when main() bails on an error path.
let webR;

async function main() {
  console.log("[tier3] Initialising webR...");
  webR = new WebR({ interactive: false });
  await webR.init();
  console.log("[tier3] webR initialised.");

  // Mount the tier-2 install output into the webR virtual filesystem.
  // We mount /tmp/wasm-lib directly rather than copying into webR's
  // own wasm library — webR's site lib supplies base packages, and the
  // mounted .libPaths entry only needs to expose `miniextendr` itself.
  await webR.FS.mkdir(WASM_LIB_MOUNT);
  await webR.FS.mount("NODEFS", { root: HOST_WASM_LIB }, WASM_LIB_MOUNT);
  await webR.evalR(`.libPaths(c('${WASM_LIB_MOUNT}', .libPaths()))`);
  console.log(`[tier3] NODEFS-mounted ${HOST_WASM_LIB} -> ${WASM_LIB_MOUNT}.`);

  // Pre-install miniextendr's Imports from the webR CRAN mirror.
  console.log(`[tier3] Installing Imports from repo.r-wasm.org: ${HARD_IMPORTS.join(", ")}`);
  try {
    await webR.installPackages(HARD_IMPORTS, { mount: false, quiet: false });
  } catch (e) {
    console.error("[tier3] FAIL: installPackages threw:", e && e.message ? e.message : e);
    process.exitCode = 1;
    return;
  }

  // Sanity-verify each Import resolved (installPackages is best-effort —
  // a 404 on one package wouldn't necessarily throw).
  const missingResult = await webR.evalR(`
    pkgs <- c(${HARD_IMPORTS.map((p) => JSON.stringify(p)).join(", ")})
    paste(setdiff(pkgs, rownames(installed.packages())), collapse=",")
  `);
  const missing = unwrapScalar(await missingResult.toJs());
  if (typeof missing === "string" && missing.length > 0) {
    console.error("[tier3] FAIL: Imports missing after installPackages:", missing);
    process.exitCode = 1;
    return;
  }

  // The gating check: load the wasm-installed miniextendr.
  console.log("[tier3] library(miniextendr) ...");
  const loadResult = await webR.evalR(`
    tryCatch({
      suppressPackageStartupMessages(library(miniextendr))
      "OK"
    }, error = function(e) paste0("ERROR: ", conditionMessage(e)))
  `);
  const loadMsg = unwrapScalar(await loadResult.toJs());
  if (typeof loadMsg !== "string" || !loadMsg.startsWith("OK")) {
    console.error("[tier3] FAIL: library(miniextendr) failed:", loadMsg);
    process.exitCode = 1;
    return;
  }
  console.log("[tier3] OK: library(miniextendr) loaded.");

  // Minimal post-load sanity: ask for the installed version. Cheap proof
  // that the package metadata is intact and that R can introspect the
  // wasm-installed namespace.
  const versionResult = await webR.evalR(
    "as.character(packageVersion('miniextendr'))",
  );
  const version = unwrapScalar(await versionResult.toJs());
  console.log(`[tier3] miniextendr version: ${version}`);

  // Scaffold leg (#1259, #1271): load each scaffolded end-user package and
  // call the template's stock functions — proof the templates' wasm branches
  // produce a side-module that not only links but dispatches into Rust in a
  // real webR runtime. The scaffolds have no R-level Imports, so no extra
  // installPackages round-trip is needed. The functions are the template's
  // add()/hello() (byte-identical in the standalone and monorepo rpkg
  // templates), renamed <pkg>_add()/<pkg>_hello() at scaffold time (webr.yml
  // create steps) because miniextendr (loaded above) also exports an `add`
  // and the C wrapper symbols are package-agnostic — under Emscripten's
  // shared-GOT side-module linking the first-loaded package's symbol wins
  // (#1273). Loading the packages sequentially in ONE session is deliberate:
  // it is exactly the multi-package shared-GOT scenario #1273 describes, so
  // per-package unique symbols are load-bearing here.
  for (const pkg of SCAFFOLD_PKGS) {
    console.log(`[tier3] scaffold leg: library(${pkg}) ...`);
    const scaffoldResult = await webR.evalR(`
      tryCatch({
        suppressPackageStartupMessages(library(${pkg}))
        paste(as.character(${pkg}::${pkg}_add(2, 3)),
              ${pkg}::${pkg}_hello("webR"), sep = " | ")
      }, error = function(e) paste0("ERROR: ", conditionMessage(e)))
    `);
    const scaffoldMsg = unwrapScalar(await scaffoldResult.toJs());
    if (scaffoldMsg !== "5 | Hello, webR!") {
      console.error(
        `[tier3] FAIL: scaffold package ${pkg} smoke returned:`,
        scaffoldMsg,
      );
      process.exitCode = 1;
      return;
    }
    console.log(
      `[tier3] OK: ${pkg} loaded; ${pkg}_add(2, 3) == 5 and ${pkg}_hello("webR") returned the template greeting.`,
    );
  }

  // Informational testthat pass (#1255). Runs LAST among the R-driving
  // steps: if the budget below abandons a wedged suite, the R worker is
  // left mid-eval and unusable for further evalR calls — nothing after this
  // point needs it (the top-level teardown's webR.close() terminates it).
  if (SMOKE_TESTTHAT) {
    let budgetTimer;
    const budget = new Promise((resolve) => {
      budgetTimer = setTimeout(() => resolve("budget"), TESTTHAT_BUDGET_MS);
    });
    const outcome = await Promise.race([
      informationalTestthat().then(
        () => "done",
        (e) => {
          console.error(
            "[tier3][testthat] FAIL: harness error before counts:",
            e && e.message ? e.message : e,
          );
          return "harness-error";
        },
      ),
      budget,
    ]);
    clearTimeout(budgetTimer);
    if (outcome === "harness-error") {
      // Per the #1255 semantics: test FAILURES never gate, but the harness
      // failing to produce a counts line at all is a smoke-infrastructure
      // regression and must be visible as a red gate.
      process.exitCode = 1;
      return;
    }
    if (outcome === "budget") {
      console.log(
        `[tier3][testthat] ${TESTTHAT_BUDGET_MS / 60000}-min budget exceeded — abandoning the informational pass (not a gate: a suite wedged on a fork/subprocess attempt is a wasm incompatibility, not a harness error; the stuck R worker is torn down by webR.close()).`,
      );
    }
  }

  console.log("[tier3] Tier-3 PASSED.");
}

// ── Informational testthat pass (#1255, opt-in via SMOKE_TESTTHAT=1) ────────
// Restores the coverage dropped when the local and CI runners were unified
// onto this script: run rpkg's testthat suite inside the webR session and
// report counts. Tolerate-and-report by design — many tests legitimately
// fail or skip under wasm (worker-thread / fork / subprocess assumptions),
// so red counts NEVER touch the exit code. Only a harness error before the
// counts line (testthat won't install, the mount fails, test_dir itself
// errors) rejects, which main() converts into a red gate. The Promise.race
// budget in main() keeps a wedged suite from eating the outer step timeout.
const TESTTHAT_BUDGET_MS = 20 * 60 * 1000;

async function informationalTestthat() {
  // testthat is in rpkg's Suggests, not Imports, so it is not part of the
  // HARD_IMPORTS install above. webr::install resolves Depends/Imports
  // recursively, so testthat's own dependency tree comes along for free.
  // Other Suggests (dplyr, ggplot2, ...) are deliberately NOT installed:
  // tests guarded by skip_if_not_installed() skip, and anything else that
  // fails on a missing Suggest is tolerated noise in an informational pass.
  console.log("[tier3][testthat] Installing testthat from repo.r-wasm.org...");
  await webR.installPackages(["testthat"], { mount: false, quiet: false });

  // Mount rpkg/tests (not the whole source tree — the suite only needs the
  // tests dir, and the package code itself comes from the wasm install
  // already loaded above). NODEFS is read-write: failing snapshot tests may
  // write _snaps/*.new files back into the checkout — untracked,
  // informational diagnostics, safe to delete.
  const testsDir = join(
    dirname(fileURLToPath(import.meta.url)), "..", "..", "rpkg", "tests",
  );
  await webR.FS.mkdir(TESTS_MOUNT);
  await webR.FS.mount("NODEFS", { root: testsDir }, TESTS_MOUNT);
  console.log(
    `[tier3][testthat] NODEFS-mounted ${testsDir} -> ${TESTS_MOUNT}; running test_dir (silent reporter)...`,
  );

  // - MINIEXTENDR_SKIP_STRESS: the gctorture files are ~94% of the suite's
  //   native runtime and have their own dedicated CI job; under the (much
  //   slower) wasm interpreter they are prohibitive. Same convention as
  //   every non-stress CI job (see rpkg/tests/testthat/helper-gc-stress.R).
  // - NOT_CRAN=true: mirrors test_local()'s local_assume_not_on_cran();
  //   without it every skip_on_cran() guard (including the gc-stress
  //   helper's) skips for the wrong reason and deflates the counts.
  // - load_package = "installed": run against the wasm-installed package
  //   already attached — never pkgload::load_all(), which would try to
  //   compile Rust inside the webR session.
  // - reporter = "silent" + stop_on_failure = FALSE: counts are the
  //   contract; per-expectation output would be enormous and failures are
  //   expected. The tryCatch turns a harness-level error into a sentinel
  //   that we re-throw JS-side.
  const res = await webR.evalR(`
    tryCatch({
      Sys.setenv(MINIEXTENDR_SKIP_STRESS = "1", NOT_CRAN = "true")
      res <- testthat::test_dir(
        "${TESTS_MOUNT}/testthat",
        package = "miniextendr",
        load_package = "installed",
        reporter = "silent",
        stop_on_failure = FALSE
      )
      df <- as.data.frame(res)
      sprintf(
        "COUNTS passed=%d failed=%d skipped=%d errors=%d warnings=%d",
        sum(df$passed), sum(df$failed), sum(df$skipped),
        sum(df$error), sum(df$warning)
      )
    }, error = function(e) paste0("HARNESS_ERROR: ", conditionMessage(e)))
  `);
  const line = unwrapScalar(await res.toJs());
  if (typeof line !== "string" || !line.startsWith("COUNTS ")) {
    throw new Error(String(line));
  }
  console.log(
    `[tier3][testthat] ${line.slice("COUNTS ".length)} (informational — test failures do not gate)`,
  );
}

// webR boots a dedicated worker (Node `worker_threads`) to host the R runtime.
// That worker — plus the channel listeners and the open NODEFS mount — keeps
// Node's event loop alive after main() resolves, so the process never exits on
// its own. Under CI that means a fully *successful* run still hangs until
// `timeout 900 node smoke.mjs` kills it with exit code 124 (the failure this
// runner was hitting). webR.close() terminates the worker; the explicit
// process.exit() then guarantees a prompt, deterministic exit regardless of any
// residual handles. This is a pass/fail CI gate, so an explicit exit code is
// exactly the contract we want. All the meaningful "[tier3] …" lines are
// emitted before we get here, so force-exiting can't truncate them.
main()
  .catch((e) => {
    console.error("[tier3] Uncaught runner error:", e && e.stack ? e.stack : e);
    process.exitCode = 1;
  })
  .finally(() => {
    if (webR) {
      try {
        webR.close();
      } catch {
        // Best-effort teardown — we're exiting anyway.
      }
    }
    process.exit(process.exitCode ?? 0);
  });
