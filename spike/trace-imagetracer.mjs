// usage: node trace-imagetracer.mjs <in.png> <out.svg> <preset>
import fs from 'node:fs';
import { PNG } from 'pngjs';
import ImageTracer from 'imagetracerjs';
const [inp, out, preset] = process.argv.slice(2);
const png = PNG.sync.read(fs.readFileSync(inp));
const presets = {
  default: { numberofcolors: 16 },
  detailed: { numberofcolors: 32, ltres: 0.5, qtres: 0.5, pathomit: 4, colorquantcycles: 3, blurradius: 0, roundcoords: 1 },
};
const t0 = performance.now();
const svg = ImageTracer.imagedataToSVG({ width: png.width, height: png.height, data: png.data }, presets[preset]);
const ms = Math.round(performance.now() - t0);
fs.writeFileSync(out, svg);
console.log(JSON.stringify({ ms }));
