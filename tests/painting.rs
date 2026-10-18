//! Regression coverage for CSS colors and software painting boundaries.
use rwe::{Canvas, Color, Rect};

#[test]
fn parses_all_css_hex_forms_and_cases() {
    for (input, expected) in [
        ("#aB3", Color::rgba(170, 187, 51, 255)),
        ("#aB38", Color::rgba(170, 187, 51, 136)),
        ("#01aBef", Color::rgba(1, 171, 239, 255)),
        ("#01aBef80", Color::rgba(1, 171, 239, 128)),
        ("#0000", Color::TRANSPARENT),
    ] {
        assert_eq!(Color::from_css_hex(input).unwrap(), expected, "{input}");
    }
}

#[test]
fn rejects_invalid_tokens_without_panicking_on_unicode() {
    for input in [
        "",
        "fff",
        "#",
        "#12",
        "#12345",
        "#1234567",
        "#123456789",
        "#ggg",
        "#１２",
        "#éé",
        " #fff",
        "#fff ",
        "red",
        "rgb(0,0,0)",
    ] {
        assert!(Color::from_css_hex(input).is_err(), "{input}");
    }
}

#[test]
fn source_over_handles_transparent_and_opaque_endpoints() {
    let destination = Color::rgba(5, 10, 20, 64);
    assert_eq!(Color::rgba(255, 0, 0, 0).over(destination), destination);
    assert_eq!(Color::WHITE.over(destination), Color::WHITE);
    let source = Color::rgba(70, 80, 90, 100);
    assert_eq!(source.over(Color::TRANSPARENT), source);
    assert_eq!(
        Color::rgba(255, 0, 0, 128).over(Color::WHITE),
        Color::rgba(255, 127, 127, 255)
    );
    assert_eq!(
        Color::rgba(255, 0, 0, 128).over(Color::rgba(0, 0, 255, 128)),
        Color::rgba(170, 0, 85, 192)
    );
}

#[test]
fn clips_left_top_and_preserves_other_pixels() {
    let mut canvas = Canvas::new(4, 3).unwrap();
    canvas.clear(Color::WHITE);
    canvas.fill_rect(Rect::new(-1, -1, 3, 3), Color::BLACK);
    for y in 0..3 {
        for x in 0..4 {
            let expected = if x < 2 && y < 2 {
                Color::BLACK
            } else {
                Color::WHITE
            };
            assert_eq!(canvas.pixel(x, y), Some(expected));
        }
    }
    assert_eq!(canvas.pixel(4, 0), None);
    assert_eq!(canvas.pixel(0, 3), None);
}

#[test]
fn clips_right_bottom_and_extreme_coordinates() {
    let mut canvas = Canvas::new(3, 2).unwrap();
    canvas.fill_rect(Rect::new(2, 1, u32::MAX, u32::MAX), Color::WHITE);
    assert_eq!(
        canvas
            .pixels()
            .iter()
            .filter(|&&p| p == Color::WHITE)
            .count(),
        1
    );
    canvas.fill_rect(
        Rect::new(i32::MAX, i32::MAX, u32::MAX, u32::MAX),
        Color::BLACK,
    );
    assert_eq!(canvas.pixel(2, 1), Some(Color::WHITE));
    canvas.fill_rect(
        Rect::new(i32::MIN, i32::MIN, u32::MAX, u32::MAX),
        Color::BLACK,
    );
    assert!(canvas.pixels().iter().all(|&p| p == Color::BLACK));
}

#[test]
fn zero_size_and_offscreen_rectangles_do_not_paint() {
    let mut canvas = Canvas::new(2, 2).unwrap();
    for rect in [
        Rect::new(0, 0, 0, 2),
        Rect::new(0, 0, 2, 0),
        Rect::new(-5, -5, 2, 2),
        Rect::new(2, 2, 2, 2),
    ] {
        canvas.fill_rect(rect, Color::WHITE);
    }
    assert!(canvas.pixels().iter().all(|&p| p == Color::TRANSPARENT));
    let mut empty = Canvas::new(0, 10).unwrap();
    empty.fill_rect(Rect::new(0, 0, 10, 10), Color::WHITE);
    assert!(empty.pixels().is_empty());
    assert_eq!(empty.pixel(0, 0), None);
}

#[test]
fn rejects_storage_overflow_before_allocation() {
    assert!(Canvas::new(u32::MAX, u32::MAX).is_err());
}

#[test]
fn overlapping_rectangles_have_stable_pixels_and_clip_at_the_canvas_edge() {
    let mut canvas = Canvas::new(320, 180).unwrap();
    let background = Color::from_css_hex("#f4f6fa").unwrap();
    canvas.clear(background);
    canvas.fill_rect(
        Rect::new(24, 24, 200, 100),
        Color::from_css_hex("#2364db").unwrap(),
    );
    canvas.fill_rect(
        Rect::new(120, 70, 220, 90),
        Color::from_css_hex("#f9731680").unwrap(),
    );
    assert_eq!(canvas.pixel(0, 0), Some(background));
    assert_eq!(canvas.pixel(24, 24), Some(Color::rgba(35, 100, 219, 255)));
    assert_eq!(canvas.pixel(120, 70), Some(Color::rgba(142, 108, 120, 255)));
    assert_ne!(canvas.pixel(319, 150), Some(background));
    assert_eq!(canvas.pixel(320, 150), None);
    assert_eq!(canvas.pixel(319, 160), Some(background));
}
