# libdof

WebAssembly/npm build of [libdof](https://github.com/O-X-E-Y/libdof) — a parser and validator for the `.dof` keyboard layout format.

## Installation

```
npm install libdof
```

## Usage

```js
import init, { Dof, DofIntermediate } from "libdof";

await init();

const dof = new Dof(dofJsonString); // throws on invalid input

console.log(dof.name());        // string
console.log(dof.authors());     // string[]
console.log(dof.year());        // number | undefined
console.log(dof.description()); // string | undefined
console.log(dof.link());        // string | undefined
console.log(dof.languages());   // string[]
console.log(dof.form_factor()); // e.g. "Standard" | "Angle" | ...
```

The WASM binary must be initialized before use. With a bundler (Vite, webpack, rollup) the `await init()` call is all that's needed.

### Layers

```js
const main = dof.main_layer();   // Layer
const shift = dof.shift_layer(); // Layer
const sym = dof.layer("sym");    // Layer | undefined

// Layer methods
main.rows();        // Key[][] — all rows
main.keys();        // Key[] — all keys in row-major order
main.shape();       // number[] — keys per row
main.row_count();   // number
main.get_key(0, 4); // Key | undefined
main.set_key(0, 4, "a"); // mutates, throws on invalid pos

// dof.layers() returns a plain Record<string, Key[][]> for all layers
```

### Key types

```js
// Key has a `kind` field ("Char" | "Word" | "Special" | "Layer" | "Magic" | "Empty" | "Transparent")
// and convenience methods:
key.is_char()        // boolean
key.is_word()        // boolean
key.is_empty()       // boolean
key.is_transparent() // boolean
key.is_layer()       // boolean
key.is_magic()       // boolean

key.char_output()    // string | undefined
key.word_output()    // string | undefined
key.layer_label()    // string | undefined
key.magic_label()    // string | undefined
```

### Board and fingering

```js
// PhysicalKey has { x, y, width, height } (all numbers)
dof.board();     // PhysicalKey[][] — physical key positions

// Finger is an enum: LP | LR | LM | LI | LT | RT | RI | RM | RR | RP
dof.fingering(); // Finger[][]

dof.anchor();         // [number, number] — [x, y] anchor offset
dof.shape();          // number[] — keys per row
dof.fingering_name(); // string | undefined — named fingering e.g. "standard"
```

### Key lookups

```js
// KeyPos has { layer: string, pos: Pos } where Pos has { row, col }
dof.get("a");       // KeyPos[] — all positions of key "a" across all layers
dof.tower(0, 4);    // Key[] — all keys at (row=0, col=4) across all layers
dof.finger(0, 4);   // Finger | undefined
```

### DescriptiveKey

`dof.keys()` returns a flat list of `DescriptiveKey` instances, one per key per layer, with full context:

```js
for (const key of dof.keys()) {
  key.output()        // Key
  key.pos()           // Pos — { row, col }
  key.row()           // number
  key.col()           // number
  key.finger()        // Finger
  key.physical_pos()  // PhysicalKey
  key.layer_name()    // string
  key.keypos()        // KeyPos

  // Predicates
  key.is_char_key()
  key.is_on_left_hand()
  key.is_on_right_hand()
  key.is_on_finger(Finger.LI)
  key.is_on_fingers([Finger.LI, Finger.RI])
  key.is_on_layer("main")

  // Output accessors
  key.char_output()   // string | undefined
  key.word_output()   // string | undefined
  key.layer_output()  // string | undefined
  key.magic_label()   // string | undefined
}
```

### Magic keys

```js
const magic = dof.magic(); // Magic

magic.labels();          // string[]
magic.len();             // number
magic.is_empty();        // boolean
magic.keys();            // Record<string, { label: string, rules: Record<string, string> }>

const mgc = magic.key("mgc"); // MagicKey | undefined
mgc.label();             // string
mgc.rules();             // Record<string, string>
mgc.leading();           // string[] — input keys
mgc.outputs();           // string[] — output values
mgc.rule("a");           // string | undefined
mgc.add_rule("a", "b");  // void
mgc.remove_rule("a");    // string | undefined — removed value
mgc.len();               // number
```

### Combos

```js
// Record<string, Array<{ positions: Pos[], output: Key }>>
const combos = dof.combos();
for (const [layer, entries] of Object.entries(combos)) {
  for (const { positions, output } of entries) {
    console.log(layer, positions, output);
  }
}
```

### Swap and serialize

```js
// Swaps keys at two positions (can be different layers)
dof.swap("main", 0, 0, "main", 0, 1); // throws on invalid position

// Serialize back to .dof JSON
const json = dof.serialize();
```

### DofIntermediate

Accepts partially-specified layouts that would fail full validation (e.g. missing fingering). Useful for tooling that works with in-progress layouts.

```js
const intermediate = new DofIntermediate(dofJsonString); // throws on parse error
const json = intermediate.serialize();
```

### Error handling

All constructors and mutating methods throw a JavaScript `Error` on failure.

```js
try {
  const dof = new Dof(input);
} catch (e) {
  console.error("Invalid .dof:", e.message);
}
```

## `.dof` format

See the [libdof repository](https://github.com/O-X-E-Y/libdof) and the [example layouts](https://github.com/O-X-E-Y/libdof/tree/main/example_dofs) for format documentation.

## Building from source

Requires [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/).

```bash
# Bundler target (webpack, vite, rollup) — use this for npm publish
wasm-pack build --target bundler --out-name libdof_wasm --release
bun run scripts/patch-pkg.js

# Node.js target (no bundler)
wasm-pack build --target nodejs --out-dir pkg-node --out-name libdof_wasm --release

# Vanilla browser (no bundler)
wasm-pack build --target web --out-dir pkg-web --out-name libdof_wasm --release
```
