//! End-to-end geometry and pixel coverage for the block rendering subset.
use rwe::{BlockLayout, BoxStyle, Canvas, Color, Document, ParseLimits, Rect};

#[test]
fn nested_flow_uses_content_box_padding_and_noncollapsing_margins() {
    let doc = Document::parse("<main style='padding: 10px'><div style='margin: 5px; height: 20px'></div><div style='height: 7px'></div></main>").unwrap();
    let layout = BlockLayout::new(&doc, 100).unwrap();
    assert_eq!(layout.boxes()[0].rect, Rect::new(0, 0, 100, 57));
    assert_eq!(layout.boxes()[1].rect, Rect::new(0, 0, 100, 57));
    assert_eq!(layout.boxes()[2].rect, Rect::new(15, 15, 70, 20));
    assert_eq!(layout.boxes()[3].rect, Rect::new(10, 40, 80, 7));
    assert_eq!((layout.width(), layout.height()), (100, 57));
}

#[test]
fn backgrounds_paint_parent_first_and_clip_visible_overflow() {
    let doc = Document::parse("<div style='width: 4px; height: 2px; background-color: #00f'><div style='width: 8px; height: 4px; background-color: #ff000080'></div></div><div style='height: 1px; background-color: #0f0'></div>").unwrap();
    let layout = BlockLayout::new(&doc, 6).unwrap();
    assert_eq!(layout.height(), 3);
    let mut canvas = Canvas::new(6, 5).unwrap();
    canvas.clear(Color::WHITE);
    layout.paint(&mut canvas);
    let red = Color::rgba(255, 0, 0, 128);
    assert_eq!(
        canvas.pixel(0, 0),
        Some(red.over(Color::rgba(0, 0, 255, 255)))
    );
    assert_eq!(canvas.pixel(5, 0), Some(red.over(Color::WHITE)));
    assert_eq!(canvas.pixel(0, 2), Some(Color::rgba(0, 255, 0, 255)));
    assert_eq!(canvas.pixel(5, 3), Some(red.over(Color::WHITE)));
    assert_eq!(canvas.pixel(5, 4), Some(Color::WHITE));
}

#[test]
fn hidden_subtrees_and_whitespace_do_not_affect_flow() {
    let doc = Document::parse("  <div style='display: none'><div style='unsupported: yes'></div></div><div style='height: 3px'>  </div>").unwrap();
    let layout = BlockLayout::new(&doc, 20).unwrap();
    assert_eq!(layout.boxes().len(), 2);
    assert_eq!(layout.height(), 3);
    assert_eq!(layout.boxes()[1].rect.y, 0);
}

#[test]
fn parses_supported_values_and_last_declaration_wins() {
    let style = BoxStyle::parse("WIDTH: 2px; width: auto; height: 0; margin: 3px; padding: 1px; background-color: #1234; display: NONE; display: block;").unwrap();
    assert_eq!(style.width, None);
    assert_eq!(style.height, Some(0));
    assert_eq!(
        (style.margin.top, style.padding.top, style.visible),
        (3, 1, true)
    );
    assert_eq!(style.background, Color::rgba(17, 34, 51, 68));
    assert_eq!(
        BoxStyle::parse("background-color: transparent").unwrap(),
        BoxStyle::default()
    );
    for source in [
        "width",
        "font-family: serif",
        "width: 50%",
        "width: -1px",
        "height: 1.5px",
        "padding: auto",
        "margin: 1px 2px 3px 4px 5px",
        "display: grid",
        "background-color: red",
        "width: 2147483648px",
        "width: +1px",
        "height: 1px !important",
        "width: px",
    ] {
        assert!(BoxStyle::parse(source).is_err(), "{source}");
    }
}

#[test]
fn rejects_overflow_in_width_height_and_sibling_offsets() {
    for source in [
        "<div style='width: 2147483647px; padding: 1px'></div>",
        "<div style='height: 2147483647px; padding: 1px'></div>",
        "<div style='height: 2147483647px'></div><div style='height: 1px'></div>",
        "<div style='margin: 2147483647px'></div>",
    ] {
        assert!(
            BlockLayout::new(&Document::parse(source).unwrap(), 100).is_err(),
            "{source}"
        );
    }
    assert!(BlockLayout::new(&Document::parse("").unwrap(), u32::MAX).is_err());
}

#[test]
fn zero_width_and_deep_trees_are_supported_without_recursion() {
    let doc = Document::parse("<div style='padding: 2px; margin: 1px'></div>").unwrap();
    let layout = BlockLayout::new(&doc, 0).unwrap();
    assert_eq!(layout.boxes()[1].rect, Rect::new(1, 1, 4, 4));
    assert_eq!(layout.height(), 6);
    let source = format!("{}{}", "<div>".repeat(2000), "</div>".repeat(2000));
    let doc = Document::parse_with_limits(
        &source,
        ParseLimits {
            max_depth: 2000,
            ..ParseLimits::default()
        },
    )
    .unwrap();
    let layout = BlockLayout::new(&doc, 20).unwrap();
    assert_eq!(layout.boxes().len(), 2001);
    assert_eq!(layout.height(), 0);
}
