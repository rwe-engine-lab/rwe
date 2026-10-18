//! Starts the react-counter application; RWE owns the window and render lifecycle.
fn main() -> Result<(), String> {
    let entry =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/react-counter/index.html");
    rwe::DesktopApp::new(entry)
        .title("React Counter")
        .size(960, 640)
        .min_size(320, 240)
        .background(rwe::Color::rgba(238, 243, 250, 255))
        .watch(true)
        .run()
}
