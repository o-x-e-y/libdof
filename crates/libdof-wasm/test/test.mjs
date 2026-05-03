// Node.js integration test for the libdof-wasm npm package.
// Requires: wasm-pack build --target nodejs --out-dir pkg-node
import { Dof, DofIntermediate, Key, Pos } from "../pkg-node/libdof_wasm.js";
import { readFileSync } from "fs";
import { resolve, dirname } from "path";
import { fileURLToPath } from "url";

const __dir = dirname(fileURLToPath(import.meta.url));
const exampleDofs = resolve(__dir, "../../../example_dofs");

let passed = 0;
let failed = 0;

function test(name, fn) {
  try {
    fn();
    console.log(`  ✓ ${name}`);
    passed++;
  } catch (e) {
    console.error(`  ✗ ${name}`);
    console.error(`    ${e.message}`);
    failed++;
  }
}

function assert(cond, msg) {
  if (!cond) throw new Error(msg ?? "assertion failed");
}

const minimalValid = readFileSync(resolve(exampleDofs, "minimal_valid.dof"), "utf8");
const minimalParsable = readFileSync(resolve(exampleDofs, "minimal_parsable.dof"), "utf8");
const maximal = readFileSync(resolve(exampleDofs, "maximal.dof"), "utf8");
const aptmak = readFileSync(resolve(exampleDofs, "aptmak.dof"), "utf8");

// ---------------------------------------------------------------------------
console.log("Dof – construction & errors");

test("parses minimal_valid.dof", () => {
  const dof = new Dof(minimalValid);
  assert(dof instanceof Dof);
});

test("parses maximal.dof", () => {
  assert(new Dof(maximal) instanceof Dof);
});

test("parses aptmak.dof", () => {
  assert(new Dof(aptmak) instanceof Dof);
});

test("throws a real Error on invalid JSON", () => {
  let threw = false;
  try {
    new Dof("not json");
  } catch (e) {
    threw = true;
    assert(e instanceof Error, "should throw Error, not string");
    assert(e.message.length > 0, "error should have a message");
  }
  assert(threw, "should have thrown");
});

test("throws on incomplete dof", () => {
  let threw = false;
  try {
    new Dof('{"name":"test"}');
  } catch (e) {
    threw = true;
  }
  assert(threw, "should have thrown on incomplete dof");
});

// ---------------------------------------------------------------------------
console.log("\nDof – metadata (minimal_valid)");

test("name()", () => {
  const dof = new Dof(minimalValid);
  assert(dof.name() === "Qwerty", `expected "Qwerty", got ${dof.name()}`);
});

test("authors() is empty when absent", () => {
  const dof = new Dof(minimalValid);
  const authors = dof.authors();
  assert(Array.isArray(authors), "authors() should be an array");
  assert(authors.length === 0, "minimal_valid has no authors");
});

test("year() is undefined when absent", () => {
  const dof = new Dof(minimalValid);
  assert(dof.year() === undefined, "minimal_valid has no year");
});

test("description() is undefined when absent", () => {
  const dof = new Dof(minimalValid);
  assert(dof.description() === undefined, "minimal_valid has no description");
});

test("link() is undefined when absent", () => {
  const dof = new Dof(minimalValid);
  assert(dof.link() === undefined, "minimal_valid has no link");
});

test("form_factor() returns a string", () => {
  const dof = new Dof(minimalValid);
  const ff = dof.form_factor();
  assert(typeof ff === "string", `form_factor() should be a string, got ${typeof ff}`);
  assert(ff.length > 0, "form_factor() should not be empty");
});

test("fingering_name() returns a string for named fingering", () => {
  const dof = new Dof(minimalValid);
  const name = dof.fingering_name();
  assert(typeof name === "string", `fingering_name() should be a string, got ${typeof name}`);
});

