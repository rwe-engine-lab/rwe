//! Optional native macOS window for RWE's software renderer.
fn main() -> std::process::ExitCode {
    let result = rwe::DesktopApp::run_cli();
    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("rwe-window: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
