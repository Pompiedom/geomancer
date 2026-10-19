// Calls the built plug-ins the way VectorCraft's host does (ABI v1) and writes tests/preview.html.
// Usage: node tests/run.mjs
import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("..", import.meta.url));
let failures = 0;
const check = (ok, msg) => { if (!ok) { failures++; console.log("FAIL", msg); } };

function load(name) {
  const bytes = readFileSync(`${root}dist/geomancer-${name}.wasm`);
  const mod = new WebAssembly.Module(bytes);
  check(WebAssembly.Module.imports(mod).length === 0, `${name}: has imports`);
  const exports = WebAssembly.Module.exports(mod).map(e => e.name);
  for (const e of ["memory", "vc_abi_version", "vc_manifest", "vc_alloc", "vc_run"]) check(exports.includes(e), `${name}: missing export ${e}`);
  return mod;
}

const unpack = (inst, v) => {
  v = BigInt.asUintN(64, v);
  const ptr = Number(v & 0xffffffffn), len = Number(v >> 32n);
  return JSON.parse(new TextDecoder().decode(new Uint8Array(inst.exports.memory.buffer, ptr, len)));
};

function manifest(mod) {
  const inst = new WebAssembly.Instance(mod, {});
  check(inst.exports.vc_abi_version() === 1, "abi version");
  const m = unpack(inst, inst.exports.vc_manifest());
  check(/^[A-Za-z0-9._-]{1,64}$/.test(m.id), `bad id ${m.id}`);
  for (const k of Object.keys(m.params)) check(/^[A-Za-z0-9][A-Za-z0-9_]*$/.test(k) && !["id", "ids", "params", "preview"].includes(k), `bad param ${k}`);
  return m;
}

// Fresh instance per run, as the host does; missing params take their defaults.
function run(mod, man, input, params) {
  const inst = new WebAssembly.Instance(mod, {});
  const full = {};
  for (const [k, s] of Object.entries(man.params)) full[k] = params[k] ?? s.default;
  full._context = params._context;
  const enc = s => new TextEncoder().encode(JSON.stringify(s));
  const [a, b] = [enc(input), enc(full)];
  const pa = inst.exports.vc_alloc(a.length); new Uint8Array(inst.exports.memory.buffer, pa, a.length).set(a);
  const pb = inst.exports.vc_alloc(b.length); new Uint8Array(inst.exports.memory.buffer, pb, b.length).set(b);
  const r = inst.exports.vc_run(pa, a.length, pb, b.length);
  check(r >= 0n, `vc_run returned ${r}`);
  return unpack(inst, r);
}

const fmt = p => `${p[0].toFixed(2)},${p[1].toFixed(2)}`;
function d(path) {
  let s = "";
  for (const sp of path.subpaths) {
    const A = sp.anchors;
    for (const a of A.concat(sp.closed ? [A[0]] : []).map((a, i, arr) => [a, i])) {
      const [an, i] = a;
      if (i === 0) { s += `M${fmt(an.p)}`; continue; }
      const prev = (sp.closed && i === A.length) ? A[A.length - 1] : A[i - 1];
      for (const p of [an.p, an.in ?? an.p, prev.out ?? prev.p].flat()) check(Number.isFinite(p) && Math.abs(p) < 4e6, "coordinate out of range");
      s += `C${fmt(prev.out ?? prev.p)} ${fmt(an.in ?? an.p)} ${fmt(an.p)}`;
    }
    if (sp.closed) s += "Z";
  }
  return s;
}

const ctx = { mode: "filter", bounds: null, artboard: [0, 0, 300, 300] };
const cards = [];
const card = (title, paths, fill) => cards.push(`<figure><svg viewBox="0 0 300 300">${paths.map(p => `<path d="${d(p)}" fill="${fill ? "#222" : "none"}" fill-rule="nonzero" stroke="${fill ? "none" : "#222"}" stroke-width="1.2"/>`).join("")}</svg><figcaption>${title}</figcaption></figure>`);

// Generators
const spiral = load("spiral"), rose = load("rose"), rosette = load("rosette");
const ms = manifest(spiral), mr = manifest(rose), mt = manifest(rosette);
check(ms.kind === "filter" && mr.kind === "filter" && mt.kind === "effect", "kinds");

for (const [title, p] of [["Spiral (defaults)", {}], ["Spiral Fermat, 4 turns", { fermat: true, turns: 4, outerRadius: 120 }], ["Spiral Fermat, inner 30", { fermat: true, turns: 3, innerRadius: 30, outerRadius: 120 }], ["Spiral inner 40, ccw", { innerRadius: 40, outerRadius: 130, turns: 3, direction: "counterclockwise" }]]) {
  const out = run(spiral, ms, { objects: [] }, { ...p, _context: ctx });
  check(out.objects?.length === 1 && out.objects[0].name === "Spiral", `spiral output ${JSON.stringify(out).slice(0, 200)}`);
  card(title, [out.objects[0].path]);
}
check(run(spiral, ms, { objects: [] }, { innerRadius: 200, outerRadius: 100, _context: ctx }).error, "spiral inner > outer should error");

for (const [title, p, fill] of [["Rose n=4 (8 petals)", {}, false], ["Rose n=5, filled", { n: 5, fill: true }, true], ["Rose n=7 d=3", { n: 7, d: 3, radius: 120 }, false]]) {
  const out = run(rose, mr, { objects: [] }, { ...p, _context: ctx });
  card(title, [out.objects[0].path], fill);
}

// Generator with a selection: keeps it, centres on its bounds.
const sel = run(rose, mr, { objects: [{ id: 7, type: "path", path: { subpaths: [] }, bounds: [0, 0, 100, 100] }] }, { _context: { ...ctx, bounds: [0, 0, 100, 100] } });
check(sel.objects[0].id === 7 && sel.objects.length === 2, "selection kept");

// Live effect on a petal
const petal = { subpaths: [{ closed: true, anchors: [{ p: [150, 150], in: [175, 115], out: [125, 115] }, { p: [150, 70], in: [120, 85], out: [180, 85] }] }] };
const leaf = { subpaths: [{ closed: true, anchors: [{ p: [150, 150], in: [185, 120], out: [135, 120] }, { p: [165, 70], in: [125, 80], out: [190, 90] }] }] };
for (const [title, shape, p] of [["Rosette 8 copies", petal, { copies: 8 }], ["Rosette 6 + mirror (asymmetric leaf)", leaf, { copies: 6, mirror: true }], ["Rosette 12, centre offset", petal, { copies: 12, offsetY: 20 }]]) {
  const out = run(rosette, mt, { objects: [{ type: "path", path: shape, fillRule: "nonzero", bounds: [125, 70, 190, 150] }] }, { ...p, _context: { mode: "effect", bounds: [125, 70, 190, 150] } });
  check(out.objects?.length === 1, "rosette output");
  card(title, [out.objects[0].path], true);
}

writeFileSync(`${root}tests/preview.html`, `<!doctype html><meta charset="utf-8"><title>Geomancer preview</title>
<style>body{font:14px system-ui;background:#f4f1ea;margin:16px}main{display:grid;grid-template-columns:repeat(auto-fill,minmax(220px,1fr));gap:12px}figure{margin:0;background:#fff;border-radius:8px;padding:8px}svg{width:100%;height:auto}figcaption{text-align:center;color:#555}</style>
<h1>Geomancer: preview of the wasm output</h1><main>${cards.join("")}</main>`);
console.log(failures ? `${failures} failure(s)` : `all checks passed (${cards.length} previews)`);
process.exit(failures ? 1 : 0);