test("anchor() returns a two-element array", () => {
  const dof = new Dof(minimalValid);
  const anchor = dof.anchor();
  assert(Array.isArray(anchor), "anchor() should be an array");
  assert(anchor.length === 2, "anchor() should have 2 elements");
  assert(typeof anchor[0] === "number", "anchor x should be a number");
  assert(typeof anchor[1] === "number", "anchor y should be a number");
});

test("shape() returns row lengths", () => {
  const dof = new Dof(minimalValid);
  const shape = dof.shape();
  assert(Array.isArray(shape), "shape() should be an array");
  assert(shape.length > 0, "shape() should not be empty");
  assert(shape.every((n) => typeof n === "number" && n > 0), "each entry should be a positive number");
});

test("languages() returns an array", () => {
  const dof = new Dof(minimalValid);
  const langs = dof.languages();
  // languages() returns Vec<Language> as [{language: string, weight: number}, ...]
  assert(Array.isArray(langs), "languages() should be an array");
});

// ---------------------------------------------------------------------------
console.log("\nDof – metadata (maximal)");

test("name()", () => {
  const dof = new Dof(maximal);
  assert(dof.name() === "Qwerty");
});

test("authors()", () => {
  const dof = new Dof(maximal);
  const authors = dof.authors();
  assert(Array.isArray(authors));
  assert(authors.length === 1);
  assert(authors[0] === "Christopher Latham Sholes");
});

test("year()", () => {
  const dof = new Dof(maximal);
  assert(dof.year() === 1878, `expected 1878, got ${dof.year()}`);
});

test("description()", () => {
  const dof = new Dof(maximal);
  const desc = dof.description();
  assert(typeof desc === "string");
  assert(desc.includes("Qwerty") || desc.length > 0, "description should be non-empty");
});

test("link()", () => {
  const dof = new Dof(maximal);
  const link = dof.link();
  assert(typeof link === "string");
  assert(link.includes("QWERTY"));
});

test("fingering_name() is undefined for custom fingering", () => {
  const dof = new Dof(maximal);
  assert(dof.fingering_name() === undefined, "maximal has custom fingering, not a named one");
});

test("languages() has english: 100", () => {
  const dof = new Dof(maximal);
  const langs = dof.languages();
  assert(Array.isArray(langs), "languages() should be an array");
  const english = langs.find((l) => l.language === "english");
  assert(english !== undefined, `expected an entry with language "english", got ${JSON.stringify(langs)}`);
  assert(english.weight === 100, `expected weight 100, got ${english.weight}`);
});

// ---------------------------------------------------------------------------
console.log("\nDof – layers");

test("main_layer() returns a Layer", () => {
  const dof = new Dof(minimalValid);
  const layer = dof.main_layer();
  assert(layer !== null && layer !== undefined);
  assert(typeof layer.rows === "function", "Layer should have rows()");
});

test("shift_layer() returns a Layer", () => {
  const dof = new Dof(minimalValid);
  const layer = dof.shift_layer();
  assert(layer !== null && layer !== undefined);
  assert(typeof layer.keys === "function");
});

test("layer(name) returns a Layer for existing layer", () => {
  const dof = new Dof(minimalValid);
  const layer = dof.layer("main");
  assert(layer !== undefined && layer !== null);
});

test("layer(name) returns undefined for nonexistent layer", () => {
  const dof = new Dof(minimalValid);
  assert(dof.layer("nonexistent") === undefined);
});

test("layers() returns a Map with layer name keys", () => {
  const dof = new Dof(minimalValid);
  const layers = dof.layers();
  assert(layers instanceof Map, "layers() should return a Map");
  assert(layers.has("main"), "should have a 'main' layer");
});

test("maximal layers() has main, shift, altgr", () => {
  const dof = new Dof(maximal);
  const layers = dof.layers();
  assert(layers.has("main"));
  assert(layers.has("shift"));
  assert(layers.has("altgr"));
});

// ---------------------------------------------------------------------------
console.log("\nLayer");

