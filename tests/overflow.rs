//! Nested overflow clipping, local paint clips, and extreme rectangle arithmetic.
use rwe::{
    BlockLayout, BoxStyle, Canvas, Color, Document, Overflow, ParseLimits, Rect, Stylesheet,
};

fn paint(source: &str, width: u32, height: u32) -> (BlockLayout, Canvas) {
    let layout = BlockLayout::new(&Document::parse(source).unwrap(), width).unwrap();
    let mut canvas = Canvas::new(width, height).unwrap();
    canvas.clear(Color::WHITE);
    layout.paint(&mut canvas);
    (layout, canvas)
}

#[test]
fn hidden_clips_descendants_to_padding_box_and_preserves_own_border() {
    let source = "<div style='width: 3px; height: 2px; padding: 1px; border: 1px solid #000; overflow: hidden'><div style='width: 10px; height: 10px; background-color: #f00'></div></div>";
    let (layout, canvas) = paint(source, 10, 10);
    assert_eq!(layout.boxes()[1].clip_rect, Some(Rect::new(1, 1, 5, 4)));
    assert_eq!(canvas.pixel(5, 4), Some(Color::rgba(255, 0, 0, 255)));
    assert_eq!(canvas.pixel(6, 4), Some(Color::BLACK));
    assert_eq!(canvas.pixel(5, 5), Some(Color::BLACK));
    assert_eq!(canvas.pixel(7, 4), Some(Color::WHITE));
    assert_eq!(canvas.pixel(5, 6), Some(Color::WHITE));
    let (visible, canvas) = paint(&source.replace("hidden", "visible"), 10, 10);
    assert_eq!(visible.boxes()[1].rect, layout.boxes()[1].rect);
    assert_eq!(visible.height(), layout.height());
    assert_eq!(canvas.pixel(6, 4), Some(Color::rgba(255, 0, 0, 255)));
    assert_eq!(visible.boxes()[1].clip_rect, None);
}

#[test]
fn intersects_nested_clips_and_restores_them_before_later_siblings() {
    let source = "<main style='width: 4px; height: 4px; overflow: hidden'><div style='margin-left: 1px; width: 2px; height: 2px; border: 1px solid #000; overflow: hidden'><div style='width: 8px; height: 8px; background-color: #f00'></div></div></main><div style='width: 1px; height: 1px; background-color: #0f0'></div>";
    let (_, canvas) = paint(source, 8, 8);
    assert_eq!(canvas.pixel(2, 1), Some(Color::rgba(255, 0, 0, 255)));
    assert_eq!(canvas.pixel(3, 2), Some(Color::rgba(255, 0, 0, 255)));
    assert_eq!(canvas.pixel(1, 1), Some(Color::BLACK));
    assert_eq!(canvas.pixel(4, 1), Some(Color::WHITE));
    assert_eq!(canvas.pixel(0, 4), Some(Color::rgba(0, 255, 0, 255)));
    assert_eq!(canvas.pixel(2, 4), Some(Color::WHITE));
}

#[test]
fn clips_partial_scaled_glyph_pixels_without_changing_text_metrics() {
    let (hidden, canvas) = paint(
        "<p style='width: 3px; height: 1px; font-size: 16px; white-space: pre; overflow: hidden'>AA</p>",
        20,
        20,
    );
    assert_eq!(hidden.text_runs()[0].rect, Rect::new(0, 0, 24, 16));
    assert_eq!(canvas.pixel(2, 0), Some(Color::BLACK));
    assert_eq!(canvas.pixel(3, 0), Some(Color::WHITE));
    assert_eq!(canvas.pixel(2, 1), Some(Color::WHITE));
    assert_eq!(hidden.height(), 1);
}

#[test]
fn empty_and_disjoint_clip_regions_do_not_leak_or_affect_other_canvases() {
    let (layout, canvas) = paint(
        "<div style='width: 0; height: 0; overflow: hidden'><div style='width: 8px; height: 8px; background-color: #f00'></div></div><div style='margin-left: 20px; width: 2px; height: 2px; overflow: hidden'><p style='background-color: #000; height: 9px'></p></div>",
        10,
        10,
    );
    assert!(canvas.pixels().iter().all(|pixel| *pixel == Color::WHITE));
    let mut second = Canvas::new(30, 10).unwrap();
    second.clear(Color::WHITE);
    layout.paint(&mut second);
    assert_eq!(second.pixel(20, 0), Some(Color::BLACK));
    assert_eq!(second.pixel(20, 2), Some(Color::WHITE));
    second.fill_rect(Rect::new(0, 0, 1, 1), Color::BLACK);
    assert_eq!(second.pixel(0, 0), Some(Color::BLACK));
}

