#!/usr/bin/env node
// Merges extra npm metadata into the wasm-pack-generated pkg/package.json.
// Run after every wasm-pack build since wasm-pack regenerates the file.
import { readFileSync, writeFileSync } from "fs";
import { resolve, dirname } from "path";
import { fileURLToPath } from "url";

const __dir = dirname(fileURLToPath(import.meta.url));
const pkgPath = resolve(__dir, "../pkg/package.json");

const pkg = JSON.parse(readFileSync(pkgPath, "utf8"));

Object.assign(pkg, {
  name: "libdof",
  description: ".dof keyboard layout format parser — Rust/WebAssembly",
  license: "Apache-2.0",
  repository: {
    type: "git",
    url: "https://github.com/O-X-E-Y/libdof",
  },
  keywords: ["keyboard", "layout", "dof", "wasm", "parser"],
  sideEffects: ["./libdof_wasm.js"],
  files: [
    "libdof_wasm_bg.wasm",
    "libdof_wasm.js",
    "libdof_wasm_bg.js",
    "libdof_wasm.d.ts",
    "libdof_wasm_bg.wasm.d.ts",
    "package.json",
  ],
});

writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + "\n");
console.log("pkg/package.json patched.");