test("rows() returns Key[][]", () => {
  const dof = new Dof(minimalValid);
  const rows = dof.main_layer().rows();
  assert(Array.isArray(rows), "rows() should be an array");
  assert(rows.length > 0, "should have at least one row");
  assert(Array.isArray(rows[0]), "each row should be an array");
  const key = rows[0][0];
  assert(typeof key === "object" && key !== null);
});

test("keys() returns a flat Key[]", () => {
  const dof = new Dof(minimalValid);
  const keys = dof.main_layer().keys();
  assert(Array.isArray(keys));
  assert(keys.length > 0);
});

test("shape() returns row lengths", () => {
  const dof = new Dof(minimalValid);
  const shape = dof.main_layer().shape();
  // Vec<usize> comes back as a Uint32Array, not a plain Array
  assert(shape.length > 0, "shape() should not be empty");
  for (const n of shape) {
    assert(n > 0, `each row length should be positive, got ${n}`);
  }
});

test("shape() matches rows().map(r => r.length)", () => {
  const dof = new Dof(minimalValid);
  const layer = dof.main_layer();
  const shape = layer.shape();
  const rows = layer.rows();
  assert(shape.length === rows.length, "shape and rows should have same length");
  for (let i = 0; i < shape.length; i++) {
    assert(shape[i] === rows[i].length, `row ${i}: shape says ${shape[i]}, rows has ${rows[i].length}`);
  }
});

test("row_count() matches rows().length", () => {
  const dof = new Dof(minimalValid);
  const layer = dof.main_layer();
  assert(layer.row_count() === layer.rows().length);
});

test("get_key(row, col) returns a Key", () => {
  const dof = new Dof(minimalValid);
  const key = dof.main_layer().get_key(0, 0);
  assert(key !== undefined, "should return a Key for valid position");
  assert(typeof key === "object" && key !== null);
});

test("get_key() returns undefined for out-of-bounds position", () => {
  const dof = new Dof(minimalValid);
  const key = dof.main_layer().get_key(999, 999);
  assert(key === undefined, "should return undefined for out-of-bounds");
});

test("set_key() updates the key at a position", () => {
  const dof = new Dof(minimalValid);
  const layer = dof.main_layer();
  layer.set_key(0, 0, "z");
  // get_key() returns a serde-serialized plain object {kind, value}
  const updated = layer.get_key(0, 0);
  assert(updated !== undefined);
  assert(updated.value === "z", `expected value "z", got ${updated.value}`);
});

// ---------------------------------------------------------------------------
console.log("\nKey");

test("new Key('a') is_char()", () => {
  const key = new Key("a");
  assert(key.is_char(), "single char should be is_char");
});

test("new Key('a') char_output() returns 'a'", () => {
  const key = new Key("a");
  assert(key.char_output() === "a", `expected 'a', got ${key.char_output()}`);
});

test("is_char / is_word / is_empty / is_transparent are booleans", () => {
  const key = new Key("a");
  assert(typeof key.is_char() === "boolean");
  assert(typeof key.is_word() === "boolean");
  assert(typeof key.is_empty() === "boolean");
  assert(typeof key.is_transparent() === "boolean");
  assert(typeof key.is_layer() === "boolean");
  assert(typeof key.is_magic() === "boolean");
});

test("char key: char_output defined, word_output undefined", () => {
  const key = new Key("a");
  assert(key.char_output() !== undefined);
  assert(key.word_output() === undefined);
  assert(key.layer_label() === undefined);
  assert(key.magic_label() === undefined);
});

// ---------------------------------------------------------------------------
console.log("\nDof – board & fingering");

test("board() returns PhysicalKey[][]", () => {
  const dof = new Dof(minimalValid);
  const board = dof.board();
  assert(Array.isArray(board), "board() should be an array");
  assert(board.length > 0, "board should have rows");
  const key = board[0][0];
  assert(typeof key === "object" && key !== null);
  assert(typeof key.x === "number", "PhysicalKey should have x");
  assert(typeof key.y === "number", "PhysicalKey should have y");
  assert(typeof key.width === "number", "PhysicalKey should have width");
  assert(typeof key.height === "number", "PhysicalKey should have height");
});

