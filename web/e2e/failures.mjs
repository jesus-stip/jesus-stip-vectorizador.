// usage: node e2e/failures.mjs <dist dir> <base url> <sample.png> : the app must explain, never sit silent
import path from 'node:path'; import { createRequire } from 'node:module';
const { chromium } = createRequire(import.meta.url)('playwright');
const [dist, base, sample] = process.argv.slice(2);
const b = await chromium.launch();
const check = (ok, msg) => { console.log(`${ok ? 'PASS' : 'FAIL'} ${msg}`); if (!ok) process.exitCode = 1; };

// 1. opened by double click from disk
let p = await b.newPage();
await p.goto('file://' + path.resolve(dist, 'index.html'));
await p.locator('#boot-error').waitFor({ state: 'visible', timeout: 6000 }).catch(() => {});
check(/doble clic/.test(await p.locator('#boot-error').textContent()), 'file:// shows the "doble clic" explanation');
await p.close();

// 2. server does not deliver the engine (.wasm 404)
p = await b.newPage();
await p.route('**/*.wasm', (r) => r.fulfill({ status: 404, body: 'not found' }));
await p.goto(base);
await p.setInputFiles('#file', sample);
await p.locator('#s01-error').waitFor({ state: 'visible', timeout: 20000 }).catch(() => {});
const t2 = await p.locator('#s01-error').textContent();
check(/motor/.test(t2 || ''), `missing .wasm explains itself: "${t2}"`);
await p.close();

// 3. server does not deliver the worker script
p = await b.newPage();
await p.route('**/worker-*.js', (r) => r.fulfill({ status: 404, body: 'not found' }));
await p.goto(base);
await p.setInputFiles('#file', sample);
await p.locator('#s01-error').waitFor({ state: 'visible', timeout: 20000 }).catch(() => {});
const t3 = await p.locator('#s01-error').textContent();
check(/motor/.test(t3 || ''), `missing worker explains itself: "${t3}"`);
await b.close();
