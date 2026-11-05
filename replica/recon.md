# Recon map: Vector Magic (web, Online Edition)

Scope: the core loop: upload a bitmap, automatic vectorization, adjust image type / detail / colors, compare the result with the original, download SVG / EPS / PDF.
For: the user's own business (internal use, not resold).
Date: 2026-11-05

> **Recon limits.** vectormagic.com could not be opened from this session (network policy blocked it).
> Everything below comes from search results that cite their public help pages, plus third-party listings and reviews.
> No screen was observed directly and `replica/screens/` is empty. Screen names and states are **inferred**.
> To firm it up: allow `vectormagic.com` in the environment's network settings, or drop screenshots from your own use into `replica/screens/`.
> Terms of service not checked (page unreachable). Read them before using your own account as a reference.

## Sources

| # | source | URL | notes |
| --- | --- | --- | --- |
| 1 | marketing site | https://es.vectormagic.com/ | the user's link; blocked from this session |
| 2 | tutorial: how to use | https://vectormagic.com/support/tutorials/how-to-use-vector-magic | auto-detected settings, custom palettes of 2 to 12 colors |
| 3 | tutorial: artwork / logos | https://vectormagic.com/support/tutorials/artwork-logos | segmentation editor, anti-aliasing handling (old page) |
| 4 | tutorial: photos | https://vectormagic.com/support/tutorials/photos | high-contrast photos; "logo with blended edges" mode |
| 5 | tutorial: scans | https://vectormagic.com/support/tutorials/scans | scan modes, color vs monochrome |
| 6 | tips | https://vectormagic.com/support/tutorials/tips | detail level, unlimited vs custom colors |
| 7 | file formats | https://vectormagic.com/support/file_formats | in: JPG PNG BMP GIF (desktop adds TIFF); out: SVG EPS PDF (desktop adds AI DXF) |
| 8 | pricing | https://vectormagic.com/pricing | online $9.95/mo; desktop $295 one-time; free preview, paid download |
| 9 | edit URL pattern | https://vectormagic.com/images/{id}/edit/{token} | each image has its own URL with an edit page |
| 10 | Trustpilot | https://ca.trustpilot.com/review/www.vectormagic.com | 2.2/5; paywall surprise only after vectorizing |
| 11 | Capterra | https://www.capterra.com/p/178297/Vector-Magic/ | 4.6/5 (23 reviews); no free plan or trial |
| 12 | SpotSaas | https://www.spotsaas.com/product/vector-magic | 3.1/5; limited editing and cost are the complaints; output quality is the praise |
| 13 | 2007 blog walkthrough | https://blog.jtbworld.com/2007/11/online-vectorization-using-vectormagic.html | browse, then upload, 12-color default palette (dated) |

## Core loop

Upload a raster image and get a clean vector (SVG/PDF/EPS) without touching a single setting, with the option to fine-tune detail and colors when the auto result isn't good enough.

## Screens

| ID | screen | route / how to reach | purpose | key components | states seen |
| --- | --- | --- | --- | --- | --- |
| S01 | Home / upload | `/` | drop or pick an image | drop zone, file button, example images, format list | empty, drag-over, uploading, invalid file, too large (inferred) |
| S02 | Processing | after upload | show progress while the image is analyzed and traced | progress indicator, thumbnail | loading, failed (inferred) |
| S03 | Result / review | `/images/{id}/edit` | compare bitmap vs vector and decide | viewer, flip / side-by-side toggle, zoom, pan, detected settings summary, download button, "re-vectorize with other settings" link | filled, zoomed, error (inferred) |
| S04 | Hand-pick settings | from S03 (Advanced, "Hand-pick Settings") | override auto settings, step by step | image-type cards, detail low/medium/high, color mode (unlimited / custom), palette picker 2 to 12 colors, preview thumbnails per option | default, option selected, re-processing |
| S05 | Download | from S03 | pick a format and save | format choice SVG / EPS / PDF, download | ready, paywall in the original (not needed here) |
| S06 | Sign in / sign up | header | account for history | email, password, OAuth (inferred) | empty, error |
| S07 | My images (history) | account menu | reopen past images | grid of thumbnails, date, delete | empty, filled |

## Flows

