// Node.js integration test for the libdof-wasm npm package.
// Requires: wasm-pack build --target nodejs --out-dir pkg-node
import { parse_dof, parse_dof_intermediate } from "../pkg-node/libdof_wasm.js";
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

console.log("parse_dof");

test("parses minimal_valid.dof", () => {
  const result = parse_dof(minimalValid);
  const obj = JSON.parse(result);
  assert(typeof obj.name === "string", "name should be a string");
  assert(obj.fingering !== undefined, "fingering should be present");
    assert(obj.layers !== undefined, "layers should be present");
    console.log(result, "\n", obj)
});

test("parses maximal.dof", () => {
  assert(parse_dof(maximal) !== undefined);
});

test("round-trips cleanly", () => {
  const first = parse_dof(minimalValid);
  const second = parse_dof(first);
  assert(first === second, "round-trip should be idempotent");
});

test("throws a real Error on invalid JSON", () => {
  let threw = false;
  try {
    parse_dof("not json");
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
    parse_dof('{"name":"test"}');
  } catch (e) {
    threw = true;
  }
  assert(threw, "should have thrown on incomplete dof");
});

console.log("\nparse_dof_intermediate");

test("parses minimal_parsable.dof", () => {
  const result = parse_dof_intermediate(minimalParsable);
  const obj = JSON.parse(result);
  assert(typeof obj.name === "string", "name should be a string");
});

test("throws on invalid JSON", () => {
  let threw = false;
  try {
    parse_dof_intermediate("not json");
  } catch (e) {
    threw = true;
  }
  assert(threw, "should have thrown");
});

console.log(`\n${passed} passed, ${failed} failed`);
if (failed > 0) process.exit(1);
