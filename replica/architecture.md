# Architecture: working name "Vectorizer" (a rebuild of Vector Magic's core features)

Constraints that drive everything:
- **Shared cPanel hosting, on a subdomain.** Shared hosts run PHP + MySQL, kill long-running processes,
  usually block `exec()` and native binaries, and give you no Node.js you can count on.
- **Internal use** for the user's own business: no billing, no public sign up.

Decision: **a static web app that vectorizes in the browser.** The tracing engine runs as WebAssembly / JS
inside a Web Worker on the user's machine. The server only serves files. Nothing to run, nothing for the
host to kill, and client images never leave the computer.

## Stack

| layer | choice | why |
| --- | --- | --- |
| web | Vite + TypeScript, no framework, built to static HTML/JS/CSS | 7 screens and one main view; a framework adds weight and nothing else. Output uploads straight to `public_html/<subdomain>` |
| styling | plain CSS with custom-property tokens from replica-design | no build-time CSS tooling to maintain |
| engine | **adapter with two candidates, picked by a spike** (see below) | output quality is the product, so it gets measured, not assumed |
| PDF export | jsPDF 4.x + svg2pdf.js 2.x (both MIT) | vector PDF from our SVG, in the browser |
| EPS export | own small writer: SVG path data (M L C Z, fill colors) -> PostScript | our output only ever uses filled paths, so this is ~150 lines; no good maintained library exists |
| storage | IndexedDB in the browser (history of images and results) | no database on the server; works per device |
| auth | cPanel **Directory Privacy** (HTTP Basic auth via `.htaccess`) on the subdomain | internal tool: zero auth code, the user manages passwords in cPanel |
| payments | none | internal use (features.csv: skip) |
| email | none | not needed in the core loop |
| jobs | none server side; one Web Worker per vectorization | |
| HTTPS | cPanel AutoSSL (Let's Encrypt) on the subdomain | needed for some browser APIs and for Basic auth not to leak |
| hosting | the user's shared cPanel, subdomain such as `vector.yourdomain.com` | what they already have |

### Engine candidates (spike in /replica-build, step 1)

| candidate | license | how it ships | notes |
| --- | --- | --- | --- |
| A. VTracer (upstream `visioncortex/vtracer`, Rust) compiled to WASM ourselves with `wasm-pack` | check the LICENSE file in the upstream repo before use (believed MIT) | `.wasm` + JS glue, static | color tracing with hierarchical clustering, smooth splines; strongest quality candidate. Do **not** use the third-party npm `vtracer-wasm@0.1.0`: no license listed |
| B. imagetracerjs 1.2.6 | Unlicense (public domain) | one JS file | simpler, pure JS, palette-based; baseline and fallback |

Pick by tracing 6 to 10 of the user's own real images (logos, a photo, a scan) with both and comparing
side by side: edge cleanliness on anti-aliased logos, shape count, speed. The engine sits behind one interface,
so swapping it later changes no screen code:

```ts
interface TraceSettings {
  imageType: 'photo' | 'artwork_blended' | 'artwork_sharp' | 'scan_color' | 'scan_bw';
  detail: 'low' | 'medium' | 'high';
  colorMode: 'unlimited' | 'custom';
  palette?: string[];          // 2..12 hex colors when colorMode = 'custom'
}
interface TraceResult { svg: string; shapeCount: number; ms: number; }
trace(pixels: ImageData, s: TraceSettings): Promise<TraceResult>   // runs in the worker
detect(pixels: ImageData): TraceSettings                            // auto settings, heuristics
```

`detail` and `imageType` map onto each engine's own knobs (filter speckle, color precision, layer
difference, corner threshold, path mode). That mapping table is written during the spike.

## Schema

No server database in v1. Data lives in **IndexedDB** (`vectorizer` DB, version 1), 3 object stores.
Access rules: none needed (single device, behind Basic auth). See `replica/schema.ts.md` for the exact shape.

