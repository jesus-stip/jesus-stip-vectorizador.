//! Preprocessing that made VTracer clean on real designs (see replica/engine-spike.md):
//! resample (2x for small images, down for huge ones), 3x3 median, k-means palette, no dithering,
//! alpha thresholded so transparency stays transparency.

pub struct Image { pub w: usize, pub h: usize, pub px: Vec<u8> }
pub struct Settings { pub colors: usize, pub detail: u32, pub max_pixels: usize, pub median: bool, pub fringe: bool }

impl Settings {
    /// The defaults the browser uses.
    pub fn new(colors: usize, detail: u32, max_pixels: usize) -> Self { Settings { colors, detail, max_pixels, median: true, fringe: true } }
}
pub struct Work { pub img: Image, pub scale_num: usize, pub scale_den: usize }

pub fn prepare(img: Image, s: &Settings) -> Work {
    let max = s.max_pixels.max(250_000);
    let area = img.w * img.h;
    // Upscale up to 2x (thin strokes and small text survive), never past the working cap;
    // downscale when the original alone is over it.
    let f = (max as f64 / area as f64).sqrt().min(2.0);
    let (img, num, den) = if (f - 1.0).abs() < 0.05 {
        (img, 1, 1)
    } else {
        let (nw, nh) = (((img.w as f64) * f).round() as usize, ((img.h as f64) * f).round() as usize);
        let den = 100usize; let num = ((f * 100.0).round() as usize).max(1);
        (resize_bilinear(&img, nw.max(1), nh.max(1)), num, den)
    };
    let img = if s.median { median3(&img) } else { img };
    let img = quantize(img, s.colors.clamp(2, 64), s.fringe);
    Work { img, scale_num: num, scale_den: den }
}

fn resize_bilinear(src: &Image, nw: usize, nh: usize) -> Image {
    let mut px = vec![0u8; nw * nh * 4];
    let sx = src.w as f32 / nw as f32;
    let sy = src.h as f32 / nh as f32;
    for y in 0..nh {
        let fy = ((y as f32 + 0.5) * sy - 0.5).max(0.0);
        let y0 = (fy as usize).min(src.h - 1);
        let y1 = (y0 + 1).min(src.h - 1);
        let ty = fy - y0 as f32;
        for x in 0..nw {
            let fx = ((x as f32 + 0.5) * sx - 0.5).max(0.0);
            let x0 = (fx as usize).min(src.w - 1);
            let x1 = (x0 + 1).min(src.w - 1);
            let tx = fx - x0 as f32;
            for c in 0..4 {
                let g = |xx: usize, yy: usize| src.px[(yy * src.w + xx) * 4 + c] as f32;
                let v = g(x0, y0) * (1.0 - tx) * (1.0 - ty) + g(x1, y0) * tx * (1.0 - ty)
                    + g(x0, y1) * (1.0 - tx) * ty + g(x1, y1) * tx * ty;
                px[(y * nw + x) * 4 + c] = v.round() as u8;
            }
        }
    }
    Image { w: nw, h: nh, px }
}

fn median3(src: &Image) -> Image {
    let (w, h) = (src.w, src.h);
    let mut px = src.px.clone();
    let mut win = [0u8; 9];
    for y in 1..h.saturating_sub(1) {
        for x in 1..w.saturating_sub(1) {
            for c in 0..3 {
                let mut k = 0;
                for dy in 0..3 { for dx in 0..3 { win[k] = src.px[((y + dy - 1) * w + (x + dx - 1)) * 4 + c]; k += 1; } }
                win.sort_unstable();
                px[(y * w + x) * 4 + c] = win[4];
            }
        }
    }
    Image { w, h, px }
}

/// Small deterministic PRNG (no `rand`, no time: works the same in WASM and native).
struct Rng(u64);
impl Rng { fn next(&mut self) -> u64 { self.0 ^= self.0 << 13; self.0 ^= self.0 >> 7; self.0 ^= self.0 << 17; self.0 } }

fn opaque_samples(px: &[u8], max: usize) -> Vec<[f32; 3]> {
    let n = px.len() / 4;
    let step = (n / max).max(1);
    (0..n).step_by(step).filter(|i| px[i * 4 + 3] >= 128)
        .map(|i| [px[i * 4] as f32, px[i * 4 + 1] as f32, px[i * 4 + 2] as f32]).collect()
}

fn d2(a: &[f32; 3], b: &[f32; 3]) -> f32 { (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2) }

