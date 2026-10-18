//! Geometry, painting and event acceptance for the bounded flex-row subset.
use rwe::{BlockLayout, Canvas, Color, Document, LayoutBox, NodeKind, Rect, Stylesheet};
fn layout(html: &str, css: &str, width: u32) -> (Document, BlockLayout) {
    let d = Document::parse(html).unwrap();
    let l = BlockLayout::with_stylesheet(&d, &Stylesheet::parse(css).unwrap(), width).unwrap();
    (d, l)
}
fn item<'a>(d: &Document, l: &'a BlockLayout, id: &str) -> &'a LayoutBox {
    l.boxes().iter().find(|b| matches!(&d.nodes()[b.node].kind, NodeKind::Element{attributes,..} if attributes.get("id").is_some_and(|v|v==id))).unwrap()
}
#[test]
fn row_counts_box_edges_gap_and_grows_content_width_before_wrapping() {
    let (d, l) = layout(
        "<main><div id='a'>AAA AAA</div><div id='b'></div></main><footer id='f'></footer>",
        "main{display:flex;gap:10px;padding:5px} div{flex-basis:20px;flex-grow:1;padding:2px;border:1px solid #000} #b{flex-grow:2;height:12px} footer{height:3px}",
        132,
    );
    // 122px content - 40px bases - 12px edges - 10px gap = 60px growth.
    assert_eq!(item(&d, &l, "a").content_rect.width, 40);
    assert_eq!(item(&d, &l, "b").content_rect.width, 60);
    assert_eq!(item(&d, &l, "a").rect.x, 5);
    assert_eq!(item(&d, &l, "b").rect.x, 61);
    assert_eq!(item(&d, &l, "f").rect.y, 48); // two 16px text lines + edges + parent padding
    assert_eq!(l.text_runs().len(), 2);
}
#[test]
fn cross_alignment_translates_nested_text_clips_and_stretches_only_auto_heights() {
    let (d, l) = layout(
        "<main><div id='a'><p id='p'>AA</p></div><div id='b'></div></main>",
        "main{display:flex;height:60px;align-items:center} div{width:30px} #a{overflow:hidden} #b{height:20px}",
        100,
    );
    assert_eq!(item(&d, &l, "a").rect.y, 22);
    assert_eq!(item(&d, &l, "p").rect.y, 22);
    assert_eq!(item(&d, &l, "a").clip_rect.unwrap().y, 22);
    assert_eq!(l.text_runs()[0].rect.y, 22);
    assert_eq!(item(&d, &l, "b").rect.y, 20);
    let (d, l) = layout(
        "<main><div id='a'></div><div id='b'></div></main>",
        "main{display:flex;height:60px} div{width:20px;margin:3px} #b{height:10px}",
        100,
    );
    assert_eq!(item(&d, &l, "a").rect.height, 54);
    assert_eq!(item(&d, &l, "b").rect.height, 10);
}
#[test]
fn justify_distribution_is_deterministic_and_hidden_items_do_not_consume_gap() {
    for (justify, xs) in [
        ("flex-start", [0, 25, 50]),
        ("center", [15, 40, 65]),
        ("flex-end", [30, 55, 80]),
        ("space-between", [0, 40, 80]),
    ] {
        let (d, l) = layout(
            "<main> <div id='a'></div><div style='display:none'><p style='bad: value'></p></div><div id='b'></div><div id='c'></div> </main>",
            &format!(
                "main{{display:flex;gap:5px;justify-content:{justify}}} div{{width:20px;height:10px}}"
            ),
            100,
        );
        assert_eq!(
            [
                item(&d, &l, "a").rect.x,
                item(&d, &l, "b").rect.x,
                item(&d, &l, "c").rect.x
            ],
            xs
        );
    }
}
#[test]
fn nested_rows_preserve_flow_height_and_paint_ancestor_clipping() {
    let (d, l) = layout(
        "<main><section id='a'><div id='n'></div><div></div></section><section id='b'></section></main><footer id='f'></footer>",
        "main{display:flex;gap:4px;width:50px;overflow:hidden} section{width:30px;flex-shrink:0} #a{display:flex;height:20px;align-items:flex-end} #a>div{width:10px;height:5px;background-color:#f00} #b{height:10px;background-color:#00f} footer{height:3px}",
        100,
    );
    assert_eq!(item(&d, &l, "f").rect.y, 20);
    assert_eq!(item(&d, &l, "n").rect, Rect::new(0, 15, 10, 5));
    let mut c = Canvas::new(100, 30).unwrap();
    l.paint(&mut c);
    let mut expected = Canvas::new(100, 30).unwrap();
    expected.fill_rect(Rect::new(0, 15, 20, 5), Color::rgba(255, 0, 0, 255));
    expected.fill_rect(Rect::new(34, 0, 16, 10), Color::rgba(0, 0, 255, 255));
    assert_eq!(c.pixels(), expected.pixels());
}
#[test]
fn unsupported_intrinsic_wrap_and_coordinate_overflow_fail_explicitly() {
    for source in [
        "<main style='display:flex'><div></div></main>",
        "<main style='display:flex'>TEXT</main>",
        "<main style='display:flex;gap:2147483647px'><div style='width:1px'></div><div style='width:1px'></div></main>",
    ] {
        let d = Document::parse(source).unwrap();
        assert!(BlockLayout::new(&d, 100).is_err(), "{source}");
    }
    assert!(Stylesheet::parse("main{flex-wrap:wrap}").is_err());
    let (d, l) = layout(
        "<main style='display:flex;display:block'><div id='a'></div><div id='b'></div></main>",
        "div{width:10px;height:10px}",
        100,
    );
    assert_eq!(item(&d, &l, "b").rect.y, 10);
}
#[cfg(feature = "javascript")]
#[test]
fn application_clicks_use_aligned_flex_geometry_after_resize() {
    use rwe::{Application, AssetBundle};
    let html = "<main style='display:flex;gap:10px;justify-content:flex-end;align-items:center;height:80px'><button id='a' style='width:80px;height:20px'>A</button><button id='b' style='width:80px;height:20px'>B</button></main><p id='result'></p><script>document.getElementById('b').onclick=()=>document.getElementById('result').textContent='CLICKED';</script>";
    let mut app =
        Application::from_assets(&AssetBundle::new("index.html", &[("index.html", html)]).unwrap())
            .unwrap();
    app.click_at(85, 10, 100).unwrap();
    assert!(!app.document().text_content(0).unwrap().contains("CLICKED"));
    assert!(app.click_at(85, 35, 100).unwrap());
    assert!(app.document().text_content(0).unwrap().contains("CLICKED"));
    assert!(app.click_at(185, 35, 200).unwrap());
}

