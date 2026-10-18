//! Native startup contracts that can be verified without opening a window.
#![cfg(feature = "window")]
use rwe::{Color, DesktopApp, WindowOptions};

#[test]
fn window_limits_cover_boundary_sizes_ordering_titles_and_opacity() {
    let mut options = WindowOptions {
        title: "我的应用".into(),
        width: 16_384,
        height: 1024,
        min_width: 1,
        min_height: 1,
        ..WindowOptions::default()
    };
    options.validate().unwrap();
    options.height = 1025; // One row beyond the total pixel budget.
    assert!(options.validate().is_err());
    options.height = 1024;
    options.width = 16_385;
    assert!(options.validate().is_err());
    options.width = 320;
    options.min_width = 321;
    assert!(options.validate().is_err());
    options.min_width = 1;
    for title in ["", "   ", "bad\0title", "two\nlines", &"x".repeat(257)] {
        options.title = title.into();
        assert!(options.validate().is_err());
    }
    options.title = "valid".into();
    options.background = Color::TRANSPARENT;
    assert!(options.validate().unwrap_err().contains("opaque"));
}

#[test]
fn invalid_configuration_is_reported_before_source_access_or_native_startup() {
    let error = DesktopApp::new("\0invalid-source")
        .size(0, 600)
        .run()
        .unwrap_err();
    assert!(error.contains("initial window"), "{error}");
    let error = DesktopApp::new("\0invalid-source")
        .size(320, 240)
        .min_size(321, 240)
        .run()
        .unwrap_err();
    assert!(error.contains("minimum window size"), "{error}");
    let error = DesktopApp::new("\0invalid-source")
        .background(Color::TRANSPARENT)
        .run()
        .unwrap_err();
    assert!(error.contains("opaque"), "{error}");
}

#[test]
fn embedded_configuration_rejects_filesystem_options_before_native_startup() {
    let assets = rwe::AssetBundle::new("index.html", &[("index.html", "<div></div>")]).unwrap();
    assert!(
        DesktopApp::from_assets(assets.clone())
            .watch(true)
            .run()
            .unwrap_err()
            .contains("embedded")
    );
    assert!(
        DesktopApp::from_assets(assets)
            .stylesheet("external.css")
            .run()
            .unwrap_err()
            .contains("embedded")
    );
}