fn nearest(p: &[f32; 3], cs: &[[f32; 3]]) -> usize {
    let mut best = 0; let mut bd = f32::MAX;
    for (i, c) in cs.iter().enumerate() { let d = d2(p, c); if d < bd { bd = d; best = i; } }
    best
}

/// k-means++ on a sample, then every pixel snaps to its nearest center.
fn palette(samples: &[[f32; 3]], k: usize) -> Vec<[f32; 3]> {
    if samples.is_empty() { return vec![[255.0; 3]]; }
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let mut cs = vec![samples[(rng.next() as usize) % samples.len()]];
    let mut dist: Vec<f32> = samples.iter().map(|s| d2(s, &cs[0])).collect();
    while cs.len() < k {
        let total: f32 = dist.iter().sum();
        if total <= 0.0 { break; }
        let mut t = (rng.next() % 1_000_000) as f32 / 1_000_000.0 * total;
        let mut pick = samples.len() - 1;
        for (i, d) in dist.iter().enumerate() { t -= d; if t <= 0.0 { pick = i; break; } }
        cs.push(samples[pick]);
        for (i, s) in samples.iter().enumerate() { dist[i] = dist[i].min(d2(s, &cs[cs.len() - 1])); }
    }
    for _ in 0..8 {
        let mut sum = vec![[0f64; 3]; cs.len()]; let mut cnt = vec![0usize; cs.len()];
        for s in samples { let i = nearest(s, &cs); for c in 0..3 { sum[i][c] += s[c] as f64; } cnt[i] += 1; }
        for i in 0..cs.len() { if cnt[i] > 0 { for c in 0..3 { cs[i][c] = (sum[i][c] / cnt[i] as f64) as f32; } } }
    }
    cs
}

const TRANSPARENT: u8 = u8::MAX;

fn rgb(px: &[u8], i: usize) -> [f32; 3] { [px[i * 4] as f32, px[i * 4 + 1] as f32, px[i * 4 + 2] as f32] }

fn quantize(img: Image, k: usize, fringe: bool) -> Image {
    let cs = palette(&opaque_samples(&img.px, 40_000), k);
    let n = img.w * img.h;
    let mut lab = vec![TRANSPARENT; n];
    #[allow(clippy::needless_range_loop)]
    for i in 0..n { if img.px[i * 4 + 3] >= 128 { lab[i] = nearest(&rgb(&img.px, i), &cs) as u8; } }
    let mut cs = cs;
    if fringe {
        for _ in 0..3 { if !resolve_fringe(&img, &mut lab, &cs, FRINGE_RES) { break; } }
        lab = resolve_edges(&img, &lab, &cs);
        cs = core_colors(&img, &lab, &cs);
        snap_neutrals(&mut cs);
    }
    let mut px = vec![0u8; n * 4];
    for i in 0..n {
        if lab[i] == TRANSPARENT { continue; }
        let c = cs[lab[i] as usize];
        px[i * 4..i * 4 + 4].copy_from_slice(&[c[0].round() as u8, c[1].round() as u8, c[2].round() as u8, 255]);
    }
    Image { w: img.w, h: img.h, px }
}

/// Second anti-aliasing pass, per pixel. Some blend pixels land in a real cluster (the dark band
/// between a black outline and a light-blue sky joins the sky's own shadow color), so the
/// cluster-level pass cannot see them. For every pixel next to a different label, look at the
/// labels within 2 px; if the pixel's own color lies on the line between two of them (strong
/// contrast, off the line by < 25 RGB), it goes to whichever end it is closer to.
fn resolve_edges(img: &Image, lab: &[u8], cs: &[[f32; 3]]) -> Vec<u8> {
    const R: usize = 2;
    let (w, h) = (img.w, img.h);
    let mut out = lab.to_vec();
    let mut near: Vec<u8> = Vec::with_capacity(25);
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let l = lab[i];
            if l == TRANSPARENT { continue; }
            near.clear();
            for yy in y.saturating_sub(R)..(y + R + 1).min(h) {
                for xx in x.saturating_sub(R)..(x + R + 1).min(w) {
                    let m = lab[yy * w + xx];
                    if m != TRANSPARENT && !near.contains(&m) { near.push(m); }
                }
            }
            if near.len() < 2 { continue; }
            let p = rgb(&img.px, i);
            let mut best: Option<(u8, f32)> = None; // (target, contrast)
            for ai in 0..near.len() {
                for bi in (ai + 1)..near.len() {
                    let (a, b) = (cs[near[ai] as usize], cs[near[bi] as usize]);
                    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                    let len2 = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];
                    if len2 < 60.0 * 60.0 { continue; }
                    let ap = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];
                    let t = (ap[0] * ab[0] + ap[1] * ab[1] + ap[2] * ab[2]) / len2;
                    if !(-0.05..=1.05).contains(&t) { continue; }
                    let off = [ap[0] - t * ab[0], ap[1] - t * ab[1], ap[2] - t * ab[2]];
                    if (off[0] * off[0] + off[1] * off[1] + off[2] * off[2]).sqrt() >= 25.0 { continue; }
                    let target = if t < 0.5 { near[ai] } else { near[bi] };
                    if best.map_or(true, |(_, c)| len2 > c) { best = Some((target, len2)); }
                }
            }
            if let Some((t, _)) = best { out[i] = t; }
        }
    }
    out
}

