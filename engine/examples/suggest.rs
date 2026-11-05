// usage: cargo run --release --example suggest -- <file.rgba>... : grid over thresholds, for tuning
fn main() {
    for f in std::env::args().skip(1) {
        let px = std::fs::read(&f).unwrap();
        let mut line = format!("{}:", f.rsplit('/').next().unwrap());
        for share in [0.004f32, 0.008, 0.015] { for merge in [34f32, 50.0, 70.0] {
            line += &format!(" s{share}/m{merge}={}", vectorizer_engine::prep::suggest_colors_with(&px, share, merge)); } }
        println!("{line}");
    }
}
