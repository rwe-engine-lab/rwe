//! Frontend assets and window configuration; fonts/editing/IME stay in RWE.
fn main() -> Result<(), String> {
    let assets = rwe::AssetBundle::new(
        "index.html",
        &[
            ("index.html", include_str!("index.html")),
            ("style.css", include_str!("style.css")),
            ("app.js", include_str!("app.js")),
        ],
    )?;
    rwe::DesktopApp::from_assets(assets)
        .title("RWE 中文输入")
        .size(900, 640)
        .run()
}
