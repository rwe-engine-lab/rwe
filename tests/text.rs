//! Text metrics, inherited styles, and bitmap paint regressions.
use rwe::{BlockLayout, BoxStyle, Canvas, Color, Document, Rect, Stylesheet};

#[test]
fn wraps_words_collapses_ascii_whitespace_and_sizes_auto_blocks() {
    let doc = Document::parse(
        "<p style='font-size: 8px; padding: 1px'>  AB\n  CD E  </p><div style='height: 2px'></div>",
    )
    .unwrap();
    let layout = BlockLayout::new(&doc, 20).unwrap();
    let runs = layout.text_runs();
    assert_eq!(
        runs.iter().map(|run| run.text.as_str()).collect::<Vec<_>>(),
        ["AB", "CD", "E"]
    );
    assert_eq!(runs[0].rect, Rect::new(1, 1, 12, 8));
    assert_eq!(runs[2].rect, Rect::new(1, 17, 6, 8));
    assert_eq!(layout.boxes()[1].rect.height, 26);
    assert_eq!(layout.boxes()[2].rect.y, 26);
    assert_eq!(layout.height(), 28);
}

#[test]
fn inherits_text_styles_and_applies_stylesheet_and_inline_overrides() {
    let doc =
        Document::parse("<main><p>A</p><p class='small' style='color: #0f0'>B</p></main>").unwrap();
    let sheet = Stylesheet::parse(
        "main { color: #f00; font-size: 24px } .small { color: #00f; font-size: 8px }",
    )
    .unwrap();
    let layout = BlockLayout::with_stylesheet(&doc, &sheet, 100).unwrap();
    let runs = layout.text_runs();
    assert_eq!(
        (runs[0].color, runs[0].font_size),
        (Color::rgba(255, 0, 0, 255), 24)
    );
    assert_eq!(
        (runs[1].color, runs[1].font_size),
        (Color::rgba(0, 255, 0, 255), 8)
    );
    assert_eq!(layout.height(), 32);
    assert_eq!(sheet.computed_style(&doc, 2).unwrap().unwrap().color, None);
}

#[test]
fn paints_glyph_shape_without_painting_cell_gaps_and_blends_foreground() {
    let doc = Document::parse("<p style='font-size: 8px; color: #ff000080'>A</p>").unwrap();
    let layout = BlockLayout::new(&doc, 10).unwrap();
    let mut canvas = Canvas::new(10, 10).unwrap();
    canvas.clear(Color::WHITE);
    layout.paint(&mut canvas);
    let ink = Color::rgba(255, 0, 0, 128).over(Color::WHITE);
    assert_eq!(canvas.pixel(0, 0), Some(Color::WHITE));
    assert_eq!(canvas.pixel(1, 0), Some(ink));
    assert_eq!(canvas.pixel(0, 3), Some(ink));
    assert_eq!(canvas.pixel(5, 3), Some(Color::WHITE));
    assert_eq!(canvas.pixel(1, 7), Some(Color::WHITE));
}

#[test]
fn fallback_glyphs_and_zero_width_are_finite_and_clip_to_canvas() {
    let doc = Document::parse("<p style='font-size: 8px'>🦀🦀</p>").unwrap();
    let layout = BlockLayout::new(&doc, 0).unwrap();
    assert_eq!(layout.text_runs().len(), 2);
    assert_eq!(layout.height(), 16);
    let mut canvas = Canvas::new(2, 2).unwrap();
    layout.paint(&mut canvas);
    assert_eq!(canvas.pixel(0, 0), Some(Color::BLACK));
    assert_eq!(canvas.pixel(1, 1), Some(Color::TRANSPARENT));
}

#[test]
fn text_and_sibling_backgrounds_paint_in_source_order_during_overflow() {
    let doc = Document::parse("<p style='height: 0; font-size: 8px'>A</p><div style='height: 2px; background-color: #0f0'></div>").unwrap();
    let layout = BlockLayout::new(&doc, 10).unwrap();
    let mut canvas = Canvas::new(10, 8).unwrap();
    layout.paint(&mut canvas);
    assert_eq!(canvas.pixel(1, 0), Some(Color::rgba(0, 255, 0, 255)));
    assert_eq!(canvas.pixel(0, 3), Some(Color::BLACK));
    assert_eq!(layout.height(), 2);
}

#[test]
fn skips_hidden_text_and_whitespace_only_nodes() {
    let doc = Document::parse(" \n <p style='display: none'>hidden</p>\n <p> \t </p>").unwrap();
    let layout = BlockLayout::new(&doc, 100).unwrap();
    assert!(layout.text_runs().is_empty());
    assert_eq!(layout.height(), 0);
}

#[test]
fn rejects_invalid_font_sizes_and_checked_text_geometry() {
    for source in [
        "font-size: 0",
        "font-size: 7px",
        "font-size: 12px",
        "font-size: 72px",
        "font-size: auto",
        "color: red",
    ] {
        assert!(BoxStyle::parse(source).is_err(), "{source}");
    }
    let doc = Document::parse("<div style='height: 2147483647px'></div>A").unwrap();
    assert!(BlockLayout::new(&doc, 100).is_err());
    let doc = Document::parse("<div style='padding: 1073741823px'>A</div>").unwrap();
    assert!(BlockLayout::new(&doc, 100).is_err());
}
