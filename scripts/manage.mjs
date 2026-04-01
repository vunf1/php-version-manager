#!/usr/bin/env node
/**
 * PHP Version Manager — manage console (Ink TUI + CLI).
 * Invoked by manage.ps1 at repo root.
 */
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import React from "react";
import { Box, Text, render, useInput } from "ink";
import Gradient from "ink-gradient";
import SelectInput from "ink-select-input";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");

const HELP = `
PHP Version Manager — manage

  dev       Tauri dev (hot reload)     -> build-dev.ps1 (Windows) / build-dev.sh (Unix)
  build     Release GUI + core         -> build.ps1 (Windows) / build.sh (Unix)
  test      cargo test --workspace
  check     cargo check --workspace
  status    Tooling / PATH check
  deps      npm install in phpvm-gui
  clean     Fresh build prep: remove phpvm-gui/dist, Vite cache, cargo clean --workspace
  help      This message

Windows: if Task Manager still shows an old app icon after rebuild, run:
  npm run refresh-icon-cache

Examples:
  .\\manage.ps1
  .\\manage.ps1 dev
  .\\manage.ps1 build
`;

function runCmdVersion(cmd) {
  if (process.platform === "win32") {
    const r = spawnSync("cmd", ["/c", `${cmd} --version 2>nul`], {
      encoding: "utf8",
      cwd: ROOT,
      windowsHide: true,
    });
    const code = r.status ?? 1;
    const out = String(r.stdout ?? "").trim();
    return { code, out };
  }
  const r = spawnSync(cmd, ["--version"], {
    encoding: "utf8",
    cwd: ROOT,
    windowsHide: true,
  });
  const code = r.status ?? 1;
  const out =
    String(r.stdout ?? "").trim() || String(r.stderr ?? "").trim();
  return { code, out };
}

function hasOnPath(name) {
  if (process.platform === "win32") {
    const r = spawnSync("where.exe", [name], {
      encoding: "utf8",
      windowsHide: true,
    });
    return (r.status ?? 1) === 0;
  }
  const r = spawnSync("command", ["-v", name], {
    encoding: "utf8",
    shell: true,
    windowsHide: true,
  });
  return (r.status ?? 1) === 0;
}

function printStatus() {
  console.log("\nStatus");

  const hasCargo = hasOnPath("cargo");
  const cargo = runCmdVersion("cargo");
  if (cargo.code === 0 && cargo.out) {
    console.log(`  [OK] cargo: ${cargo.out}`);
  } else if (hasCargo) {
    console.log("  [!!] cargo on PATH but failed (e.g. run rustup default stable)");
  } else {
    console.log("  [!!] cargo not on PATH");
  }

  const hasNode = hasOnPath("node");
  const node = runCmdVersion("node");
  if (node.code === 0 && node.out) {
    console.log(`  [OK] node:  ${node.out}`);
  } else if (hasNode) {
    console.log("  [!!] node on PATH but failed");
  } else {
    console.log("  [!!] node not on PATH");
  }

  const hasNpm = hasOnPath("npm");
  const npm = runCmdVersion("npm");
  if (npm.code === 0 && npm.out) {
    console.log(`  [OK] npm:   ${npm.out}`);
  } else if (hasNpm) {
    console.log("  [!!] npm on PATH but failed");
  } else {
    console.log("  [!!] npm not on PATH");
  }

  console.log(`\nRepo root: ${ROOT}`);
  console.log("Workspace: phpvm-core, phpvm-gui/src-tauri (see Cargo.toml)\n");
}

function runPs1(file) {
  const full = path.join(ROOT, file);
  if (!fs.existsSync(full)) {
    console.error(`Missing ${file} at repo root.`);
    return 1;
  }
  const r = spawnSync(
    "powershell.exe",
    ["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", full],
    { cwd: ROOT, stdio: "inherit", env: process.env, windowsHide: false },
  );
  return r.status ?? 1;
}

function runShellScript(file) {
  const full = path.join(ROOT, file);
  if (!fs.existsSync(full)) {
    console.error(`Missing ${file} at repo root.`);
    return 1;
  }
  let r = spawnSync("bash", [full], {
    cwd: ROOT,
    stdio: "inherit",
    env: process.env,
    windowsHide: false,
  });
  if (r.error?.code === "ENOENT") {
    r = spawnSync("sh", [full], {
      cwd: ROOT,
      stdio: "inherit",
      env: process.env,
      windowsHide: false,
    });
  }
  return r.status ?? 1;
}

function runDevStack() {
  if (process.platform === "win32") return runPs1("build-dev.ps1");
  return runShellScript("build-dev.sh");
}

function runReleaseStack() {
  if (process.platform === "win32") return runPs1("build.ps1");
  return runShellScript("build.sh");
}

function runCargo(args) {
  const r = spawnSync("cargo", args, {
    cwd: ROOT,
    stdio: "inherit",
    env: process.env,
    windowsHide: false,
  });
  return r.status ?? 1;
}

function rmDirIfExists(absPath) {
  if (!fs.existsSync(absPath)) return false;
  fs.rmSync(absPath, { recursive: true, force: true });
  return true;
}

/** Rust targets + frontend artifacts cargo does not touch (needed for a truly fresh Tauri release build). */
function runClean() {
  if (!hasOnPath("cargo")) {
    console.error("cargo not found.");
    return 1;
  }

  const gui = path.join(ROOT, "phpvm-gui");
  const dist = path.join(gui, "dist");
  if (rmDirIfExists(dist)) {
    console.log("Removed phpvm-gui/dist (Vite build; Tauri bundles this in release).");
  }

  const viteCache = path.join(gui, "node_modules", ".vite");
  if (rmDirIfExists(viteCache)) {
    console.log("Removed phpvm-gui/node_modules/.vite (Vite transform cache).");
  }

  const code = runCargo(["clean", "--workspace"]);
  if (code !== 0) return code;

  console.log(
    "cargo clean --workspace: cleared Rust target/ for phpvm-core and phpvm-gui/src-tauri.",
  );
  console.log(
    "Not removed: node_modules (run deps if needed). Windows icon cache: npm run refresh-icon-cache",
  );
  return 0;
}