#[test]
fn taffy_rounding_conserves_pixels_and_rejects_imprecise_geometry() {
    let (d, l) = layout(
        "<main><div id='a'></div><div></div><div></div><div id='d'></div></main>",
        "main{display:flex} div{flex-basis:0;flex-grow:4294967295}",
        1001,
    );
    assert!((250..=251).contains(&item(&d, &l, "a").content_rect.width));
    let last = item(&d, &l, "d");
    assert_eq!(last.rect.x as u32 + last.rect.width, 1001);
    let huge =
        Document::parse("<main style='display:flex'><div style='width:1px'></div></main>").unwrap();
    assert!(BlockLayout::new(&huge, 2147483647).is_err());
    let d = Document::parse("<main style='display:flex'>&nbsp;</main>").unwrap();
    assert!(BlockLayout::new(&d, 100).is_err());
}

#[test]
fn shrink_uses_scaled_bases_and_keeps_edges_gaps_and_fixed_items() {
    let (d, l) = layout(
        "<main><div id='a'></div><div id='b'></div><div id='fixed'></div></main>",
        "main{display:flex;gap:5px} div{padding:2px;border:1px solid #000} #a{width:100px} #b{width:200px} #fixed{width:20px;flex-shrink:0}",
        198,
    );
    // 150px deficit: bases 100:200 shrink by 50:100; edges and gaps remain 28px.
    assert_eq!(item(&d, &l, "a").content_rect.width, 50);
    assert_eq!(item(&d, &l, "b").content_rect.width, 100);
    let fixed = item(&d, &l, "fixed");
    assert_eq!(fixed.content_rect.width, 20);
    assert_eq!(fixed.rect.x as u32 + fixed.rect.width, 198);
}
#[test]
fn weighted_shrink_freezes_zero_items_and_redistributes_without_negative_widths() {
    let (d, l) = layout(
        "<main><div id='a'></div><div id='b'></div></main>",
        "main{display:flex} div{width:100px} #a{flex-shrink:3}",
        50,
    );
    assert_eq!(item(&d, &l, "a").content_rect.width, 0);
    assert_eq!(item(&d, &l, "b").content_rect.width, 50);
    let (d, l) = layout(
        "<main><div id='a'></div><div id='b'></div></main>",
        "main{display:flex;gap:10px} div{width:100px;padding:5px;flex-shrink:4294967295}",
        5,
    );
    assert_eq!(item(&d, &l, "a").content_rect.width, 0);
    assert_eq!(item(&d, &l, "b").content_rect.width, 0);
    assert_eq!(item(&d, &l, "b").rect.x, 20); // unshrinkable edges/gap overflow
}
#[test]
fn shrinking_reflows_nested_text_and_handles_rounding_zero_basis_and_cascade() {
    let (d, l) = layout(
        "<main><section id='a'><p>AAA AAA</p></section><section id='b'></section><section id='zero'></section></main><footer id='f'></footer>",
        "main{display:flex} section{width:80px} #zero{flex-basis:0;flex-shrink:4294967295} footer{height:1px}",
        81,
    );
    assert_eq!(item(&d, &l, "a").content_rect.width, 41);
    assert_eq!(item(&d, &l, "b").content_rect.width, 40);
    assert_eq!(item(&d, &l, "zero").content_rect.width, 0);
    assert_eq!(l.text_runs().len(), 2);
    assert_eq!(item(&d, &l, "f").rect.y, 32);
    assert!(rwe::BoxStyle::parse("flex-shrink: -1").is_err());
    assert!(rwe::BoxStyle::parse("flex-shrink: 0.5").is_err());
}

#[test]
fn shrinking_conserves_pixels_across_viewports_and_never_changes_fixed_items() {
    for width in 0..=70 {
        let (d, l) = layout(
            "<main><div id='fixed'></div><div id='a'></div><div id='b'></div><div id='c'></div></main>",
            "main{display:flex} #fixed{width:7px;flex-shrink:0} #a{width:18px;flex-shrink:4} #b{width:33px} #c{width:4px;flex-shrink:4294967295}",
            width,
        );
        let widths: Vec<_> = ["fixed", "a", "b", "c"]
            .iter()
            .map(|id| item(&d, &l, id).content_rect.width)
            .collect();
        assert_eq!(widths[0], 7);
        assert!(
            widths
                .iter()
                .zip([7, 18, 33, 4])
                .all(|(&actual, basis)| actual <= basis)
        );
        assert_eq!(widths.iter().sum::<u32>(), width.clamp(7, 62));
    }
}