test("board() dimensions match shape()", () => {
  const dof = new Dof(minimalValid);
  const board = dof.board();
  const shape = dof.shape();
  assert(board.length === shape.length, "board and shape should have same row count");
  for (let i = 0; i < shape.length; i++) {
    assert(board[i].length === shape[i], `row ${i}: board has ${board[i].length} keys, shape says ${shape[i]}`);
  }
});

test("fingering() returns Finger[][]", () => {
  const dof = new Dof(minimalValid);
  const fingering = dof.fingering();
  assert(Array.isArray(fingering), "fingering() should be an array");
  assert(fingering.length > 0);
  assert(Array.isArray(fingering[0]));
  fingering[0].forEach((f) => assert(f !== undefined, "each finger entry should be defined"));
});

test("fingering() dimensions match shape()", () => {
  const dof = new Dof(minimalValid);
  const fingering = dof.fingering();
  const shape = dof.shape();
  assert(fingering.length === shape.length);
  for (let i = 0; i < shape.length; i++) {
    assert(fingering[i].length === shape[i], `row ${i}: fingering has ${fingering[i].length}, shape says ${shape[i]}`);
  }
});

// ---------------------------------------------------------------------------
console.log("\nDof – magic (maximal)");

test("magic() is defined", () => {
  const dof = new Dof(maximal);
  const magic = dof.magic();
  assert(magic !== undefined && magic !== null);
});

test("magic().len() returns number of magic keys", () => {
  const dof = new Dof(maximal);
  assert(dof.magic().len() === 2, `expected 2 magic keys, got ${dof.magic().len()}`);
});

test("magic().is_empty() returns false when magic keys exist", () => {
  const dof = new Dof(maximal);
  assert(!dof.magic().is_empty());
});

test("magic().labels() returns all magic key labels", () => {
  const dof = new Dof(maximal);
  const labels = dof.magic().labels();
  assert(Array.isArray(labels));
  assert(labels.includes("mgc"), `expected 'mgc' in labels: ${labels}`);
  assert(labels.includes("mgc2"), `expected 'mgc2' in labels: ${labels}`);
});

test("magic().key(label) returns a MagicKey", () => {
  const dof = new Dof(maximal);
  const key = dof.magic().key("mgc");
  assert(key !== undefined && key !== null);
  assert(typeof key.label === "function");
});

test("magic().key() returns undefined for unknown label", () => {
  const dof = new Dof(maximal);
  assert(dof.magic().key("nonexistent") === undefined);
});

test("magic().keys() returns a Map", () => {
  const dof = new Dof(maximal);
  const keys = dof.magic().keys();
  assert(keys instanceof Map, "magic().keys() should return a Map");
  assert(keys.has("mgc"), "should have 'mgc'");
  assert(keys.has("mgc2"), "should have 'mgc2'");
});

test("MagicKey label()", () => {
  const dof = new Dof(maximal);
  const key = dof.magic().key("mgc");
  assert(key.label() === "mgc", `expected 'mgc', got ${key.label()}`);
});

test("MagicKey leading() lists all leading characters", () => {
  const dof = new Dof(maximal);
  const leading = dof.magic().key("mgc").leading();
  assert(Array.isArray(leading));
  assert(leading.includes("a"), `expected 'a' in leading: ${leading}`);
  assert(leading.includes("abc"), `expected 'abc' in leading: ${leading}`);
});

test("MagicKey outputs() lists all output strings", () => {
  const dof = new Dof(maximal);
  const outputs = dof.magic().key("mgc").outputs();
  assert(Array.isArray(outputs));
  assert(outputs.includes("b"), `expected 'b' in outputs: ${outputs}`);
});

