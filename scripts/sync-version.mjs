#!/usr/bin/env node
/**
 * Single source of truth: root Cargo.toml [workspace.package] version (via cargo metadata).
 * Aligns phpvm-gui/package.json, package-lock.json, and src-tauri/tauri.conf.json before Vite/Tauri.
 */
import { execSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");

function readWorkspaceVersion() {
  const out = execSync("cargo metadata --format-version 1 --no-deps", {
    cwd: ROOT,
    encoding: "utf8",
    stdio: ["pipe", "pipe", "pipe"],
  });
  const { packages } = JSON.parse(out);
  const core = packages.find((p) => p.name === "phpvm-core");
  if (!core?.version) {
    throw new Error(
      "sync-version: phpvm-core version missing from cargo metadata (is the workspace root correct?)"
    );
  }
  return core.version;
}

function writeJson(file, obj) {
  fs.writeFileSync(file, JSON.stringify(obj, null, 2) + "\n", "utf8");
}

function main() {
  const version = readWorkspaceVersion();
  console.log(`sync-version: ${version} (from workspace / phpvm-core)`);

  const pkgPath = path.join(ROOT, "phpvm-gui", "package.json");
  const pkg = JSON.parse(fs.readFileSync(pkgPath, "utf8"));
  if (pkg.version !== version) {
    pkg.version = version;
    writeJson(pkgPath, pkg);
    console.log(`  write ${path.relative(ROOT, pkgPath)}`);
  }

  const tauriConfPath = path.join(ROOT, "phpvm-gui", "src-tauri", "tauri.conf.json");
  const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, "utf8"));
  if (tauriConf.version !== version) {
    tauriConf.version = version;
    writeJson(tauriConfPath, tauriConf);
    console.log(`  write ${path.relative(ROOT, tauriConfPath)}`);
  }

  const lockPath = path.join(ROOT, "phpvm-gui", "package-lock.json");
  const lock = JSON.parse(fs.readFileSync(lockPath, "utf8"));
  let lockChanged = false;
  if (lock.version !== version) {
    lock.version = version;
    lockChanged = true;
  }
  if (lock.packages?.[""]?.version !== version) {
    lock.packages[""].version = version;
    lockChanged = true;
  }
  if (lockChanged) {
    writeJson(lockPath, lock);
    console.log(`  write ${path.relative(ROOT, lockPath)}`);
  }
}

main();
