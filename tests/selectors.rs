//! Structural selectors, ambiguous ancestors, specificity, and rendering regressions.
use rwe::{BlockLayout, BoxStyle, Canvas, Color, Document, NodeKind, ParseLimits, Stylesheet};

fn style(document: &Document, sheet: &Stylesheet, id: &str) -> BoxStyle {
    let node = document.nodes().iter().position(|node| matches!(&node.kind, NodeKind::Element { attributes, .. } if attributes.get("id").map(String::as_str) == Some(id))).unwrap();
    sheet.computed_style(document, node).unwrap().unwrap()
}

#[test]
fn child_and_descendant_combinators_distinguish_direct_and_nested_elements() {
    let doc = Document::parse("<main><p id='direct'></p><section><p id='nested'></p></section></main><p id='outside'></p>").unwrap();
    let sheet = Stylesheet::parse("main p { height: 8px } main > p { padding: 2px }").unwrap();
    assert_eq!(
        (
            style(&doc, &sheet, "direct").height,
            style(&doc, &sheet, "direct").padding.top
        ),
        (Some(8), 2)
    );
    assert_eq!(
        (
            style(&doc, &sheet, "nested").height,
            style(&doc, &sheet, "nested").padding.top
        ),
        (Some(8), 0)
    );
    assert_eq!(style(&doc, &sheet, "outside").height, None);
}

#[test]
fn matches_compounds_mixed_chains_whitespace_comments_and_lists() {
    let doc = Document::parse("<main id='app'><section class='card'><div><p id='hit' class='note active'></p></div></section></main>").unwrap();
    for selector in [
        "#app>section.card p.note.active",
        "#app > section.card\t p.note.active",
        "MAIN\nSECTION.card > DIV > p",
        "main/* gap */ p.note",
        "#missing > p, #app .note",
    ] {
        let sheet = Stylesheet::parse(&format!("{selector} {{ height: 3px }}")).unwrap();
        assert_eq!(style(&doc, &sheet, "hit").height, Some(3), "{selector}");
    }
}

#[test]
fn retains_farther_ancestor_alternatives_when_the_nearest_one_fails_a_child_relation() {
    let doc = Document::parse("<main id='outer'><section class='a'><div><section class='a'><p id='target' class='b'></p></section></div></section></main>").unwrap();
    let sheet = Stylesheet::parse("#outer > .a .b { height: 7px }").unwrap();
    assert_eq!(style(&doc, &sheet, "target").height, Some(7));
    let doc = Document::parse("<main id='outer'><div><section class='a'><p id='target' class='b'></p></section></div></main>").unwrap();
    assert_eq!(style(&doc, &sheet, "target").height, None);
}

#[test]
fn never_uses_target_as_its_own_ancestor_or_the_document_root_as_an_element() {
    let doc = Document::parse("<main id='top' class='a'><p id='child' class='a'></p></main><p id='sibling' class='a'></p>").unwrap();
    let sheet = Stylesheet::parse(
        "* > main { height: 9px } .a .a { padding: 2px } main > * { margin: 1px }",
    )
    .unwrap();
    assert_eq!(style(&doc, &sheet, "top").height, None);
    assert_eq!(style(&doc, &sheet, "top").padding.top, 0);
    assert_eq!(style(&doc, &sheet, "child").padding.top, 2);
    assert_eq!(style(&doc, &sheet, "child").margin.top, 1);
    assert_eq!(style(&doc, &sheet, "sibling").padding.top, 0);
}

#[test]
fn sums_specificity_over_the_whole_chain_and_only_counts_matching_list_members() {
    let doc = Document::parse(
        "<main id='app'><section class='card'><p id='target' class='note'></p></section></main>",
    )
    .unwrap();
    let sheet = Stylesheet::parse("#app p { height: 2px } .card > .note { height: 9px; padding: 1px } main section p { margin: 2px } section > p { margin: 8px } #missing p, p { height: 10px } #app > section > p { height: 4px }").unwrap();
    let result = style(&doc, &sheet, "target");
    assert_eq!(
        (result.height, result.padding.top, result.margin.top),
        (Some(4), 1, 2)
    );
    let sheet =
        Stylesheet::parse("main > section p { height: 2px } main section > p { height: 3px }")
            .unwrap();
    assert_eq!(style(&doc, &sheet, "target").height, Some(3));
}

#[test]
fn rejects_incomplete_chains_and_unsupported_sibling_syntax() {
    for selector in [
        "> p",
        "p >",
        "p >> a",
        "p > > a",
        "p > , a",
        "p + a",
        "p ~ a",
        "p || a",
        "p > :hover",
        "p > [id=x]",
        "p > .",
        "p > >",
        "p > 你好",
        "p > #",
        "p > *oops",
        "p > ,",
    ] {
        assert!(
            Stylesheet::parse(&format!("{selector} {{height: 1px}}")).is_err(),
            "{selector}"
        );
    }
}

#[test]
fn structural_rules_control_layout_background_pixels_and_inherited_text() {
    let doc =
        Document::parse("<main><section class='card'><p>A</p></section></main><p>B</p>").unwrap();
    let sheet = Stylesheet::parse("p { font-size: 8px } main > section.card { padding: 1px; background-color: #fff } .card > p { height: 8px; background-color: #00f } main p { color: #f00 }").unwrap();
    let layout = BlockLayout::with_stylesheet(&doc, &sheet, 12).unwrap();
    assert_eq!(layout.height(), 18);
    assert_eq!(layout.text_runs()[0].color, Color::rgba(255, 0, 0, 255));
    assert_eq!(layout.text_runs()[1].color, Color::BLACK);
    let mut canvas = Canvas::new(12, 18).unwrap();
    layout.paint(&mut canvas);
    assert_eq!(canvas.pixel(0, 0), Some(Color::WHITE));
    assert_eq!(canvas.pixel(1, 1), Some(Color::rgba(0, 0, 255, 255)));
    assert_eq!(canvas.pixel(2, 1), Some(Color::rgba(255, 0, 0, 255)));
}

#[test]
fn deep_mixed_chains_match_iteratively_without_recursive_ancestor_search() {
    let source = format!(
        "<main id='root'>{}<p id='target'></p>{}</main>",
        "<div>".repeat(1500),
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
    let selector = format!("#root > {} p", vec!["div"; 256].join(" > "));
    let sheet = Stylesheet::parse(&format!("{selector} {{ height: 1px }}")).unwrap();
    assert_eq!(style(&doc, &sheet, "target").height, Some(1));
    let sheet = Stylesheet::parse(&format!(
        "#absent {} p {{ height: 1px }}",
        vec!["div"; 256].join(" ")
    ))
    .unwrap();
    assert_eq!(style(&doc, &sheet, "target").height, None);
}
