//! A relocatable native Vue application with no runtime asset directory.
fn main() -> Result<(), String> {
    let assets = rwe::AssetBundle::new(
        "index.html",
        &[
            ("index.html", include_str!("../vue-counter/index.html")),
            ("style.css", include_str!("../vue-counter/style.css")),
            ("app.js", include_str!("../vue-counter/app.js")),
            (
                "vendor/vue.global.prod.js",
                include_str!("../vue-counter/vendor/vue.global.prod.js"),
            ),
        ],
    )?;
    rwe::DesktopApp::from_assets(assets)
        .title("Embedded Vue Counter")
        .size(960, 640)
        .min_size(320, 240)
        .run()
}