/// Print-friendly neutrals: a near-black becomes a true neutral black and a near-white becomes
/// paper white, so outlines are one clean black and "white" is really no ink.
fn snap_neutrals(cs: &mut [[f32; 3]]) {
    for c in cs.iter_mut() {
        let (mx, mn) = (c[0].max(c[1]).max(c[2]), c[0].min(c[1]).min(c[2]));
        let lum = 0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2];
        if lum < 60.0 && mx - mn < 40.0 { *c = [lum * 0.4; 3]; }
        else if lum > 228.0 && mx - mn < 24.0 { *c = [255.0; 3]; }
    }
}

/// How far off the A-B line a cluster center may sit and still count as a blend of A and B.
const FRINGE_RES: f32 = 30.0;

/// Re-estimate each color from its interior pixels only (all 8 neighbours share the label).
/// Edge pixels are blends, so averaging them in turns black into dark brown and white into grey.
fn core_colors(img: &Image, lab: &[u8], cs: &[[f32; 3]]) -> Vec<[f32; 3]> {
    let (w, h) = (img.w, img.h);
    let mut sum = vec![[0f64; 3]; cs.len()];
    let mut cnt = vec![0usize; cs.len()];
    for y in 1..h.saturating_sub(1) {
        for x in 1..w.saturating_sub(1) {
            let l = lab[y * w + x];
            if l == TRANSPARENT { continue; }
            let interior = (0..3).all(|dy| (0..3).all(|dx| lab[(y + dy - 1) * w + (x + dx - 1)] == l));
            if !interior { continue; }
            let p = rgb(&img.px, y * w + x);
            for c in 0..3 { sum[l as usize][c] += p[c] as f64; }
            cnt[l as usize] += 1;
        }
    }
    cs.iter().enumerate()
        .map(|(i, c)| if cnt[i] >= 16 { [0, 1, 2].map(|k| (sum[i][k] / cnt[i] as f64) as f32) } else { *c })
        .collect()
}

/// Summed-area table of one label's mask, (w+1) x (h+1).
fn sat(lab: &[u8], w: usize, h: usize, l: u8) -> Vec<u32> {
    let mut t = vec![0u32; (w + 1) * (h + 1)];
    for y in 0..h {
        let mut row = 0u32;
        for x in 0..w {
            row += (lab[y * w + x] == l) as u32;
            t[(y + 1) * (w + 1) + x + 1] = t[y * (w + 1) + x + 1] + row;
        }
    }
    t
}

fn window(t: &[u32], w: usize, h: usize, x: usize, y: usize, r: usize) -> u32 {
    let (x0, y0, x1, y1) = (x.saturating_sub(r), y.saturating_sub(r), (x + r + 1).min(w), (y + r + 1).min(h));
    let s = w + 1;
    t[y1 * s + x1] + t[y0 * s + x0] - t[y0 * s + x1] - t[y1 * s + x0]
}

