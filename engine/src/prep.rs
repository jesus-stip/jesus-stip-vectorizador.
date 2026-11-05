//! Preprocessing that made VTracer clean on real designs (see replica/engine-spike.md):
//! resample (2x for small images, down for huge ones), 3x3 median, k-means palette, no dithering,
//! alpha thresholded so transparency stays transparency.

pub struct Image { pub w: usize, pub h: usize, pub px: Vec<u8> }
pub struct Settings { pub colors: usize, pub detail: u32, pub max_pixels: usize }
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
    let img = median3(&img);
    let img = quantize(img, s.colors.clamp(2, 64));
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

fn quantize(mut img: Image, k: usize) -> Image {
    let cs = palette(&opaque_samples(&img.px, 40_000), k);
    let cs8: Vec<[u8; 3]> = cs.iter().map(|c| [c[0].round() as u8, c[1].round() as u8, c[2].round() as u8]).collect();
    for p in img.px.chunks_exact_mut(4) {
        if p[3] < 128 { p.copy_from_slice(&[0, 0, 0, 0]); continue; }
        let i = nearest(&[p[0] as f32, p[1] as f32, p[2] as f32], &cs);
        p[..3].copy_from_slice(&cs8[i]); p[3] = 255;
    }
    img
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

