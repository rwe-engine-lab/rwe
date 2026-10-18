//! Consumer-facing framework API tests; examples need no rendering pipeline.
use rwe::{Application, Color, RenderOptions};

#[test]
fn framework_renders_exact_or_automatic_sizes_and_exports_ppm() {
    let app = Application::from_html(
        "<div style='height: 4px; background-color: #123456'></div>",
        "",
    )
    .unwrap();
    let options = RenderOptions {
        width: 2,
        min_height: 6,
        ..RenderOptions::default()
    };
    let canvas = app.render(options).unwrap();
    assert_eq!((canvas.width(), canvas.height()), (2, 6));
    assert_eq!(
        canvas.pixel(0, 0),
        Some(Color::from_css_hex("#123456").unwrap())
    );
    assert_eq!(canvas.pixel(0, 5), Some(Color::WHITE));
    let mut output = Vec::new();
    app.write_ppm(
        &mut output,
        RenderOptions {
            height: Some(1),
            ..options
        },
    )
    .unwrap();
    assert_eq!(output, b"P6\n2 1\n255\n\x12\x34\x56\x12\x34\x56");
    assert!(
        app.render(RenderOptions {
            width: 0,
            ..options
        })
        .is_err()
    );
}

#[test]
fn framework_png_export_preserves_existing_files_and_validates_before_creation() {
    let path = std::env::temp_dir().join(format!("rwe-public-export-{}.png", std::process::id()));
    let app = Application::from_html("<p>EXAMPLE</p>", "").unwrap();
    let options = RenderOptions {
        width: 120,
        ..RenderOptions::default()
    };
    app.save_png(&path, options).unwrap();
    let before = std::fs::read(&path).unwrap();
    assert!(before.starts_with(b"\x89PNG\r\n\x1a\n"));
    assert!(app.save_png(&path, options).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    std::fs::remove_file(&path).unwrap();
    assert!(
        app.save_png(
            &path,
            RenderOptions {
                height: Some(0),
                ..options
            }
        )
        .is_err()
    );
    assert!(!path.exists());
}

#[cfg(feature = "javascript")]
#[test]
fn a_consumer_runs_framework_apps_without_building_a_dom_or_canvas_pipeline() {
    for name in ["vue", "vue-template", "react"] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
            "examples/{}/index.html",
            match name {
                "vue" => "vue-counter",
                "react" => "react-counter",
                other => other,
            }
        ));
        let mut app = Application::load(&path, None).unwrap();
        let options = RenderOptions {
            width: 480,
            min_height: 360,
            ..RenderOptions::default()
        };
        let before = app.render(options).unwrap();
        app.click("increment").unwrap();
        let after = app.render(options).unwrap();
        assert!(app.document().text_content(0).unwrap().contains("COUNT: 1"));
        assert_ne!(before.pixels(), after.pixels());
        let static_app = Application::load_static(&path, None).unwrap();
        assert!(
            static_app
                .layout(480)
                .unwrap()
                .text_runs()
                .iter()
                .all(|r| !r.text.contains("COUNT:"))
        );
    }
}
