//! Text controls render without enabling a JavaScript runtime.
use rwe::{Application, RenderOptions};
#[test]
fn static_inputs_render_values_placeholders_and_clip_to_one_line() {
    let app=Application::from_html("<input type=hidden value=SECRET><input value='ABCDE' style='width:24px; padding:4px; border:1px solid #888'><input type=search placeholder=NAME>","").unwrap();
    let layout = app.layout(100).unwrap();
    assert_eq!(layout.text_runs().len(), 2);
    assert_eq!(layout.text_runs()[0].text, "AB");
    assert_eq!(layout.text_runs()[1].text, "NAME");
    assert!(layout.text_runs().iter().all(|r| r.rect.height == 16));
    assert!(
        app.render(RenderOptions {
            width: 100,
            ..Default::default()
        })
        .is_ok()
    );
    let tiny =
        Application::from_html("<input value=LONG style='width:1px;height:1px'>", "").unwrap();
    assert_eq!(tiny.layout(100).unwrap().text_runs()[0].text, "");
    assert!(
        tiny.render(RenderOptions {
            width: 100,
            ..Default::default()
        })
        .is_ok()
    );
}

#[test]
fn checkbox_radio_have_intrinsic_boxes_real_marks_and_ancestor_clipping() {
    let app = Application::from_html(
        "<input type=checkbox checked><input type=radio checked><input type=checkbox disabled>",
        "",
    )
    .unwrap();
    let layout = app.layout(80).unwrap();
    let control=layout.boxes().iter().find(|b| matches!(&app.document().nodes()[b.node].kind,rwe::NodeKind::Element{name,..} if name=="input")).unwrap();
    assert_eq!(control.content_rect.width, 16);
    assert_eq!(control.content_rect.height, 16);
    let canvas = app
        .render(RenderOptions {
            width: 80,
            height: Some(64),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(canvas.pixel(0, 0), Some(rwe::Color::rgba(37, 99, 235, 255)));
    assert_eq!(
        canvas.pixel(7, 23),
        Some(rwe::Color::rgba(37, 99, 235, 255))
    );
    assert_eq!(canvas.pixel(0, 16), Some(rwe::Color::WHITE));
    assert_eq!(
        canvas.pixel(3, 35),
        Some(rwe::Color::rgba(229, 234, 240, 255))
    );
    let clipped = Application::from_html(
        "<div style='height:8px;width:8px;overflow:hidden'><input type=checkbox checked></div>",
        "",
    )
    .unwrap();
    let canvas = clipped
        .render(RenderOptions {
            width: 40,
            height: Some(32),
            ..Default::default()
        })
        .unwrap();
    for y in 0..32 {
        for x in 0..40 {
            if x >= 8 || y >= 8 {
                assert_eq!(canvas.pixel(x, y), Some(rwe::Color::WHITE));
            }
        }
    }
}