test("MagicKey rule() looks up a specific rule", () => {
  const dof = new Dof(maximal);
  const key = dof.magic().key("mgc");
  assert(key.rule("a") === "b", `expected rule('a') === 'b', got ${key.rule("a")}`);
  assert(key.rule("abc") === "defghijklmnopqrstuvwxyz");
  assert(key.rule("x") === undefined, "unknown rule should return undefined");
});

test("MagicKey len() and is_empty()", () => {
  const dof = new Dof(maximal);
  const key = dof.magic().key("mgc");
  assert(key.len() === 2, `expected 2 rules, got ${key.len()}`);
  assert(!key.is_empty());
});

test("MagicKey add_rule() and remove_rule()", () => {
  const dof = new Dof(maximal);
  const key = dof.magic().key("mgc");
  key.add_rule("x", "y");
  assert(key.rule("x") === "y");
  assert(key.len() === 3);
  const removed = key.remove_rule("x");
  assert(removed === "y", `expected remove_rule to return 'y', got ${removed}`);
  assert(key.len() === 2);
  assert(key.rule("x") === undefined);
});

// ---------------------------------------------------------------------------
console.log("\nDof – combos (maximal)");

test("combos() returns a Map with layer keys", () => {
  const dof = new Dof(maximal);
  const combos = dof.combos();
  assert(combos instanceof Map, "combos() should return a Map");
  assert(combos.has("main"), "expected 'main' in combos");
  assert(combos.has("shift"), "expected 'shift' in combos");
});

test("combos() entries have positions and output", () => {
  const dof = new Dof(maximal);
  const mainCombos = dof.combos().get("main");
  assert(Array.isArray(mainCombos), "layer combos should be an array");
  assert(mainCombos.length > 0, "main should have combos");
  const entry = mainCombos[0];
  assert(Array.isArray(entry.positions), "combo entry should have positions");
  assert(entry.positions.length > 1, "combo should involve more than one key");
  assert(typeof entry.output === "object", "combo entry should have an output key");
});

test("minimal_valid.dof has no combos", () => {
  const dof = new Dof(minimalValid);
  const combos = dof.combos();
  assert(combos instanceof Map);
  assert(combos.size === 0, "minimal_valid should have no combos");
});

// ---------------------------------------------------------------------------
console.log("\nDof – keys() / DescriptiveKey");

test("keys() returns a non-empty array", () => {
  const dof = new Dof(minimalValid);
  const keys = dof.keys();
  assert(Array.isArray(keys));
  assert(keys.length > 0);
});

test("keys() count matches sum of all layer keys", () => {
  const dof = new Dof(minimalValid);
  const keys = dof.keys();
  const layerKeys = dof.main_layer().keys().length + dof.shift_layer().keys().length;
  assert(keys.length === layerKeys, `keys() count ${keys.length} should match sum of layer keys ${layerKeys}`);
});

test("DescriptiveKey row() and col() are non-negative integers", () => {
  const dof = new Dof(minimalValid);
  for (const key of dof.keys()) {
    assert(Number.isInteger(key.row()) && key.row() >= 0, `row should be non-negative int, got ${key.row()}`);
    assert(Number.isInteger(key.col()) && key.col() >= 0, `col should be non-negative int, got ${key.col()}`);
    break;
  }
});

test("DescriptiveKey pos() matches row() and col()", () => {
  const dof = new Dof(minimalValid);
  const dk = dof.keys()[0];
  const pos = dk.pos();
  // Pos exposes row/col as plain JS properties (from pub struct fields)
  assert(pos.row === dk.row(), `pos.row ${pos.row} should match row() ${dk.row()}`);
  assert(pos.col === dk.col(), `pos.col ${pos.col} should match col() ${dk.col()}`);
});

test("DescriptiveKey output() returns a Key", () => {
  const dof = new Dof(minimalValid);
  const dk = dof.keys()[0];
  const out = dk.output();
  assert(out !== undefined && out !== null);
  assert(typeof out.is_char === "function", "output should be a Key with is_char()");
});

