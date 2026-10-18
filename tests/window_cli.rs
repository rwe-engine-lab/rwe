//! Real native-launcher argument validation before any display is opened.
#![cfg(all(feature = "window", target_os = "macos"))]
use std::process::Command;

#[test]
fn help_documents_window_options_without_creating_a_native_window() {
    let output = Command::new(env!("CARGO_BIN_EXE_rwe-window"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    for flag in [
        "--title",
        "--width",
        "--height",
        "--min-width",
        "--min-height",
        "--background",
        "--fixed-size",
    ] {
        assert!(text.contains(flag), "{flag}");
    }
}

#[test]
fn invalid_window_options_fail_before_reading_a_missing_application() {
    for (args, message) in [
        (vec!["missing-page.html", "--width", "0"], "--width"),
        (vec!["missing-page.html", "--background", "#0000"], "opaque"),
        (
            vec!["missing-page.html", "--min-width", "1000"],
            "minimum window size",
        ),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_rwe-window"))
            .args(args)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains(message), "{error}");
        assert!(!error.contains("cannot open"), "{error}");
    }
}