function runNpmInstallGui() {
  const gui = path.join(ROOT, "phpvm-gui");
  if (!fs.existsSync(gui)) {
    console.error("phpvm-gui folder not found.");
    return 1;
  }
  const r = spawnSync("npm", ["install"], {
    cwd: gui,
    stdio: "inherit",
    env: process.env,
    shell: true,
    windowsHide: false,
  });
  return r.status ?? 1;
}

function resolveAction(raw) {
  const a = String(raw ?? "")
    .trim()
    .toLowerCase();
  const map = {
    "": "",
    dev: "dev",
    tauri: "dev",
    "tauri:dev": "dev",
    build: "build",
    release: "build",
    prod: "build",
    test: "test",
    check: "check",
    status: "status",
    deps: "deps",
    install: "deps",
    "npm-install": "deps",
    clean: "clean",
    help: "help",
    "-h": "help",
    "--help": "help",
    "/?": "help",
  };
  if (a in map) return map[a];
  return null;
}

function runAction(name) {
  switch (name) {
    case "quit":
      return 0;
    case "help":
      console.log(HELP);
      return 0;
    case "status":
      printStatus();
      return 0;
    case "deps":
      return runNpmInstallGui();
    case "clean":
      return runClean();
    case "check":
      if (!hasOnPath("cargo")) {
        console.error("cargo not found.");
        return 1;
      }
      return runCargo(["check", "--workspace"]);
    case "test":
      if (!hasOnPath("cargo")) {
        console.error("cargo not found.");
        return 1;
      }
      return runCargo(["test", "--workspace"]);
    case "dev":
      return runDevStack();
    case "build":
      return runReleaseStack();
    default:
      console.error(`Unknown action: ${name}`);
      console.error("Run manage.ps1 help for a list of actions.");
      return 1;
  }
}

function PremiumIndicator({ isSelected }) {
  return React.createElement(
    Box,
    { marginRight: 1, minWidth: 2 },
    React.createElement(
      Text,
      { color: isSelected ? "cyan" : "gray" },
      isSelected ? "\u276F" : " ",
    ),
  );
}

function PremiumItem({ isSelected, label }) {
  return React.createElement(
    Text,
    {
      bold: isSelected,
      color: isSelected ? "white" : "gray",
      dimColor: !isSelected,
    },
    label,
  );
}

function Menu({ onSelect }) {
  useInput((input, key) => {
    if (input === "q" || key.escape) {
      onSelect("quit");
    }
  });

  const items = [
    {
      label: "Develop   Tauri dev + hot reload",
      value: "dev",
    },
    {
      label: "Build     Release installers + core",
      value: "build",
    },
    {
      label: "Test      cargo test --workspace",
      value: "test",
    },
    {
      label: "Check     cargo check --workspace",
      value: "check",
    },
    {
      label: "Status    cargo / node / npm",
      value: "status",
    },
    {
      label: "Deps      npm install (phpvm-gui)",
      value: "deps",
    },
    {
      label: "Clean     dist + Rust targets (fresh build)",
      value: "clean",
    },
    {
      label: "Help      print actions",
      value: "help",
    },
    {
      label: "Quit",
      value: "quit",
    },
  ];

  return React.createElement(
    Box,
    { flexDirection: "column", paddingX: 1, paddingY: 0 },
    React.createElement(
      Box,
      { flexDirection: "column", marginBottom: 1 },
      React.createElement(
        Gradient,
        { colors: ["#22d3ee", "#6366f1", "#e879f9"] },
        React.createElement(Text, { bold: true }, "  PHP Version Manager  "),
      ),
      React.createElement(
        Text,
        { dimColor: true },
        "  Control center · pick an action",
      ),
    ),
    React.createElement(
      Box,
      {
        flexDirection: "column",
        borderStyle: "double",
        borderColor: "#67e8f9",
        paddingX: 2,
        paddingY: 1,
        marginTop: 1,
      },
      React.createElement(
        Text,
        { dimColor: true },
        `  ${ROOT}`,
      ),
      React.createElement(
        Box,
        { marginTop: 1, flexDirection: "column" },
        React.createElement(SelectInput, {
          items,
          indicatorComponent: PremiumIndicator,
          itemComponent: PremiumItem,
          onSelect(item) {
            onSelect(item.value);
          },
        }),
      ),
    ),
    React.createElement(
      Box,
      { marginTop: 1 },
      React.createElement(
        Text,
        { dimColor: true },
        "  \u2191\u2193 move   Enter run   q / Esc quit",
      ),
    ),
  );
}

async function runTui() {
  const selected = { value: "quit" };
  let inst;
  inst = render(
    React.createElement(Menu, {
      onSelect(v) {
        selected.value = v;
        inst.unmount();
      },
    }),
  );
  await inst.waitUntilExit();
  return selected.value;
}

const arg = String(process.argv[2] ?? "").trim();
const resolved = resolveAction(arg);

if (arg !== "" && resolved === null) {
  console.error(`Unknown action: ${arg}`);
  console.log(HELP);
  process.exitCode = 1;
} else if (arg !== "") {
  process.exitCode = runAction(resolved);
} else {
  const choice = await runTui();
  process.exitCode = runAction(choice);
}
