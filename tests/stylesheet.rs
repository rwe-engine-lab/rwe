//! Cascade, selector matching, strict parsing, and stylesheet rendering regressions.
use rwe::{BlockLayout, Canvas, Color, Document, Rect, Stylesheet};

#[test]
fn cascade_resolves_each_property_by_specificity_then_source_order() {
    let doc = Document::parse(
        "<div id='target' class='card active' style='padding: 0; width: auto'></div>",
    )
    .unwrap();
    let css = Stylesheet::parse("#target { height: 7px; margin: 4px; padding: 8px; width: 10px } .card.active { height: 20px; background-color: #f00 } .card { margin: 30px; background-color: #0f0 } div { height: 40px } #target { height: 9px }").unwrap();
    let style = css.computed_style(&doc, 1).unwrap().unwrap();
    assert_eq!(style.height, Some(9));
    assert_eq!(style.margin.top, 4);
    assert_eq!(style.padding.top, 0);
    assert_eq!(style.width, None);
    assert_eq!(style.background, Color::rgba(255, 0, 0, 255));
}

#[test]
fn selector_lists_use_highest_matching_specificity_and_exact_class_tokens() {
    let doc = Document::parse(
        "<DIV id='Hero' class='card active'></DIV><div id='hero' class='cardinal'></div>",
    )
    .unwrap();
    let css = Stylesheet::parse("*, #Hero { height: 3px } .card { height: 8px } DIV.card.active { padding: 2px } #hero { margin: 1px }").unwrap();
    let first = css.computed_style(&doc, 1).unwrap().unwrap();
    let second = css.computed_style(&doc, 2).unwrap().unwrap();
    assert_eq!(
        (first.height, first.padding.top, first.margin.top),
        (Some(3), 2, 0)
    );
    assert_eq!(
        (second.height, second.padding.top, second.margin.top),
        (Some(3), 0, 1)
    );
}

#[test]
fn id_specificity_beats_any_number_of_classes_and_unmatched_list_members_do_not_count() {
    let doc = Document::parse("<div id='x' class='a b c'></div>").unwrap();
    let css =
        Stylesheet::parse("#x { height: 1px } .a.b.c { height: 2px } #absent, div { height: 3px }")
            .unwrap();
    assert_eq!(
        css.computed_style(&doc, 1).unwrap().unwrap().height,
        Some(1)
    );
}

#[test]
fn renders_external_rules_with_inline_overrides_and_hidden_subtrees() {
    let doc = Document::parse("<main><div class='card'></div><div class='hidden'><p style='bad: value'></p></div><div class='card' style='height: 3px; background-color: #0f0'></div></main>").unwrap();
    let css = Stylesheet::parse("main { padding: 2px; background-color: #fff } .card { height: 4px; background-color: #f00 } .hidden { display: none }").unwrap();
    let layout = BlockLayout::with_stylesheet(&doc, &css, 10).unwrap();
    assert_eq!(layout.height(), 11);
    assert_eq!(layout.boxes().len(), 4);
    assert_eq!(layout.boxes()[3].rect, Rect::new(2, 6, 6, 3));
    let mut canvas = Canvas::new(10, 11).unwrap();
    layout.paint(&mut canvas);
    assert_eq!(canvas.pixel(2, 2), Some(Color::rgba(255, 0, 0, 255)));
    assert_eq!(canvas.pixel(2, 6), Some(Color::rgba(0, 255, 0, 255)));
    assert_eq!(canvas.pixel(9, 10), Some(Color::WHITE));
}

#[test]
fn supports_comments_empty_rules_and_absent_nodes_without_inheritance() {
    let doc = Document::parse("<main><p>text</p></main>").unwrap();
    let css = Stylesheet::parse(
        "/* 你好 */ main { height: 12PX; /* note */ height: 7px } p {} /* end */",
    )
    .unwrap();
    assert_eq!(
        css.computed_style(&doc, 1).unwrap().unwrap().height,
        Some(7)
    );
    assert_eq!(css.computed_style(&doc, 2).unwrap().unwrap().height, None);
    for id in [0, 3, 100] {
        assert_eq!(css.computed_style(&doc, id).unwrap(), None);
    }
    assert!(Stylesheet::parse("  /* empty */ ").is_ok());
}

#[test]
fn rejects_unsupported_or_malformed_rules_even_when_unmatched() {
    for source in [
        "div",
        "div {} }",
        "{ }",
        "div, {}",
        ",div {}",
        "div:hover {}",
        "[id=x] {}",
        "@media {}",
        "div { p {} }",
        "# {}",
        ". {}",
        "div { font-family: serif }",
        "d/*unterminated iv {}",
        "div { height: 1px !important }",
        ".你好 {}",
        ".2bad {}",
        "div { width: 20% }",
        "absent { unknown: 1px }",
    ] {
        assert!(Stylesheet::parse(source).is_err(), "{source}");
    }
    assert!(Stylesheet::parse(&" ".repeat(1024 * 1024 + 1)).is_err());
}

#[test]
fn stylesheet_can_restore_block_display_and_inline_values_reset_author_rules() {
    let doc = Document::parse("<div class='hidden' style='display: block; background-color: transparent; margin: 0; height: auto'></div>").unwrap();
    let css = Stylesheet::parse(
        ".hidden { display: none; background-color: #f00; margin: 9px; height: 4px }",
    )
    .unwrap();
    let style = css.computed_style(&doc, 1).unwrap().unwrap();
    assert!(style.visible);
    assert_eq!(style.background, Color::TRANSPARENT);
    assert_eq!(style.margin.top, 0);
    assert_eq!(style.height, None);
    assert_eq!(
        BlockLayout::with_stylesheet(&doc, &css, 10)
            .unwrap()
            .boxes()
            .len(),
        2
    );
}
