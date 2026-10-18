//! Word boundaries, preserved lines, alignment, cascade, and pixel regressions.
use rwe::{
    BlockLayout, BoxStyle, Canvas, Color, Document, Rect, Stylesheet, TextAlign, WhiteSpace,
};

#[test]
fn keeps_words_together_and_fills_exact_fit_lines_without_trailing_spaces() {
    let doc = Document::parse("<p style='font-size: 8px'>AB CD EF</p>").unwrap();
    let layout = BlockLayout::new(&doc, 30).unwrap();
    let runs = layout.text_runs();
    assert_eq!(
        runs.iter().map(|run| run.text.as_str()).collect::<Vec<_>>(),
        ["AB CD", "EF"]
    );
    assert_eq!(runs[0].rect, Rect::new(0, 0, 30, 8));
    assert_eq!(runs[1].rect, Rect::new(0, 8, 12, 8));
    assert_eq!(layout.height(), 16);
}

#[test]
fn splits_overlong_unicode_words_and_uses_the_final_fragment_for_the_next_word() {
    let doc = Document::parse("<p style='font-size: 8px'>ABCDE F 中🦀文</p>").unwrap();
    let layout = BlockLayout::new(&doc, 24).unwrap();
    assert_eq!(
        layout
            .text_runs()
            .iter()
            .map(|run| run.text.as_str())
            .collect::<Vec<_>>(),
        ["ABCD", "E F", "中🦀文"]
    );
    let narrow = BlockLayout::new(&doc, 0).unwrap();
    assert!(
        narrow
            .text_runs()
            .iter()
            .all(|run| run.text.chars().count() == 1)
    );
    assert_eq!(narrow.height(), 9 * 8);
}

#[test]
fn pre_preserves_spaces_empty_lines_crlf_and_tab_stops_without_wrapping() {
    let doc = Document::parse("<pre style='font-size: 8px; white-space: pre'>A\tB\r\n\n  C\rD\n</pre><div style='height: 1px'></div>").unwrap();
    let layout = BlockLayout::new(&doc, 12).unwrap();
    let runs = layout.text_runs();
    assert_eq!(
        runs.iter().map(|run| run.text.as_str()).collect::<Vec<_>>(),
        ["A   B", "", "  C", "D"]
    );
    assert_eq!(runs[0].rect, Rect::new(0, 0, 30, 8));
    assert_eq!(runs[1].rect, Rect::new(0, 8, 0, 8));
    assert_eq!(layout.boxes()[2].rect.y, 32);
    assert_eq!(layout.height(), 33);
}

#[test]
fn preserved_whitespace_occupies_height_but_empty_input_does_not() {
    for (source, height, lines) in [
        ("", 0, 0),
        ("\n", 8, 1),
        ("\n\n", 16, 2),
        ("  ", 8, 1),
        ("\t\tX", 8, 1),
    ] {
        let doc = Document::parse(&format!(
            "<p style='font-size: 8px; white-space: pre'>{source}</p>"
        ))
        .unwrap();
        let layout = BlockLayout::new(&doc, 10).unwrap();
        assert_eq!(
            (layout.height(), layout.text_runs().len()),
            (height, lines),
            "{source:?}"
        );
        if source == "\t\tX" {
            assert_eq!(layout.text_runs()[0].text, "        X");
        }
    }
}

#[test]
fn centers_or_right_aligns_each_line_inside_padding_and_paints_at_its_position() {
    for (alignment, offset) in [("left", 2), ("center", 11), ("right", 20)] {
        let doc = Document::parse(&format!(
            "<p style='font-size: 8px; padding: 2px; text-align: {alignment}'>A</p>"
        ))
        .unwrap();
        let layout = BlockLayout::new(&doc, 28).unwrap();
        assert_eq!(layout.text_runs()[0].rect, Rect::new(offset, 2, 6, 8));
        let mut canvas = Canvas::new(28, 12).unwrap();
        canvas.clear(Color::WHITE);
        layout.paint(&mut canvas);
        assert_eq!(canvas.pixel(offset as u32 + 1, 2), Some(Color::BLACK));
        assert_eq!(canvas.pixel(offset as u32, 2), Some(Color::WHITE));
    }
    let doc = Document::parse("<p style='font-size: 8px; text-align: center'>A BC</p>").unwrap();
    let layout = BlockLayout::new(&doc, 19).unwrap();
    assert_eq!(
        layout
            .text_runs()
            .iter()
            .map(|run| run.rect.x)
            .collect::<Vec<_>>(),
        [6, 3]
    );
    let doc =
        Document::parse("<p style='font-size: 8px; white-space: pre; text-align: right'>LONG</p>")
            .unwrap();
    assert_eq!(BlockLayout::new(&doc, 1).unwrap().text_runs()[0].rect.x, 0);
}

#[test]
fn inherits_and_cascades_alignment_and_whitespace_with_explicit_resets() {
    let doc = Document::parse("<main><p>A\nB</p><p class='reset' style='text-align: left; white-space: normal'>A\nB</p></main>").unwrap();
    let sheet = Stylesheet::parse("main { text-align: right; white-space: pre; font-size: 8px } .reset { text-align: center; white-space: pre }").unwrap();
    let layout = BlockLayout::with_stylesheet(&doc, &sheet, 30).unwrap();
    assert_eq!(
        layout
            .text_runs()
            .iter()
            .map(|run| (run.text.as_str(), run.rect.x))
            .collect::<Vec<_>>(),
        [("A", 24), ("B", 24), ("A B", 0)]
    );
    let resolved = sheet.computed_style(&doc, 4).unwrap().unwrap();
    assert_eq!(resolved.text_align, Some(TextAlign::Left));
    assert_eq!(resolved.white_space, Some(WhiteSpace::Normal));
}

#[test]
fn parses_keywords_case_insensitively_and_rejects_unsupported_values() {
    let style = BoxStyle::parse("TEXT-ALIGN: CENTER; WHITE-SPACE: PRE; text-align: RIGHT").unwrap();
    assert_eq!(style.text_align, Some(TextAlign::Right));
    assert_eq!(style.white_space, Some(WhiteSpace::Pre));
    for source in [
        "text-align: justify",
        "text-align: start",
        "text-align: inherit",
        "white-space: pre-wrap",
        "white-space: nowrap",
        "white-space: initial",
    ] {
        assert!(BoxStyle::parse(source).is_err(), "{source}");
        assert!(
            Stylesheet::parse(&format!("p {{{source}}}")).is_err(),
            "{source}"
        );
    }
}

#[test]
fn preserved_empty_lines_and_aligned_overflow_use_checked_coordinates() {
    let doc = Document::parse(
        "<div style='height: 2147483647px'></div><p style='white-space: pre'>\n</p>",
    )
    .unwrap();
    assert!(BlockLayout::new(&doc, 10).is_err());
    let doc =
        Document::parse("<p style='margin: 1px; width: 2147483647px; text-align: right'>A</p>")
            .unwrap();
    assert!(BlockLayout::new(&doc, 10).is_err());
}
