//! End-to-end fixture validation through the public framework API.
use rwe::{Application, Color, NodeKind, RenderOptions};
use std::path::Path;

#[test]
fn reference_page_and_overflow_fixtures_render_through_the_framework() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/rendering");
    let options = RenderOptions {
        width: 480,
        min_height: 440,
        ..RenderOptions::default()
    };
    let page =
        Application::load_static(&root.join("page.html"), Some(&root.join("page.css"))).unwrap();
    let canvas = page.render(options).unwrap();
    assert_eq!(
        canvas.pixel(0, 0),
        Some(Color::from_css_hex("#eef2f8").unwrap())
    );
    assert!(
        page.layout(480)
            .unwrap()
            .text_runs()
            .iter()
            .any(|r| r.text.contains("RWE ENGINE"))
    );

    let page = Application::load_static(
        &root.join("overflow.html"),
        Some(&root.join("overflow.css")),
    )
    .unwrap();
    let canvas = page.render(options).unwrap();
    let layout = page.layout(480).unwrap();
    let background = Color::from_css_hex("#f4f6fa").unwrap();
    for (class, clipped) in [("sample visible", false), ("sample hidden", true)] {
        let container = layout.boxes().iter().find(|b| matches!(&page.document().node(b.node).unwrap().kind,
            NodeKind::Element { attributes, .. } if attributes.get("class").is_some_and(|s| s == class))).unwrap();
        let x = container.rect.x as u32 + container.rect.width + 5;
        let y = container.content_rect.y as u32 + 20;
        assert_eq!(canvas.pixel(x, y) == Some(background), clipped, "{class}");
    }
}