test("DescriptiveKey physical_pos() returns PhysicalKey with x/y/width/height", () => {
  const dof = new Dof(minimalValid);
  const dk = dof.keys()[0];
  const phys = dk.physical_pos();
  assert(typeof phys.x === "number");
  assert(typeof phys.y === "number");
  assert(typeof phys.width === "number");
  assert(typeof phys.height === "number");
});

test("DescriptiveKey layer_name() is a non-empty string", () => {
  const dof = new Dof(minimalValid);
  const dk = dof.keys()[0];
  assert(typeof dk.layer_name() === "string" && dk.layer_name().length > 0);
});

test("DescriptiveKey is_on_left_hand() and is_on_right_hand() are booleans", () => {
  const dof = new Dof(minimalValid);
  const dk = dof.keys()[0];
  assert(typeof dk.is_on_left_hand() === "boolean");
  assert(typeof dk.is_on_right_hand() === "boolean");
});

test("DescriptiveKey is_on_left_hand() and is_on_right_hand() are exclusive", () => {
  const dof = new Dof(minimalValid);
  for (const dk of dof.keys()) {
    const l = dk.is_on_left_hand();
    const r = dk.is_on_right_hand();
    assert(l !== r, `key at (${dk.row()},${dk.col()}) should be on exactly one hand (left=${l}, right=${r})`);
  }
});

test("DescriptiveKey is_on_layer() returns true for own layer", () => {
  const dof = new Dof(minimalValid);
  const dk = dof.keys()[0];
  assert(dk.is_on_layer(dk.layer_name()), "is_on_layer should return true for own layer");
  assert(!dk.is_on_layer("nonexistent"), "is_on_layer should return false for wrong layer");
});

test("DescriptiveKey char_output() on a char key returns its character", () => {
  const dof = new Dof(minimalValid);
  const charKey = dof.keys().find((k) => k.is_char_key());
  assert(charKey !== undefined, "should have at least one char key");
  const ch = charKey.char_output();
  assert(typeof ch === "string" && ch.length === 1, `char_output should be a single char, got ${ch}`);
});

test("DescriptiveKey keypos() has layer and pos", () => {
  const dof = new Dof(minimalValid);
  const dk = dof.keys()[0];
  const kp = dk.keypos();
  assert(typeof kp.layer() === "string");
  assert(typeof kp.pos().row === "number");
});

// ---------------------------------------------------------------------------
console.log("\nDof – get() / tower() / finger()");

test("get('q') returns KeyPos[] for minimal_valid", () => {
  const dof = new Dof(minimalValid);
  const positions = dof.get("q");
  assert(Array.isArray(positions));
  assert(positions.length > 0, "q should be on at least one layer");
  const kp = positions[0];
  assert(typeof kp.layer() === "string");
  assert(typeof kp.pos().row === "number");
  assert(typeof kp.pos().col === "number");
});

test("get() for an absent key returns empty array", () => {
  const dof = new Dof(minimalValid);
  const positions = dof.get("ü");
  assert(Array.isArray(positions));
  assert(positions.length === 0, "ü is not on minimal_valid");
});

test("tower(row, col) returns a Key[]", () => {
  const dof = new Dof(minimalValid);
  const tower = dof.tower(0, 0);
  assert(Array.isArray(tower));
  assert(tower.length > 0, "tower should have at least one key");
  const key = tower[0];
  assert(typeof key === "object" && key !== null);
});

test("tower(row, col) length equals number of layers", () => {
  const dof = new Dof(minimalValid);
  const layerCount = dof.layers().size;
  const tower = dof.tower(0, 0);
  assert(tower.length === layerCount, `tower length ${tower.length} should equal layer count ${layerCount}`);
});

test("finger(row, col) returns a Finger value", () => {
  const dof = new Dof(minimalValid);
  const f = dof.finger(0, 0);
  assert(f !== undefined, "finger(0,0) should return a finger");
});

