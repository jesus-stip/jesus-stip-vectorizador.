// usage: node render.mjs <in.svg> <out.png> <w> <h> : renders an SVG on white with Chromium
import fs from 'node:fs'; import { createRequire } from 'node:module';
const { chromium } = createRequire(import.meta.url)('playwright');
const [inp, out, w, h] = process.argv.slice(2);
const b = await chromium.launch(); const p = await b.newPage({ viewport: { width: +w, height: +h } });
await p.setContent(`<body style="margin:0;background:#fff"><img style="display:block;width:${w}px;height:${h}px" src="data:image/svg+xml;base64,${fs.readFileSync(inp).toString('base64')}">`);
await p.screenshot({ path: out }); await b.close();
