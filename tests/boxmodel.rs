//! Asymmetric box geometry, shorthand cascade, solid borders, and pixel regressions.
use rwe::{BlockLayout, BorderStyle, BoxStyle, Canvas, Color, Document, Edges, Rect, Stylesheet};

#[test]
fn expands_one_two_three_and_four_edge_values_and_respects_longhand_order() {
    for (source, expected) in [
        ("1px", Edges::all(1)),
        (
            "1px 2px",
            Edges {
                top: 1,
                right: 2,
                bottom: 1,
                left: 2,
            },
        ),
        (
            "1px 2px 3px",
            Edges {
                top: 1,
                right: 2,
                bottom: 3,
                left: 2,
            },
        ),
        (
            "1px 2px 3px 4px",
            Edges {
                top: 1,
                right: 2,
                bottom: 3,
                left: 4,
            },
        ),
    ] {
        let style = BoxStyle::parse(&format!("margin: {source}; padding: {source}")).unwrap();
        assert_eq!(style.margin, expected);
        assert_eq!(style.padding, expected);
    }
    let style = BoxStyle::parse(
        "padding-left: 9px; padding: 2px; padding-right: 4px; margin: 1px; margin-top: 3px",
    )
    .unwrap();
    assert_eq!(
        style.padding,
        Edges {
            right: 4,
            ..Edges::all(2)
        }
    );
    assert_eq!(
        style.margin,
        Edges {
            top: 3,
            ..Edges::all(1)
        }
    );
}

#[test]
fn includes_asymmetric_edges_and_border_in_content_box_geometry_and_sibling_flow() {
    let doc = Document::parse("<div style='width: 20px; height: 10px; margin: 1px 2px 3px 4px; padding: 5px 6px 7px 8px; border: 2px solid #000'></div><p style='height: 1px'></p>").unwrap();
    let layout = BlockLayout::new(&doc, 50).unwrap();
    assert_eq!(layout.boxes()[1].rect, Rect::new(4, 1, 38, 26));
    assert_eq!(layout.boxes()[1].content_rect, Rect::new(14, 8, 20, 10));
    assert_eq!(layout.boxes()[2].rect.y, 30);
    assert_eq!(layout.height(), 31);
}

#[test]
fn auto_width_and_text_height_account_for_each_edge_and_border() {
    let doc = Document::parse("<p style='font-size: 8px; margin: 1px 2px 3px 4px; padding: 5px 6px 7px 8px; border: 2px solid #000'>A</p>").unwrap();
    let layout = BlockLayout::new(&doc, 50).unwrap();
    assert_eq!(layout.boxes()[1].rect, Rect::new(4, 1, 44, 24));
    assert_eq!(layout.boxes()[1].content_rect, Rect::new(14, 8, 26, 8));
    assert_eq!(layout.text_runs()[0].rect, Rect::new(14, 8, 6, 8));
    assert_eq!(layout.height(), 28);
}

#[test]
fn cascades_shorthands_and_longhands_per_side_without_resetting_other_sides() {
    let doc = Document::parse(
        "<p id='target' class='card' style='padding-top: 5px; margin-right: 8px'></p>",
    )
    .unwrap();
    let sheet = Stylesheet::parse("#target { padding-left: 9px; margin-bottom: 7px } .card { padding: 1px 2px 3px 4px; margin: 2px }").unwrap();
    let style = sheet.computed_style(&doc, 1).unwrap().unwrap();
    assert_eq!(
        style.padding,
        Edges {
            top: 5,
            right: 2,
            bottom: 3,
            left: 9
        }
    );
    assert_eq!(
        style.margin,
        Edges {
            top: 2,
            right: 8,
            bottom: 7,
            left: 2
        }
    );
    let sheet = Stylesheet::parse(
        "#target { padding: 6px; padding-left: 7px } .card { padding-left: 10px }",
    )
    .unwrap();
    assert_eq!(
        sheet.computed_style(&doc, 1).unwrap().unwrap().padding,
        Edges {
            top: 5,
            left: 7,
            ..Edges::all(6)
        }
    );
}

#[test]
fn paints_background_under_border_without_double_blending_corners() {
    for size in [0, 2] {
        let doc = Document::parse(&format!("<div style='width: {size}px; height: {size}px; background-color: #00f; border: 1px solid #ff000080'></div>")).unwrap();
        let layout = BlockLayout::new(&doc, 4).unwrap();
        let mut canvas = Canvas::new(4, 4).unwrap();
        canvas.clear(Color::WHITE);
        layout.paint(&mut canvas);
        let edge = Color::rgba(255, 0, 0, 128).over(Color::rgba(0, 0, 255, 255));
        assert_eq!(canvas.pixel(0, 0), Some(edge));
        assert_eq!(canvas.pixel(size + 1, size + 1), Some(edge));
        if size == 2 {
            assert_eq!(canvas.pixel(1, 1), Some(Color::rgba(0, 0, 255, 255)));
        } else {
            assert_eq!(canvas.pixel(2, 0), Some(Color::WHITE));
        }
    }
}

#[test]
fn resolves_currentcolor_after_text_inheritance_and_honors_border_resets() {
    let doc = Document::parse("<main style='color: #f00'><div class='border'></div><div class='border' style='border: none'></div></main>").unwrap();
    let sheet = Stylesheet::parse(
        ".border { height: 1px; border: 1px solid #00f; border-color: currentcolor }",
    )
    .unwrap();
    let layout = BlockLayout::with_stylesheet(&doc, &sheet, 10).unwrap();
    assert_eq!(layout.boxes()[2].border_color, Color::rgba(255, 0, 0, 255));
    assert_eq!(layout.boxes()[2].border_width, 1);
    assert_eq!(layout.boxes()[3].border_width, 0);
    assert_eq!(layout.boxes()[3].rect.height, 1);
    let style = BoxStyle::parse("border-width: 7px; border-style: none").unwrap();
    assert_eq!(style.border_style, BorderStyle::None);
    assert_eq!(style.border_width, 7);
}

#[test]
fn borders_clip_at_canvas_edges_and_children_can_paint_over_them() {
    let doc = Document::parse("<div style='width: 1px; height: 0; border: 1px solid #000'><div style='width: 4px; height: 4px; background-color: #0f0'></div></div>").unwrap();
    let layout = BlockLayout::new(&doc, 3).unwrap();
    let mut canvas = Canvas::new(3, 3).unwrap();
    layout.paint(&mut canvas);
    assert_eq!(canvas.pixel(0, 0), Some(Color::BLACK));
    assert_eq!(canvas.pixel(1, 1), Some(Color::rgba(0, 255, 0, 255)));
    assert_eq!(canvas.pixel(2, 2), Some(Color::rgba(0, 255, 0, 255)));
}

#[test]
fn rejects_unsupported_edges_borders_and_overflow() {
    for value in [
        "margin: auto",
        "padding: -1px",
        "padding: 1px 2px 3px 4px 5px",
        "border: solid",
        "border: 1px dashed #000",
        "border: #000 solid 1px",
        "border-width: 1px 2px",
        "border-color: red",
        "border-radius: 2px",
    ] {
        assert!(BoxStyle::parse(value).is_err(), "{value}");
    }
    for value in [
        "padding: 0 2147483647px",
        "margin-left: 2147483647px; width: 1px",
        "border: 2147483647px solid #000",
        "height: 2147483647px; border: 1px solid #000",
    ] {
        let doc = Document::parse(&format!("<div style='{value}'></div>")).unwrap();
        assert!(BlockLayout::new(&doc, 10).is_err(), "{value}");
    }
}