```
images          key id (uuid)
                name, mime, width, height, bytes, createdAt (ISO, UTC),
                original: Blob, thumb: Blob (256px PNG)
                index: createdAt

vectorizations  key id (uuid)
                imageId -> images.id (delete cascades in code)
                settings: TraceSettings, auto: boolean,
                status: 'done' | 'failed', error?: string,
                svg: string, shapeCount: number, ms: number, createdAt
                index: imageId, [imageId, createdAt]

prefs           key 'singleton'
                lastSettings?: TraceSettings, maxMegapixels: number (default 4)
```

Exports (SVG/PDF/EPS) are generated on download from `vectorizations.svg` and never stored.
If the user later wants history shared across machines or people: add a small PHP + MySQL endpoint on the
same cPanel (`upload.php`, `list.php`), same fields. Not in v1.

## API

No server routes. The "API" is the worker message protocol and three local modules.

| message / call | does | who | input | output | flow |
| --- | --- | --- | --- | --- | --- |
| worker `detect` | guess image type and settings | main thread | ImageData | TraceSettings | F01 |
| worker `trace` | vectorize | main thread | ImageData + TraceSettings | TraceResult or error | F01, F02 |
| worker `cancel` | stop a running trace (terminate + respawn worker) | main thread | job id | ack | F02 |
| `export.svg(result)` | clean + download SVG | UI | svg | Blob | F01 |
| `export.pdf(result)` | SVG -> vector PDF (jsPDF + svg2pdf) | UI | svg, w, h | Blob | F01 |
| `export.eps(result)` | SVG paths -> PostScript EPS with bounding box | UI | svg, w, h | Blob | F01 |
| `store.*` | save / list / get / delete images and results | UI | | | F03 |

Webhooks: none. Jobs: none. External APIs: none.

Hosting files shipped with the build:
- `.htaccess`: `AddType application/wasm .wasm`, long cache on hashed assets, no cache on `index.html`,
  redirect http -> https. (cPanel Directory Privacy writes its own auth lines into this file; keep both.)

## The parts that bite

- **Big images in the browser:** a 6000x4000 photo is 96 MB of RGBA and tracing it can freeze or crash the tab.
  Downscale to `maxMegapixels` (default 4) before tracing, warn when that happens, always trace in the worker.
- **WASM MIME type on cPanel:** without `application/wasm` some Apache setups serve it as octet-stream and
  streaming compile fails. The `.htaccess` above fixes it; the loader falls back to non-streaming instantiate.
- **Tracing quality:** the whole product. Measured in the spike and again by /replica-diff on the same images.
- **Auto-detect:** heuristics on the downscaled pixels: distinct color count after quantization, share of
  anti-aliased edge pixels, noise level, near-grayscale check. Wrong guesses are cheap because F02 exists.
- **Transparent PNGs:** keep alpha as "no shape", not as a white or black region.
- **Animated GIFs:** first frame only, say so.
- **EPS correctness:** test opening in Illustrator / Inkscape / CorelDRAW (whatever the business uses).
- **IndexedDB limits:** browser quota and eviction. Keep originals and results, show used space, allow delete.
  It is per browser: clearing site data wipes the history. Say so in the UI.
- **Browser support:** current Chrome, Edge, Firefox, Safari. WASM + module workers are fine there.
- **Privacy:** images never leave the device. That is a real upside over the original; keep it true.
- Not applicable: time zones, webhooks, payments, email, multi-tenancy, server-side rate limits.

## Build order

1. **Spike + vertical slice (S01 -> S03 -> S05, F01):** both engines behind the adapter, upload a file,
   trace with fixed settings in a worker, show the SVG, download SVG. Pick the engine on the user's images.
   Deploy it to the subdomain once, to prove the hosting (`.wasm` MIME, HTTPS, Basic auth).
2. **Must-haves:** drag and drop + format checks (S01), processing state + cancel (S02), auto-detect,
   flip / side-by-side viewer with zoom and pan (S03), hand-pick settings: type, detail, colors, palette
   2 to 12 (S04), re-vectorize, PDF export, downscale guard.
3. **Should-haves:** EPS export, history in IndexedDB (S07) with delete, palette editing (eyedropper).
   S06 (sign in) is replaced by cPanel Directory Privacy.
4. **Could-haves:** DXF export, batch of several images.
5. **Fixes from replica-entrepreneur**, once it has run (e.g. no surprise paywall: here there is none at all).
