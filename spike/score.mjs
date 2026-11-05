// usage: node score.mjs <samples/png dir> <out dir>  -> renders every SVG, writes score.json
import fs from 'node:fs'; import path from 'node:path';
import { PNG } from 'pngjs';
import { createRequire } from 'node:module';
const { chromium } = createRequire(import.meta.url)('playwright');
const [IN, OUT] = process.argv.slice(2);
const times = Object.fromEntries(fs.readFileSync(path.join(OUT, 'times.txt'), 'utf8').trim().split('\n')
  .map(l => l.split(' ')).map(([n, e, ms]) => [`${n}.${e}`, +ms]));
const browser = await chromium.launch(); const page = await browser.newPage();
const rows = [];
for (const svgFile of fs.readdirSync(OUT).filter(f => f.endsWith('.svg')).sort()) {
  const [n, engine] = svgFile.replace('.svg', '').split('.');
  const orig = PNG.sync.read(fs.readFileSync(path.join(IN, n + '.png')));
  const svg = fs.readFileSync(path.join(OUT, svgFile), 'utf8');
  await page.setViewportSize({ width: orig.width, height: orig.height });
  await page.setContent(`<html><body style="margin:0;background:#fff">
    <img style="display:block;width:${orig.width}px;height:${orig.height}px"
      src="data:image/svg+xml;base64,${Buffer.from(svg).toString('base64')}"></body></html>`);
  const shot = path.join(OUT, svgFile.replace('.svg', '.png'));
  await page.screenshot({ path: shot });
  const r = PNG.sync.read(fs.readFileSync(shot));
  let err = 0, px = 0;
  for (let i = 0; i < orig.data.length; i += 4) {
    const a = orig.data[i + 3] / 255;                    // composite original on white, like the render
    for (let c = 0; c < 3; c++) err += Math.abs(orig.data[i + c] * a + 255 * (1 - a) - r.data[i + c]);
    px++;
  }
  rows.push({ image: n, engine, mae: +(err / px / 3).toFixed(2), paths: (svg.match(/<path/g) || []).length,
    kb: Math.round(svg.length / 1024), ms: times[`${n}.${engine}`] });
}
await browser.close();
fs.writeFileSync(path.join(OUT, 'score.json'), JSON.stringify(rows, null, 1));
console.table(rows);