test("finger() returns undefined for out-of-bounds", () => {
  const dof = new Dof(minimalValid);
  assert(dof.finger(999, 999) === undefined);
});

// ---------------------------------------------------------------------------
console.log("\nDof – swap()");

test("swap() exchanges two keys", () => {
  const dof = new Dof(minimalValid);
  // get_key() returns a serde-serialized plain object {kind, value}
  const before0 = dof.main_layer().get_key(0, 0).value;
  const before1 = dof.main_layer().get_key(0, 1).value;
  dof.swap("main", 0, 0, "main", 0, 1);
  const after0 = dof.main_layer().get_key(0, 0).value;
  const after1 = dof.main_layer().get_key(0, 1).value;
  assert(after0 === before1, `after swap: pos(0,0) should be '${before1}', got '${after0}'`);
  assert(after1 === before0, `after swap: pos(0,1) should be '${before0}', got '${after1}'`);
});

test("swap() on same position is a no-op", () => {
  const dof = new Dof(minimalValid);
  const before = dof.main_layer().get_key(0, 0).value;
  dof.swap("main", 0, 0, "main", 0, 0);
  const after = dof.main_layer().get_key(0, 0).value;
  assert(after === before, "same-position swap should be a no-op");
});

test("swap() throws on nonexistent layer", () => {
  const dof = new Dof(minimalValid);
  let threw = false;
  try {
    dof.swap("nonexistent", 0, 0, "main", 0, 1);
  } catch (e) {
    threw = true;
    assert(e instanceof Error);
  }
  assert(threw, "should throw on nonexistent layer");
});

// ---------------------------------------------------------------------------
console.log("\nDof – Pos / KeyPos");

test("new Pos(row, col) has correct row and col", () => {
  // Pos exposes row/col as plain JS properties (from pub struct fields)
  const pos = new Pos(2, 5);
  assert(pos.row === 2, `expected row 2, got ${pos.row}`);
  assert(pos.col === 5, `expected col 5, got ${pos.col}`);
});

// ---------------------------------------------------------------------------
console.log("\nDof – serialize()");

test("serialize() returns valid JSON", () => {
  const dof = new Dof(minimalValid);
  const json = dof.serialize();
  assert(typeof json === "string");
  const obj = JSON.parse(json);
  assert(typeof obj.name === "string");
});

test("serialize() round-trips cleanly", () => {
  const first = new Dof(minimalValid);
  const json = first.serialize();
  const second = new Dof(json);
  assert(second.serialize() === json, "second serialization should be identical");
});

test("serialize() preserves name and year from maximal", () => {
  const dof = new Dof(maximal);
  const obj = JSON.parse(dof.serialize());
  assert(obj.name === "Qwerty");
  assert(obj.year === 1878);
});

// ---------------------------------------------------------------------------
console.log("\nDofIntermediate");

test("parses minimal_parsable.dof", () => {
  const dof = new DofIntermediate(minimalParsable);
  assert(dof instanceof DofIntermediate);
});

test("serialize() returns valid JSON with name field", () => {
  const dof = new DofIntermediate(minimalParsable);
  const json = dof.serialize();
  assert(typeof json === "string");
  const obj = JSON.parse(json);
  assert(typeof obj.name === "string");
});

test("throws a real Error on invalid JSON", () => {
  let threw = false;
  try {
    new DofIntermediate("not json");
  } catch (e) {
    threw = true;
    assert(e instanceof Error, "should throw Error, not string");
    assert(e.message.length > 0);
  }
  assert(threw, "should have thrown");
});

test("accepts minimal_parsable which Dof rejects", () => {
  assert(new DofIntermediate(minimalParsable) instanceof DofIntermediate);
  let threw = false;
  try {
    new Dof(minimalParsable);
  } catch (_) {
    threw = true;
  }
  assert(threw, "Dof should reject minimal_parsable");
});

// ---------------------------------------------------------------------------
console.log(`\n${passed} passed, ${failed} failed`);
if (failed > 0) process.exit(1);
