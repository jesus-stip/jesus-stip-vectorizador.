// usage: cargo run --release --example trace -- <file.rgba> <w> <h> <colors> <cap> <out.svg> [nomedian] [nofringe]
use std::time::Instant;
use vectorizer_engine::{prep::{Image, Settings}, trace_rgba};
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let px = std::fs::read(&a[1]).unwrap();
    let mut s = Settings::new(a[4].parse().unwrap(), 1, a[5].parse().unwrap());
    s.median = !a.iter().any(|x| x == "nomedian");
    s.fringe = !a.iter().any(|x| x == "nofringe");
    let t = Instant::now();
    let svg = trace_rgba(Image { w: a[2].parse().unwrap(), h: a[3].parse().unwrap(), px }, &s).unwrap();
    std::fs::write(&a[6], &svg).unwrap();
    println!("{:?} paths {}", t.elapsed(), svg.matches("<path").count());
}
