//! Vectorizer engine: preprocessing (resample, denoise, palette) + VTracer, behind a tiny C ABI
//! so the browser can call it without wasm-bindgen.
//!
//! JS side: alloc(len) -> ptr, write RGBA, call trace(...), read result_ptr()/result_len() as UTF-8,
//! then dealloc(ptr, len). A return code != 0 means the result buffer holds an error message.

pub mod prep;
pub use prep as prep_public;

use prep::{Image, Settings};
use std::cell::RefCell;
use visioncortex::PointF64;
use vtracer::{ColorImage, Config};

thread_local! { static RESULT: RefCell<Vec<u8>> = RefCell::new(Vec::new()); }

fn set_result(s: String) { RESULT.with(|r| *r.borrow_mut() = s.into_bytes()); }

#[no_mangle]
pub extern "C" fn alloc(len: usize) -> *mut u8 {
    let mut v = Vec::<u8>::with_capacity(len);
    let p = v.as_mut_ptr();
    std::mem::forget(v);
    p
}

/// # Safety
/// `ptr` must come from `alloc(len)` with the same `len`.
#[no_mangle]
pub unsafe extern "C" fn dealloc(ptr: *mut u8, len: usize) { drop(Vec::from_raw_parts(ptr, 0, len)); }

#[no_mangle]
pub extern "C" fn result_ptr() -> *const u8 { RESULT.with(|r| r.borrow().as_ptr()) }
#[no_mangle]
pub extern "C" fn result_len() -> usize { RESULT.with(|r| r.borrow().len()) }

/// Suggest a color count for an RGBA image (8..=32). Result is the return value.
///
/// # Safety
/// `ptr` must point to `w*h*4` readable bytes.
#[no_mangle]
pub unsafe extern "C" fn suggest_colors(ptr: *const u8, w: usize, h: usize) -> u32 {
    let px = std::slice::from_raw_parts(ptr, w * h * 4);
    prep::suggest_colors(px) as u32
}

/// Vectorize. `colors` 2..=64, `detail` 0 low / 1 medium / 2 high, `max_pixels` caps the working size.
/// Returns 0 and the SVG in the result buffer, or 1 and an error message.
///
/// # Safety
/// `ptr` must point to `w*h*4` readable bytes.
#[no_mangle]
pub unsafe extern "C" fn trace(ptr: *const u8, w: usize, h: usize, colors: u32, detail: u32, max_pixels: u32) -> u32 {
    let px = std::slice::from_raw_parts(ptr, w * h * 4).to_vec();
    match trace_rgba(Image { w, h, px }, &Settings::new(colors as usize, detail, max_pixels as usize)) {
        Ok(svg) => { set_result(svg); 0 }
        Err(e) => { set_result(e); 1 }
    }
}

/// Safe entry point, also used by native tests.
pub fn trace_rgba(img: Image, s: &Settings) -> Result<String, String> {
    if img.w == 0 || img.h == 0 || img.px.len() != img.w * img.h * 4 { return Err("imagen vacía o corrupta".into()); }
    let (orig_w, orig_h) = (img.w, img.h);
    let work = prep::prepare(img, s);
    // VTracer drops patches smaller than filter_speckle² working pixels, so filter_speckle is a
    // side length: scale it linearly. Base sides in original pixels: low 3, medium 2, high 1.
    // (Scaling it by f² erased whole letters such as an "I" at 2x.)
    let base = match s.detail { 0 => 3.0, 2 => 1.0, _ => 2.0 };
    let f = work.scale_num as f64 / work.scale_den as f64;
    let speckle = (base * f).round() as usize;

    let cfg = Config {
        filter_speckle: speckle.max(1),
        color_precision: 8,
        layer_difference: 1,
        corner_threshold: 60,
        length_threshold: 4.0,
        splice_threshold: 45,
        path_precision: Some(1),
        ..Config::default()
    };
    let (ww, wh) = (work.img.w, work.img.h);
    let svg = vtracer::convert(ColorImage { pixels: work.img.px, width: ww, height: wh }, cfg)?;
    let mut out = String::with_capacity(svg.paths.len() * 200 + 256);
    out.push_str(&format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" version=\"1.1\" width=\"{orig_w}\" height=\"{orig_h}\" viewBox=\"0 0 {ww} {wh}\">\n"
    ));
    for p in &svg.paths {
        let (d, off) = p.path.to_svg_string(true, PointF64::default(), Some(1));
        out.push_str(&format!("<path d=\"{}\" fill=\"{}\" transform=\"translate({},{})\"/>\n", d, p.color.to_hex_string(), off.x, off.y));
    }
    out.push_str("</svg>\n");
    Ok(out)
}
