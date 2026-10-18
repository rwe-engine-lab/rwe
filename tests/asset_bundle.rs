//! Embedded applications exercise the same resource and execution pipeline.
use rwe::{Application, AssetBundle, LocalPage, RenderOptions};

#[test]
fn virtual_resources_resolve_from_entry_and_preserve_stylesheet_order() {
    let assets = AssetBundle::new("pages/./index.html", &[
        ("pages/index.html", "<link rel=stylesheet href=../shared.css><style>div { height: 8px; }</style><div></div>"),
        ("shared.css", "div { height: 4px; background-color: #123456; }"),
    ]).unwrap();
    let page = LocalPage::from_assets(&assets).unwrap();
    assert_eq!(page.resources.len(), 2);
    let app = Application::from_assets(&assets.clone()).unwrap();
    let canvas = app
        .render(RenderOptions {
            width: 2,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(canvas.height(), 8);
    assert_eq!(
        canvas.pixel(0, 0),
        Some(rwe::Color::from_css_hex("#123456").unwrap())
    );
}

#[test]
fn rejects_missing_duplicate_unsafe_and_oversized_assets() {
    assert!(AssetBundle::new("missing", &[]).is_err());
    assert!(AssetBundle::new("a", &[("a", ""), ("./a", "")]).is_err());
    for name in ["../a", "/a", "https://a", "a?x", "a#x", "a\\b", "a\0b", "."] {
        assert!(AssetBundle::new(name, &[(name, "")]).is_err(), "{name}");
    }
    assert!(AssetBundle::new("a", &[("a", &"x".repeat(1_048_577))]).is_err());
    let source = "x".repeat(1_048_576);
    let names: Vec<_> = (0..9).map(|n| n.to_string()).collect();
    let entries: Vec<_> = names
        .iter()
        .map(|n| (n.as_str(), source.as_str()))
        .collect();
    assert!(AssetBundle::new("0", &entries).is_err());
    assert!(AssetBundle::new("a", &vec![("a", ""); 257]).is_err());
}

#[test]
fn references_never_fall_back_to_files_and_share_page_limits() {
    for reference in [
        "Cargo.toml",
        "../outside.css",
        "/absolute.css",
        "https://a",
        "x?y",
        "x#y",
        "a\\b",
    ] {
        let html = format!("<link rel=stylesheet href='{reference}'>");
        let assets = AssetBundle::new("index.html", &[("index.html", &html)]).unwrap();
        assert!(LocalPage::from_assets(&assets).is_err(), "{reference}");
    }
    let html = "<link rel=stylesheet href=a.css>".repeat(33);
    let assets = AssetBundle::new("index.html", &[("index.html", &html), ("a.css", "")]).unwrap();
    assert!(
        LocalPage::from_assets(&assets)
            .err()
            .unwrap()
            .contains("32 resource")
    );
}

#[cfg(not(feature = "javascript"))]
#[test]
fn scripts_require_javascript_feature() {
    let assets = AssetBundle::new("index.html", &[("index.html", "<script>1</script>")]).unwrap();
    assert!(
        Application::from_assets(&assets)
            .err()
            .unwrap()
            .contains("JavaScript")
    );
}

#[cfg(feature = "javascript")]
#[test]
fn embedded_vue_and_react_update_without_runtime_files() {
    for example in ["vue-counter", "react-counter"] {
        let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join(example);
        let mut files = vec!["index.html", "style.css", "app.js"];
        files.extend(if example == "vue-counter" {
            vec!["vendor/vue.global.prod.js"]
        } else {
            vec![
                "vendor/react.production.min.js",
                "vendor/react-dom.production.min.js",
            ]
        });
        let texts: Vec<_> = files
            .iter()
            .map(|name| std::fs::read_to_string(base.join(name)).unwrap())
            .collect();
        let entries: Vec<_> = files
            .iter()
            .zip(&texts)
            .map(|(name, text)| (*name, text.as_str()))
            .collect();
        let assets = AssetBundle::new("index.html", &entries).unwrap();
        drop(entries);
        drop(texts);
        let mut embedded = Application::from_assets(&assets).unwrap();
        let mut local = Application::load(&base.join("index.html"), None).unwrap();
        for count in 0..3 {
            assert!(
                embedded
                    .document()
                    .text_content(0)
                    .unwrap()
                    .contains(&format!("COUNT: {count}"))
            );
            let mut embedded_png = Vec::new();
            let mut local_png = Vec::new();
            embedded
                .write_png(&mut embedded_png, RenderOptions::default())
                .unwrap();
            local
                .write_png(&mut local_png, RenderOptions::default())
                .unwrap();
            assert_eq!(embedded_png, local_png);
            embedded.click("increment").unwrap();
            local.click("increment").unwrap();
        }
    }
}