/// Anti-aliasing fix. An edge between colors A and B leaves pixels that are a blend of the two;
/// k-means gives that blend its own cluster, which VTracer then traces as a grey (or pink, or
/// muddy) band along every outline. A cluster C is treated as such a fringe when
///   1. its center lies on the line from A to B (between 12% and 88%, off the line by < FRINGE_RES), and
///   2. most of its pixels (55%+) sit within 3 px of both an A pixel and a B pixel.
/// Rule 2 keeps real mid-tones (skin between red and white, a light blue sky) that cover areas.
/// Each fringe pixel then goes to A or B by where its own color falls on the A-B line, which puts
/// the edge at the 50% crossing: the sharp edge the artwork had before it was rasterized.
fn resolve_fringe(img: &Image, lab: &mut [u8], cs: &[[f32; 3]], max_res: f32) -> bool {
    const RADIUS: usize = 3;
    let (w, h) = (img.w, img.h);
    let mut cnt = vec![0usize; cs.len()];
    for &l in lab.iter() { if l != TRANSPARENT { cnt[l as usize] += 1; } }
    let mut sats: Vec<Option<Vec<u32>>> = vec![None; cs.len()];
    let mut changed = false;
    // smallest clusters first: fringes are thin, so they are small
    let mut order: Vec<usize> = (0..cs.len()).filter(|&i| cnt[i] > 0).collect();
    order.sort_by_key(|&i| cnt[i]);
    for c in order {
        if cnt[c] == 0 { continue; }
        let mut best: Option<(usize, usize, f32)> = None;
        for a in 0..cs.len() {
            if a == c || cnt[a] == 0 { continue; }
            for b in (a + 1)..cs.len() {
                if b == c || cnt[b] == 0 { continue; }
                let ab = [cs[b][0] - cs[a][0], cs[b][1] - cs[a][1], cs[b][2] - cs[a][2]];
                let len2 = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];
                if len2 < 60.0 * 60.0 { continue; } // too little contrast to leave a visible fringe
                let ac = [cs[c][0] - cs[a][0], cs[c][1] - cs[a][1], cs[c][2] - cs[a][2]];
                let t = (ac[0] * ab[0] + ac[1] * ab[1] + ac[2] * ab[2]) / len2;
                if !(0.12..=0.88).contains(&t) { continue; }
                let off = [ac[0] - t * ab[0], ac[1] - t * ab[1], ac[2] - t * ab[2]];
                let res = (off[0] * off[0] + off[1] * off[1] + off[2] * off[2]).sqrt();
                if res < max_res && best.map_or(true, |(_, _, r)| res < r) { best = Some((a, b, res)); }
            }
        }
        let Some((a, b, _)) = best else { continue };
        for l in [a, b] { if sats[l].is_none() { sats[l] = Some(sat(lab, w, h, l as u8)); } }
        let (sa, sb) = (sats[a].as_ref().unwrap(), sats[b].as_ref().unwrap());
        let (mut near, mut total) = (0usize, 0usize);
        for y in 0..h { for x in 0..w {
            if lab[y * w + x] as usize != c { continue; }
            total += 1;
            if window(sa, w, h, x, y, RADIUS) > 0 && window(sb, w, h, x, y, RADIUS) > 0 { near += 1; }
        } }
        if total == 0 || (near as f32) < 0.55 * total as f32 { continue; }
        let ab = [cs[b][0] - cs[a][0], cs[b][1] - cs[a][1], cs[b][2] - cs[a][2]];
        let len2 = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];
        for i in 0..w * h {
            if lab[i] as usize != c { continue; }
            let p = rgb(&img.px, i);
            let t = ((p[0] - cs[a][0]) * ab[0] + (p[1] - cs[a][1]) * ab[1] + (p[2] - cs[a][2]) * ab[2]) / len2;
            let to = if t < 0.5 { a } else { b };
            lab[i] = to as u8; cnt[to] += 1;
        }
        cnt[c] = 0;
        // a and b grew: their tables are stale
        sats[a] = None; sats[b] = None;
        changed = true;
    }
    changed
}

/// Suggest a palette size: cluster into 32, drop clusters under 0.8% of the pixels (edge fringe and
/// JPEG noise), merge clusters closer than `MERGE` in RGB, and count what is left (+2 headroom).
pub fn suggest_colors(px: &[u8]) -> usize { suggest_colors_with(px, 0.008, 34.0) }

pub fn suggest_colors_with(px: &[u8], min_share: f32, merge: f32) -> usize {
    let samples = opaque_samples(px, 30_000);
    if samples.is_empty() { return 8; }
    let cs = palette(&samples, 32);
    let mut cnt = vec![0usize; cs.len()];
    for s in &samples { cnt[nearest(s, &cs)] += 1; }
    let min = (samples.len() as f32 * min_share) as usize;
    let mut kept: Vec<[f32; 3]> = Vec::new();
    let mut order: Vec<usize> = (0..cs.len()).filter(|&i| cnt[i] > min).collect();
    order.sort_by(|a, b| cnt[*b].cmp(&cnt[*a]));
    for i in order { if kept.iter().all(|k| d2(k, &cs[i]).sqrt() >= merge) { kept.push(cs[i]); } }
    (kept.len() + 2).clamp(8, 32)
}

