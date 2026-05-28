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
//   /opt/webr/dist/                 ← baseUrl for R.bin.wasm / worker / vfs
//
// webR's esbuild config (.webr/src/esbuild.ts) emits two distinct
// bundles: a *browser* one at /opt/webr/dist/webr.mjs (which stubs out
// `fs`/`worker_threads`/`url`/etc and can't run in Node), and a *Node*
// one at /opt/webr/src/dist/webr.{cjs,mjs} (the .mjs has a banner
// defining __dirname/__filename/createRequire so the ESM build works in
// Node). The image's Dockerfile runs `cd src && make clean` which
// removes /opt/webr/src/dist; the CI workflow rebuilds it via
// `npm run build` before this script runs.
//
// Hard miniextendr Imports (cli/lifecycle/R6/S7/vctrs) are fetched from
// repo.r-wasm.org via `webR.installPackages` before the library load.
// If the network fetch fails the smoke aborts loudly — we cannot prove
// the side-module is healthy without the deps that its NAMESPACE pulls in.

import { WebR } from "file:///opt/webr/src/dist/webr.mjs";

const WASM_LIB_MOUNT = "/wasm-rlib";
const HOST_WASM_LIB = "/tmp/wasm-lib";

// Imports listed in rpkg/DESCRIPTION. `methods` is part of base R, so we
// don't try to install it here. Kept in sync manually — if DESCRIPTION's
// Imports change, update this list and the matching CI permissions on
// repo.r-wasm.org availability.
const HARD_IMPORTS = ["cli", "lifecycle", "R6", "S7", "vctrs"];

function unwrapScalar(rJsResult) {
  if (rJsResult && Array.isArray(rJsResult.values) && rJsResult.values.length > 0) {
    return rJsResult.values[0];
  }
  return String(rJsResult);
}

async function main() {
  console.log("[tier3] Initialising webR...");
  const webR = new WebR({
    baseUrl: "file:///opt/webr/dist/",
    interactive: false,
  });
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

  console.log("[tier3] Tier-3 PASSED.");
}

main().catch((e) => {
  console.error("[tier3] Uncaught runner error:", e && e.stack ? e.stack : e);
  process.exitCode = 1;
});
