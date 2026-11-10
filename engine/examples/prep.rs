// usage: cargo run --release --example prep -- <file.rgba> <w> <h> <colors> <cap> <out.rgba> [nofringe] [nomedian]
// dumps the preprocessed (quantized) image, to see what VTracer receives
use vectorizer_engine::prep::{prepare, Image, Settings};
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let px = std::fs::read(&a[1]).unwrap();
    let mut s = Settings::new(a[4].parse().unwrap(), 1, a[5].parse().unwrap());
    s.fringe = !a.iter().any(|x| x == "nofringe");
    s.median = !a.iter().any(|x| x == "nomedian");
    let w = prepare(Image { w: a[2].parse().unwrap(), h: a[3].parse().unwrap(), px }, &s);
    std::fs::write(&a[6], &w.img.px).unwrap();
    println!("{} {}", w.img.w, w.img.h);
}
