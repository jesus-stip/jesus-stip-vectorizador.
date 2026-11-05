use wasm_bindgen::prelude::*;
use vtracer::{ColorImage, Config, convert};

/// rgba: width*height*4 bytes. Returns the SVG document as a string.
#[wasm_bindgen]
pub fn trace_rgba(rgba: Vec<u8>, width: usize, height: usize, filter_speckle: usize, corner_threshold: i32) -> Result<String, JsValue> {
    let img = ColorImage { pixels: rgba, width, height };
    let mut cfg = Config::default();
    cfg.filter_speckle = filter_speckle;
    cfg.color_precision = 8;
    cfg.layer_difference = 1;
    cfg.corner_threshold = corner_threshold;
    let svg = convert(img, cfg).map_err(|e| JsValue::from_str(&e))?;
    Ok(svg.to_string())
}