#[test]
fn overflow_cascades_without_inheritance_and_inline_can_restore_visible() {
    let doc = Document::parse(
        "<main><div id='target' class='clip' style='overflow: visible'><p></p></div></main>",
    )
    .unwrap();
    let css = Stylesheet::parse(
        "main { overflow: hidden } .clip { overflow: hidden } #target { overflow: hidden }",
    )
    .unwrap();
    assert_eq!(
        css.computed_style(&doc, 1).unwrap().unwrap().overflow,
        Overflow::Hidden
    );
    assert_eq!(
        css.computed_style(&doc, 2).unwrap().unwrap().overflow,
        Overflow::Visible
    );
    assert_eq!(
        css.computed_style(&doc, 3).unwrap().unwrap().overflow,
        Overflow::Visible
    );
    assert_eq!(
        BoxStyle::parse("OVERFLOW: HIDDEN").unwrap().overflow,
        Overflow::Hidden
    );
    for value in ["scroll", "auto", "clip", "hidden visible", "inherit"] {
        assert!(BoxStyle::parse(&format!("overflow: {value}")).is_err());
    }
}

#[test]
fn rectangle_intersections_handle_empty_touching_negative_and_extreme_edges() {
    let a = Rect::new(-3, -2, 7, 5);
    let b = Rect::new(1, 1, 9, 9);
    assert_eq!(a.intersection(b), Some(Rect::new(1, 1, 3, 2)));
    assert_eq!(a.intersection(b), b.intersection(a));
    assert_eq!(
        Rect::new(0, 0, 1, 1).intersection(Rect::new(1, 0, 1, 1)),
        None
    );
    assert_eq!(a.intersection(Rect::new(0, 0, 0, 1)), None);
    assert_eq!(
        Rect::new(i32::MIN, i32::MIN, u32::MAX, u32::MAX).intersection(Rect::new(
            i32::MAX - 1,
            i32::MAX - 1,
            u32::MAX,
            u32::MAX
        )),
        Some(Rect::new(i32::MAX - 1, i32::MAX - 1, 1, 1))
    );
}

#[test]
fn local_clip_is_intersected_with_canvas_and_applies_alpha_only_once() {
    let mut canvas = Canvas::new(4, 4).unwrap();
    let color = Color::rgba(255, 0, 0, 128);
    canvas.fill_rect_clipped(
        Rect::new(i32::MIN, i32::MIN, u32::MAX, u32::MAX),
        color,
        Rect::new(1, 1, 2, 2),
    );
    assert_eq!(canvas.pixel(1, 1), Some(color));
    assert_eq!(canvas.pixel(0, 0), Some(Color::TRANSPARENT));
    assert_eq!(canvas.pixel(3, 3), Some(Color::TRANSPARENT));
    canvas.fill_rect(Rect::new(0, 0, 1, 1), Color::BLACK);
    assert_eq!(canvas.pixel(0, 0), Some(Color::BLACK));
}

#[test]
fn deeply_nested_hidden_boxes_use_an_iterative_clip_stack() {
    let source = format!(
        "{}<div style='height: 1px; background-color: #f00'></div>{}",
        "<div style='overflow: hidden'>".repeat(1500),
        "</div>".repeat(1500)
    );
    let doc = Document::parse_with_limits(
        &source,
        ParseLimits {
            max_depth: 1600,
            ..ParseLimits::default()
        },
    )
    .unwrap();
    let layout = BlockLayout::new(&doc, 2).unwrap();
    let mut canvas = Canvas::new(2, 2).unwrap();
    layout.paint(&mut canvas);
    assert_eq!(canvas.pixel(0, 0), Some(Color::rgba(255, 0, 0, 255)));
    assert_eq!(canvas.pixel(0, 1), Some(Color::TRANSPARENT));
}
