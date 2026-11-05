// usage: node e2e/slice.mjs <base url> <sample.png> <shots dir>  -> F01 end to end in Chromium
import fs from 'node:fs'; import path from 'node:path'; import { createRequire } from 'node:module';
const { chromium } = createRequire(import.meta.url)('playwright');
const [base, sample, shots] = process.argv.slice(2);
const b = await chromium.launch();
const errors = [];
const check = (ok, msg) => { console.log(`${ok ? 'PASS' : 'FAIL'} ${msg}`); if (!ok) process.exitCode = 1; };

for (const vp of [{ name: 'desktop', width: 1440, height: 900 }, { name: 'mobile', width: 390, height: 844 }]) {
  const ctx = await b.newContext({ viewport: vp, acceptDownloads: true });
  const p = await ctx.newPage();
  p.on('console', (m) => { if (m.type() === 'error') errors.push(`${vp.name}: ${m.text()}`); });
  p.on('pageerror', (e) => errors.push(`${vp.name}: ${e.message}`));
  p.on('response', (r) => { if (r.status() >= 500) errors.push(`${vp.name}: ${r.status()} ${r.url()}`); });
  await p.goto(base);
  if (vp.name === 'desktop') await p.screenshot({ path: path.join(shots, 'S01.png') });

  // invalid file -> inline error, stays on S01
  await p.setInputFiles('#file', { name: 'notas.txt', mimeType: 'text/plain', buffer: Buffer.from('hola') });
  check(await p.locator('#s01-error').isVisible(), `${vp.name}: rejects a .txt with a visible error`);

  // F01: upload -> processing -> result
  await p.setInputFiles('#file', sample);
  check(await p.locator('#s02').isVisible(), `${vp.name}: shows the processing screen`);
  if (vp.name === 'desktop') await p.screenshot({ path: path.join(shots, 'S02.png') });
  const t0 = Date.now();
  await p.locator('#s03').waitFor({ state: 'visible', timeout: 60_000 });
  console.log(`     ${vp.name}: result in ${((Date.now() - t0) / 1000).toFixed(1)} s, stats: ${await p.locator('#s03-stats').textContent()}`);
  await p.waitForFunction(() => document.querySelector('#s03-vec').complete && document.querySelector('#s03-vec').naturalWidth > 0);
  check(true, `${vp.name}: vector image rendered`);
  check(!(await p.locator('#s02').isVisible()) && !(await p.locator('#s01').isVisible()), `${vp.name}: only the result screen is showing`);
  await p.screenshot({ path: path.join(shots, vp.name === 'desktop' ? 'S03.png' : 'S03-mobile.png'), fullPage: true });

  // S05: download SVG
  const [dl] = await Promise.all([p.waitForEvent('download'), p.click('#s05-svg')]);
  const file = await dl.path();
  const svg = fs.readFileSync(file, 'utf8');
  check(dl.suggestedFilename().endsWith('.svg') && svg.includes('<svg') && svg.includes('viewBox'), `${vp.name}: downloads a valid SVG (${dl.suggestedFilename()}, ${Math.round(svg.length / 1024)} KB)`);

  // keyboard: "Otra imagen" returns to S01
  await p.click('#s03-new');
  check(await p.locator('#s01').isVisible(), `${vp.name}: "Otra imagen" goes back to upload`);
  await ctx.close();
}
check(errors.length === 0, `no console errors or 5xx${errors.length ? ': ' + errors.join(' | ') : ''}`);
await b.close();
