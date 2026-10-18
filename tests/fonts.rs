//! Real fallback glyphs, variable advances and edited/IME positions.
#![cfg(feature = "fonts")]
use rwe::{Application, BlockLayout, Canvas, Color, Document, RenderOptions};
#[test]
fn chinese_glyphs_have_distinct_ink_coverage_and_full_width_advances() {
    let app = Application::from_html("<p>你好，世界！</p>", "").unwrap();
    let layout = app.layout(200).unwrap();
    assert_eq!(layout.text_runs()[0].rect.width, 96);
    let canvas = app
        .render(RenderOptions {
            width: 200,
            ..Default::default()
        })
        .unwrap();
    let pixels = |start: u32| {
        (0..16)
            .flat_map(|y| (0..16).map(move |x| (start + x, y)))
            .map(|(x, y)| canvas.pixel(x, y).unwrap())
            .collect::<Vec<_>>()
    };
    let first = pixels(0);
    let second = pixels(16);
    assert_ne!(first, second);
    assert!(first.iter().any(|p| p.r > 0 && p.r < 255));
    assert!(first.iter().any(|p| p.r == 0));
    assert!(first.contains(&Color::WHITE));
}
#[test]
fn mixed_text_wraps_and_aligns_using_the_same_pixel_metrics_as_painting() {
    let doc = Document::parse("<p style='font-size:16px;text-align:right'>A中B文C</p>").unwrap();
    let layout = BlockLayout::new(&doc, 40).unwrap();
    assert_eq!(
        layout
            .text_runs()
            .iter()
            .map(|r| (r.text.as_str(), r.rect.x, r.rect.width))
            .collect::<Vec<_>>(),
        [("A中B", 0, 40), ("文C", 12, 28)]
    );
    let zero = BlockLayout::new(&doc, 0).unwrap();
    assert_eq!(zero.text_runs().len(), 5);
    let preserved = Application::from_html("<p style='white-space:pre'>中 A</p>", "").unwrap();
    assert_eq!(preserved.layout(2).unwrap().text_runs()[0].rect.width, 40);
}
#[test]
fn outline_alpha_and_geometry_stay_inside_ancestor_clips() {
    let app=Application::from_html("<div style='height:8px;width:12px;overflow:hidden'><p style='color:#ff000080'>中文</p></div>","").unwrap();
    let canvas = app
        .render(RenderOptions {
            width: 60,
            height: Some(24),
            ..Default::default()
        })
        .unwrap();
    let mut painted = false;
    for y in 0..24 {
        for x in 0..60 {
            let p = canvas.pixel(x, y).unwrap();
            if x >= 12 || y >= 8 {
                assert_eq!(p, Color::WHITE);
            } else if p != Color::WHITE {
                painted = true;
                assert_eq!(p.r, 255);
                assert!(p.g >= 127);
                assert_eq!(p.g, p.b);
            }
        }
    }
    assert!(painted);
}
#[test]
fn bounded_cache_eviction_does_not_change_previously_rendered_glyphs() {
    let original = Application::from_html("<p>中文</p>", "")
        .unwrap()
        .render(RenderOptions {
            width: 40,
            ..Default::default()
        })
        .unwrap();
    let many: String = (0x4e00..0x5100).filter_map(char::from_u32).collect();
    let doc = Document::parse(&format!("<p>{many}</p>")).unwrap();
    let layout = BlockLayout::new(&doc, 512).unwrap();
    let mut canvas = Canvas::new(512, layout.height()).unwrap();
    layout.paint(&mut canvas);
    let again = Application::from_html("<p>中文</p>", "")
        .unwrap()
        .render(RenderOptions {
            width: 40,
            ..Default::default()
        })
        .unwrap();
    for y in 0..16 {
        for x in 0..40 {
            assert_eq!(original.pixel(x, y), again.pixel(x, y));
        }
    }
}
#[cfg(feature = "javascript")]
#[test]
fn mixed_width_input_selection_horizontal_scroll_and_ime_use_real_advances() {
    use rwe::{AssetBundle, ImeEvent, KeyEvent, Rect};
    let assets = AssetBundle::new(
        "index.html",
        &[(
            "index.html",
            "<input id=field style='width:100px;padding:4px'>",
        )],
    )
    .unwrap();
    let mut app = Application::from_assets(&assets).unwrap();
    app.keyboard(&KeyEvent::down("Tab")).unwrap();
    app.insert_text("A中文B").unwrap();
    assert_eq!(
        app.ime_cursor_rect(140).unwrap().unwrap(),
        Rect::new(60, 4, 1, 16)
    );
    app.keyboard(&KeyEvent::down("ArrowLeft")).unwrap();
    assert_eq!(app.ime_cursor_rect(140).unwrap().unwrap().x, 48);
    let mut select = KeyEvent::down("ArrowLeft");
    select.shift_key = true;
    app.keyboard(&select).unwrap();
    assert_eq!(app.ime_cursor_rect(140).unwrap().unwrap().x, 32);
    app.ime(&ImeEvent::Preedit {
        text: "你好".into(),
        cursor: Some((6, 6)),
    })
    .unwrap();
    assert_eq!(app.ime_cursor_rect(140).unwrap().unwrap().x, 64);
    app.ime(&ImeEvent::Commit("你好".into())).unwrap();
    assert_eq!(app.layout(140).unwrap().text_runs()[0].text, "A中你好B");
    let small = AssetBundle::new(
        "index.html",
        &[("index.html", "<input style='width:40px'>")],
    )
    .unwrap();
    let mut app = Application::from_assets(&small).unwrap();
    app.keyboard(&KeyEvent::down("Tab")).unwrap();
    app.insert_text("中文中文").unwrap();
    assert_eq!(app.layout(60).unwrap().text_runs()[0].text, "中文");
    assert_eq!(app.ime_cursor_rect(60).unwrap().unwrap().x, 32);
}
