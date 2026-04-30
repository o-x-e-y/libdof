# libdof

WebAssembly/npm build of [libdof](https://github.com/O-X-E-Y/libdof) — a parser and validator for the `.dof` keyboard layout format.

## Installation

```
npm install libdof
```

## Usage

The package exposes two functions. Both accept a `.dof` JSON string and return a JSON string (use `JSON.parse` to get an object). They throw a JavaScript `Error` on failure.

**Important:** The WASM binary must be initialized before calling any function. With a bundler (webpack, vite, rollup) this looks like:

```js
import init, { parse_dof, parse_dof_intermediate } from "libdof";

await init();

// Full parse + validation. Expands shift layers, resolves named fingerings,
// normalizes board to physical key coordinates.
const result = JSON.parse(parse_dof(dofJsonString));

// Parse only, no validation. Accepts partially-specified layouts.
const intermediate = JSON.parse(parse_dof_intermediate(dofJsonString));
```

### Error handling

```js
try {
  const layout = JSON.parse(parse_dof(input));
} catch (e) {
  console.error("Invalid .dof:", e.message);
}
```

### Node.js

Build with `--target nodejs` (see [Building from source](#building-from-source)) or use the bundler output with a tool like `@rollup/plugin-wasm`.

## `.dof` format

See the [libdof repository](https://github.com/O-X-E-Y/libdof) and the [example layouts](https://github.com/O-X-E-Y/libdof/tree/main/example_dofs) for format documentation.

## Building from source

Requires [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/).

```bash
# bundler target (webpack, vite, rollup)
wasm-pack build --target bundler --out-name libdof_wasm --release
node scripts/patch-pkg.js

# Node.js target
wasm-pack build --target nodejs --out-dir pkg-node --out-name libdof_wasm --release

# Vanilla browser (no bundler)
wasm-pack build --target web --out-dir pkg-web --out-name libdof_wasm --release
```
