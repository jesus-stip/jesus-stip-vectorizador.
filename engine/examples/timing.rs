// usage: cargo run --release --example timing -- <file.rgba> <w> <h> <colors>
use std::time::Instant;
use vectorizer_engine::{prep_public as prep, trace_rgba};
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let px = std::fs::read(&a[1]).unwrap();
    let (w, h, k): (usize, usize, usize) = (a[2].parse().unwrap(), a[3].parse().unwrap(), a[4].parse().unwrap());
    let s = prep::Settings { colors: k, detail: 1, max_pixels: a[5].parse().unwrap() };
    let t = Instant::now();
    let work = prep::prepare(prep::Image { w, h, px: px.clone() }, &s);
    println!("prepare: {:?} -> {}x{}", t.elapsed(), work.img.w, work.img.h);
    let t = Instant::now();
    let svg = trace_rgba(prep::Image { w, h, px }, &s).unwrap();
    println!("total: {:?}, paths {}", t.elapsed(), svg.matches("<path").count());
}
