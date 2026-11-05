# Engine spike (2026-11-05)

Question: which open-source tracer goes behind the `trace()` adapter in architecture.md.
Inputs: the user's 5 real samples (apparel designs: flat color, black outlines, big lettering).
Samples 1 and 3 are clean design files; 2, 4 and 5 are photos of printed garments (fabric texture, folds).
Sample images are not in the repo (`.gitignore`). Scripts are in `spike/`.

## Candidates

| id | engine | settings |
| --- | --- | --- |
| itj-default | imagetracerjs 1.2.6 (Unlicense) | 16 colors |
| itj-detailed | imagetracerjs 1.2.6 | 32 colors, ltres/qtres 0.5, pathomit 4 |
| vt-poster / vt-medium / vt-high | VTracer 0.6.5 (MIT OR Apache-2.0, crates.io, publisher tyt2y3 = visioncortex) | preset poster; spline f4 p6 g16; spline f2 p7 g8 |
| vtq16 / vtq32 | **preprocess + VTracer**: 2x Lanczos upscale, 3px median, median-cut to 16 / 32 colors (no dithering), then VTracer stacked spline f4 p8 g1 c60 | |

## Numbers

MAE = mean absolute RGB error of the rendered SVG against the original, 0..255, lower is better.
It rewards copying JPEG noise, so read it together with the path count and the crops.

| image | itj-default | itj-detailed | vt-high | vtq16 | vtq32 |
| --- | --- | --- | --- | --- | --- |
| 1 (Peanuts, many colors) | 12.22 / 21465 paths | 6.61 / 84719 | 8.82 / 13314 | 9.65 / 7011 | 7.76 / 14679 |
| 2 (shirt photo) | 16.67 / 11338 | 7.56 / 47706 | 11.57 / 4883 | 9.57 / 6472 | 8.74 / 10125 |
| 3 (Snoopy, clean art) | 8.28 / 13481 | 7.04 / 53476 | 8.75 / 7179 | 8.14 / 3570 | 7.13 / 7946 |
| 4 (shirt photo) | 13.02 / 3760 | 9.23 / 17854 | 11.88 / 2801 | 11.90 / 2394 | 10.25 / 3529 |
| 5 (shirt photo) | 14.46 / 3373 | 9.59 / 14752 | 13.33 / 2214 | 12.85 / 2028 | 11.28 / 2944 |

Times (native CLI, this container): VTracer 0.2 to 11 s depending on size and color count; imagetracerjs 0.3 to 3 s.

## What the crops show (samples/out/comparacion-*.png, local only)

- **imagetracerjs:** jagged stair-step edges, muddy colors at default (brown outlines, grey skin).
  "detailed" is close in MAE only because it emits 50k to 85k tiny shapes: unusable for print or editing.
- **VTracer raw:** smooth curves and true colors, but light halos between black outlines and fills
  (JPEG and anti-aliasing fringe traced as its own band) and smeared patches.
- **VTracer + preprocessing (vtq16):** halos gone, flat solid fills, eyes and small lettering kept. Best overall.
  The first attempt (1x, speckle filter 6, mode filter) deleted Charlie Brown's eyes and eroded the text:
  small features need the 2x upscale and a small speckle filter.
- **Limit found:** on sample 1 (many hues) 16 colors merge pinks into red and dark green into grey.
  32 colors fixes most of it. The color count must come from auto-detect, not a constant.
- Garment photos (2, 4, 5) trace the fabric texture too. Expected: they are not design files.

## Decision

**VTracer 0.6.5, compiled to WASM by us, behind a preprocessing step** (upscale, denoise, palette
quantization). imagetracerjs is dropped (kept only as an emergency fallback).

WASM check: a minimal `wasm-bindgen` wrapper around `vtracer::convert` (`spike/wasm`) builds for
`wasm32-unknown-unknown` in release at 150 KB. The build step also needs `wasm-bindgen-cli` for the JS glue.

## Carried into the build

1. Preprocessing in the worker: 2x upscale when the short side is under ~1500 px, 3px median,
   k-means / median-cut to N colors, no dithering, alpha thresholded at 128.
2. Auto-detect picks N by counting distinct hue clusters: 12 to 16 for logos and lettering, 24 to 32 for busy art.
   Detail low / medium / high maps to N and to the speckle filter (f 8 / 4 / 2 at 2x scale).
3. Output SVG gets a `viewBox` plus the original's width and height (traced at 2x, shown at 1x).
4. Speed: 11 s worst case (900 px, 32 colors) is too slow. Profile in WASM, cap at 4 MP, show progress.
5. Next quality targets: snap near-black outlines to one exact color; merge tiny same-color islands.
