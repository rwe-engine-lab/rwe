//! A real Vue form using framework-owned control, event and rendering lifecycles.
fn main() -> Result<(), String> {
    let assets = rwe::AssetBundle::new(
        "index.html",
        &[
            ("index.html", include_str!("index.html")),
            ("style.css", include_str!("style.css")),
            ("app.js", include_str!("app.js")),
            (
                "vendor/vue.global.prod.js",
                include_str!("vendor/vue.global.prod.js"),
            ),
        ],
    )?;
    rwe::DesktopApp::from_assets(assets)
        .title("RWE 应用设置")
        .size(900, 760)
        .run()
}
