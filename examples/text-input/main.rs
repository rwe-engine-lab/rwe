//! Application configuration; editing, selection and DOM events belong to RWE.
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
        .title("RWE Text Input")
        .size(800, 600)
        .run()
}
