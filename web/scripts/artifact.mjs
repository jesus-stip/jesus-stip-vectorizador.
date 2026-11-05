// usage: node scripts/artifact.mjs  (after `npm run build`)
// Turns dist/ into artifact/: a claude.ai Artifact page (content only, CSS inlined) plus its files.
import fs from 'node:fs'; import path from 'node:path';
const dist = 'dist', out = 'artifact';
fs.rmSync(out, { recursive: true, force: true });
fs.mkdirSync(path.join(out, 'assets'), { recursive: true });
let html = fs.readFileSync(path.join(dist, 'index.html'), 'utf8');
const head = html.slice(html.indexOf('<head>') + 6, html.indexOf('</head>'));
const body = html.slice(html.indexOf('<body>') + 6, html.indexOf('</body>'));
const title = head.match(/<title>.*?<\/title>/s)[0];
const fonts = [...head.matchAll(/<link rel="(?:preconnect|stylesheet)" href="https:\/\/fonts\.[^>]*>/g)].map((m) => m[0]).join('\n');
const cssHref = head.match(/<link rel="stylesheet" crossorigin href="\.\/(assets\/[^"]+\.css)">/)[1];
const css = fs.readFileSync(path.join(dist, cssHref), 'utf8');
const script = head.match(/<script type="module" crossorigin src="\.\/(assets\/[^"]+\.js)"><\/script>/);
const page = [
  title,
  '<meta name="claude:viewport-fit-sides" content="cover">',
  fonts,
  `<style>${css}</style>`,
  body.trim(),
  `<script type="module" src="${script[1]}"></script>`,
].join('\n');
fs.writeFileSync(path.join(out, 'index.html'), page + '\n');
const files = {};
for (const f of fs.readdirSync(path.join(dist, 'assets'))) {
  if (f.endsWith('.css')) continue;
  fs.copyFileSync(path.join(dist, 'assets', f), path.join(out, 'assets', f));
  files[`assets/${f}`] = f.endsWith('.wasm') ? { from: `assets/${f}`, contentType: 'application/wasm' } : `assets/${f}`;
}
fs.copyFileSync(path.join(dist, 'ejemplo.jpg'), path.join(out, 'ejemplo.jpg'));
files['ejemplo.jpg'] = 'ejemplo.jpg';
fs.writeFileSync(path.join(out, 'files.json'), JSON.stringify(files, null, 1));
console.log(JSON.stringify(files, null, 1));
