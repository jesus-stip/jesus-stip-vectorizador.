// usage: node wasm-check.mjs <engine.wasm> <in.png> <out.svg>  -> runs the browser engine in Node
import fs from 'node:fs';
import { PNG } from 'pngjs';
const [wasmPath, inp, out] = process.argv.slice(2);
const mod = await WebAssembly.compile(fs.readFileSync(wasmPath));
console.log('imports:', WebAssembly.Module.imports(mod).map(i => `${i.module}.${i.name}`).join(', ') || 'none');
const { exports: e } = await WebAssembly.instantiate(mod, {});
const png = PNG.sync.read(fs.readFileSync(inp));
const len = png.width * png.height * 4;
const ptr = e.alloc(len);
new Uint8Array(e.memory.buffer, ptr, len).set(png.data);
const colors = e.suggest_colors(ptr, png.width, png.height);
const t0 = performance.now();
const cap = +(process.argv[5] || 1_600_000);
const rc = e.trace(ptr, png.width, png.height, colors, 1, cap);
const ms = Math.round(performance.now() - t0);
const text = new TextDecoder().decode(new Uint8Array(e.memory.buffer, e.result_ptr(), e.result_len()));
e.dealloc(ptr, len);
if (rc !== 0) { console.error('error:', text); process.exit(1); }
fs.writeFileSync(out, text);
console.log(JSON.stringify({ colors, ms, paths: (text.match(/<path/g) || []).length, kb: Math.round(text.length / 1024) }));
