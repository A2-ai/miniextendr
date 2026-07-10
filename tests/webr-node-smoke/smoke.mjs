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

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { WebR } from "file:///opt/webr/src/dist/webr.mjs";

const WASM_LIB_MOUNT = "/wasm-rlib";
const HOST_WASM_LIB = "/tmp/wasm-lib";

// Scaffold leg (#1259): when set, an end-user package freshly scaffolded from
// minirextendr's templates has been wasm-installed into HOST_WASM_LIB
// alongside miniextendr (see the "Scaffold leg" steps in
// .github/workflows/webr.yml). Load it and drive the template's stock
// #[miniextendr] functions (add/hello from
// minirextendr/inst/templates/rpkg/lib.rs). Unset → behave exactly as before
// (tests/webr-smoke.sh has no scaffold leg).
const SCAFFOLD_PKG = process.env.SMOKE_SCAFFOLD_PKG ?? "";

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

  // Scaffold leg (#1259): load the scaffolded end-user package and call the
  // template's stock functions — proof the templates' wasm branches produce a
  // side-module that not only links but dispatches into Rust in a real webR
  // runtime. The scaffold has no R-level Imports, so no extra installPackages
  // round-trip is needed.
  if (SCAFFOLD_PKG) {
    console.log(`[tier3] scaffold leg: library(${SCAFFOLD_PKG}) ...`);
    const scaffoldResult = await webR.evalR(`
      tryCatch({
        suppressPackageStartupMessages(library(${SCAFFOLD_PKG}))
        paste(as.character(${SCAFFOLD_PKG}::add(2, 3)), ${SCAFFOLD_PKG}::hello("webR"), sep = " | ")
      }, error = function(e) paste0("ERROR: ", conditionMessage(e)))
    `);
    const scaffoldMsg = unwrapScalar(await scaffoldResult.toJs());
    if (scaffoldMsg !== "5 | Hello, webR!") {
      console.error(
        `[tier3] FAIL: scaffold package ${SCAFFOLD_PKG} smoke returned:`,
        scaffoldMsg,
      );
      process.exitCode = 1;
      return;
    }
    console.log(
      `[tier3] OK: ${SCAFFOLD_PKG} loaded; add(2, 3) == 5 and hello("webR") returned the template greeting.`,
    );
  }

  console.log("[tier3] Tier-3 PASSED.");
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
