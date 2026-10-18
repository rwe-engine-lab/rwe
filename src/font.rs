//! Optional, embedded and reviewed TrueType fallback; no system font APIs.
use crate::{Canvas, Color, Rect};
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex, OnceLock},
};

struct Raster {
    metrics: fontdue::Metrics,
    alpha: Vec<u8>,
}
#[derive(Default)]
struct GlyphCache {
    entries: HashMap<(char, u32), Arc<Raster>>,
    order: VecDeque<(char, u32)>,
}
fn face() -> &'static fontdue::Font {
    static FONT: OnceLock<fontdue::Font> = OnceLock::new();
    FONT.get_or_init(|| {
        fontdue::Font::from_bytes(
            include_bytes!("assets/fonts/RWECJK-Regular.ttf") as &[u8],
            fontdue::FontSettings::default(),
        )
        .expect("reviewed embedded font must parse")
    })
}
fn supported(ch: char) -> bool {
    !ch.is_ascii() && ch != '\u{a0}' && face().has_glyph(ch)
}
pub(crate) fn advance(ch: char, size: u32) -> Option<u32> {
    supported(ch).then(|| {
        face()
            .metrics(ch, size as f32)
            .advance_width
            .round()
            .max(1.0) as u32
    })
}
fn raster(ch: char, size: u32) -> Arc<Raster> {
    static CACHE: OnceLock<Mutex<GlyphCache>> = OnceLock::new();
    let mutex = CACHE.get_or_init(|| Mutex::new(GlyphCache::default()));
    let mut cache = mutex.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(glyph) = cache.entries.get(&(ch, size)) {
        return Arc::clone(glyph);
    }
    let (metrics, alpha) = face().rasterize(ch, size as f32);
    let glyph = Arc::new(Raster { metrics, alpha });
    // Bounded FIFO: at most 512 rasters, and only the embedded face is accepted.
    if cache.entries.len() == 512 {
        if let Some(old) = cache.order.pop_front() {
            cache.entries.remove(&old);
        }
    }
    cache.order.push_back((ch, size));
    cache.entries.insert((ch, size), Arc::clone(&glyph));
    glyph
}
pub(crate) fn paint(
    ch: char,
    size: u32,
    x: i64,
    y: i64,
    color: Color,
    canvas: &mut Canvas,
    clip: Rect,
) -> bool {
    if !supported(ch) {
        return false;
    }
    let glyph = raster(ch, size);
    let metrics = glyph.metrics;
    // Keep the established cell height; use a common 7/8-em baseline and clip
    // outline overhang vertically, so fallback glyphs do not cover other lines.
    let top = y + i64::from(size * 7 / 8) - i64::from(metrics.ymin) - metrics.height as i64;
    for row in 0..metrics.height {
        let py = top + row as i64;
        if py < y || py >= y + i64::from(size) {
            continue;
        }
        for col in 0..metrics.width {
            let alpha = glyph.alpha[row * metrics.width + col];
            if alpha == 0 {
                continue;
            }
            let px = x + i64::from(metrics.xmin) + col as i64;
            if px < x || px >= x + i64::from(metrics.advance_width.round().max(1.0) as u32) {
                continue;
            }
            let (Ok(px), Ok(py)) = (i32::try_from(px), i32::try_from(py)) else {
                continue;
            };
            let alpha = ((u16::from(alpha) * u16::from(color.a) + 127) / 255) as u8;
            canvas.fill_rect_clipped(
                Rect::new(px, py, 1, 1),
                Color::rgba(color.r, color.g, color.b, alpha),
                clip,
            );
        }
    }
    true
}
