//! Starts the vue-template application; RWE owns the window and render lifecycle.
fn main() -> Result<(), String> {
    let entry =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/vue-template/index.html");
    rwe::DesktopApp::new(entry)
        .title("Vue Template")
        .size(960, 640)
        .min_size(320, 240)
        .background(rwe::Color::rgba(237, 244, 239, 255))
        .watch(true)
        .run()
}