```
F01 Vectorize an image with auto settings and download it
    S01 -> S02 -> S03 -> S05
    happy path clicks: about 3 (pick file, Download, choose format). The number to beat.
    edge: unsupported format (WEBP, HEIC, TIFF), huge image (many megapixels), tiny image (16px icon),
          transparent PNG, animated GIF (first frame?), noisy JPG, network drop during upload

F02 Fix a poor result by hand-picking settings
    S03 -> S04 (type -> detail -> colors) -> S02 -> S03 -> S05
    happy path clicks: about 6
    edge: custom palette with 2 colors on a photo, unlimited colors on a logo (too many shapes),
          back out of the wizard and keep the old result

F03 Reopen a past image
    S06 -> S07 -> S03
    edge: deleted image, expired link
```

## Components

| component | variants | states | used on |
| --- | --- | --- | --- |
| Button | primary, secondary, ghost | default, hover, focus, disabled, loading | all |
| Drop zone | page-wide, inline | idle, drag-over, uploading, error | S01 |
| Progress indicator | bar or steps | indeterminate, determinate, failed | S02 |
| Image viewer | side-by-side, flip (toggle) | loading, zoomed, panning | S03 |
| Zoom control | buttons + wheel | min, max | S03 |
| Option card (with preview thumbnail) | image type, detail level | default, selected, focus | S04 |
| Segmented control | detail low / medium / high | selected | S04 |
| Color swatch / palette picker | proposed palette, editable swatch, eyedropper (inferred) | selected, editing | S04 |
| Format picker | SVG, EPS, PDF | selected | S05 |
| Toast / inline error | info, error | visible, dismissed | S01, S02, S05 |
| Header / nav | signed out, signed in | | all |
| Thumbnail grid | | empty, filled | S07 |

## Inferred data model

```
User      id, email, password_hash / oauth_id, created_at
          evidence: account + subscription on pricing page (source 8)
          confidence: medium (optional for internal use)

Image     id, user_id (nullable for anonymous), original_filename, mime, width, height,
          storage_key, created_at
          evidence: per-image URL /images/{id}/edit (source 9), input formats (source 7)
          confidence: high

Vectorization  id, image_id, image_type (photo | artwork_blended | artwork_sharp | scan),
               detail (low | medium | high), color_mode (unlimited | custom),
               palette (json array of hex, 2..12 when custom), auto (bool),
               status (queued | processing | done | failed), svg_storage_key,
               shape_count, created_at
          evidence: settings in tutorials (sources 2, 4, 5, 6); re-vectorize after settings change (source 13)
          confidence: high for settings, guess for status / shape_count

Export    id, vectorization_id, format (svg | eps | pdf), storage_key, created_at
          evidence: output formats (source 7)
          confidence: medium (could be generated on the fly, never stored)
```

Relationships: User 1-n Image, Image 1-n Vectorization (one per settings combination), Vectorization 1-n Export.

## Feature matrix

See `features.csv`. Must: 11, should: 5, could: 6 (3 of them marked skip).

## Out of scope (cannot or should not be cloned)

- **Their tracing engine.** Proprietary. The clone uses its own pipeline on an open library (e.g. VTracer, MIT, or a potrace-style approach; check licenses in /replica-architect). Output quality is their main edge and it will take tuning to approach.
- **Segmentation editor** (pixel brush, merge, Zap): left out by user decision. A should-have later if auto results fall short.
- **Billing and the download paywall:** internal use, nobody pays.
- **Desktop edition** (TIFF in, AI/DXF out, offline): web only. DXF is listed as a cheap could-have.
- Their name, logo, mascot, example images, copy and tutorial videos.

## Size

Screens 7, flows 3, entities 4. Hard parts:
1. **Tracing quality**: segmentation, color quantization, smooth Bezier fitting, and clean edges on anti-aliased logos. This is the whole product.
2. **Auto-detection** of image type and settings (heuristics: color count, edge sharpness, noise).
3. **Heavy processing**: large images take CPU time, so you need a job queue or a worker, timeouts, and per-image size limits. EPS/PDF export from SVG.

Size: **M** (a few weeks) for the core loop with an open-source engine. Matching Vector Magic's quality on hard images is open-ended, and /replica-diff will measure it.
