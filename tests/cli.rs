//! CLI integration tests use isolated files and assert actual PPM bytes and failures.
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Workspace(PathBuf);
impl Workspace {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "rwe-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_rwe"))
            .current_dir(&self.0)
            .args(args)
            .output()
            .unwrap()
    }
    fn write(&self, name: &str, bytes: impl AsRef<[u8]>) {
        fs::write(self.0.join(name), bytes).unwrap();
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn failure(output: Output, message: &str) {
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(message),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn renders_file_stylesheet_to_stdout_with_auto_height_and_exact_pixels() {
    let work = Workspace::new();
    work.write(
        "page.html",
        "<main style='padding: 1px; background-color: #fff'><div class='box'></div></main>",
    );
    work.write(
        "page.css",
        "main > .box { height: 2px; background-color: #f00 }",
    );
    let output = work.run(&["render", "page.html", "--css", "page.css", "--width", "4"]);
    assert!(output.status.success(), "{:?}", output.stderr);
    assert!(output.stderr.is_empty());
    let mut expected = b"P6\n4 4\n255\n".to_vec();
    for y in 0..4 {
        for x in 0..4 {
            expected.extend(if (1..3).contains(&x) && (1..3).contains(&y) {
                [255, 0, 0]
            } else {
                [255, 255, 255]
            });
        }
    }
    assert_eq!(output.stdout, expected);
}

#[test]
fn creates_new_output_and_preserves_existing_files_including_input() {
    let work = Workspace::new();
    work.write("page.html", "<p style='height: 2px'></p>");
    let original = fs::read(work.0.join("page.html")).unwrap();
    let args = [
        "render",
        "page.html",
        "--width",
        "1",
        "--height",
        "2",
        "--background",
        "#123",
        "--output",
        "image.ppm",
    ];
    let output = work.run(&args);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert_eq!(
        fs::read(work.0.join("image.ppm")).unwrap(),
        b"P6\n1 2\n255\n\x11\x22\x33\x11\x22\x33"
    );
    failure(work.run(&args), "cannot create");
    failure(
        work.run(&["render", "page.html", "--output", "page.html"]),
        "cannot create",
    );
    assert_eq!(fs::read(work.0.join("page.html")).unwrap(), original);
}

#[test]
fn help_is_text_and_invalid_arguments_fail_without_binary_output() {
    let work = Workspace::new();
    for args in [&[][..], &["--help"], &["render", "--help"]] {
        let output = work.run(args);
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("Usage:"));
    }
    for args in [
        vec!["unknown"],
        vec!["render"],
        vec!["render", "a", "b"],
        vec!["render", "a", "--wat"],
        vec!["render", "a", "--width"],
        vec!["render", "a", "--width", "1", "--width", "2"],
    ] {
        failure(work.run(&args), "rwe:");
    }
    for value in ["0", "-1", "+1", "1.5", "16385", "4294967296"] {
        failure(
            work.run(&["render", "absent", "--width", value]),
            "--width must be",
        );
    }
    failure(
        work.run(&["render", "absent", "--background", "#fff0"]),
        "background must be opaque",
    );
    failure(
        work.run(&["render", "absent", "--background", "red"]),
        "CSS hex color",
    );
    failure(
        work.run(&["render", "absent", "--width", "16384", "--height", "16384"]),
        "pixel limit",
    );
}

#[test]
fn source_read_and_parse_errors_do_not_create_output_files() {
    let work = Workspace::new();
    failure(work.run(&["render", "missing"]), "cannot open");
    work.write("bad.html", [255_u8]);
    failure(work.run(&["render", "bad.html"]), "not valid UTF-8");
    work.write("bad.html", vec![b' '; 1024 * 1024 + 1]);
    failure(work.run(&["render", "bad.html"]), "source limit");
    work.write("bad.html", "<div>".repeat(300));
    failure(
        work.run(&["render", "bad.html", "--output", "out.ppm"]),
        "HTML",
    );
    assert!(!work.0.join("out.ppm").exists());
    work.write("page.html", "<p></p>");
    work.write("bad.css", "p { unsupported: 1px }");
    failure(
        work.run(&[
            "render",
            "page.html",
            "--css",
            "bad.css",
            "--output",
            "out.ppm",
        ]),
        "CSS",
    );
    assert!(!work.0.join("out.ppm").exists());
    work.write("bad.html", "<p style='width: 50%'></p>");
    failure(work.run(&["render", "bad.html"]), "layout:");
}

#[test]
fn auto_height_has_a_minimum_and_excludes_fixed_box_overflow() {
    let work = Workspace::new();
    work.write("empty.html", "");
    let output = work.run(&["render", "empty.html", "--width", "1", "--height", "auto"]);
    assert_eq!(output.stdout, b"P6\n1 1\n255\n\xff\xff\xff");
    work.write(
        "page.html",
        "<div style='height: 1px'><div style='height: 4px; background-color: #000'></div></div>",
    );
    let output = work.run(&["render", "page.html", "--width", "1"]);
    assert_eq!(output.stdout, b"P6\n1 1\n255\n\0\0\0");
    work.write("page.html", "<div style='height: 16385px'></div>");
    failure(
        work.run(&["render", "page.html", "--width", "1"]),
        "dimension or pixel limit",
    );
}

#[test]
fn accepts_paths_with_spaces_and_dash_prefix_after_separator() {
    let work = Workspace::new();
    work.write("-my page.html", "");
    let output = work.run(&["render", "--width", "1", "--", "-my page.html"]);
    assert!(output.status.success());
    assert_eq!(output.stdout, b"P6\n1 1\n255\n\xff\xff\xff");
}

#[path = "support/png.rs"]
mod png;

#[test]
fn infers_png_extension_and_preserves_transparency_in_file_and_stdout() {
    let work = Workspace::new();
    work.write(
        "page.html",
        "<div style='width: 1px; height: 1px; background-color: #f008'></div>",
    );
    let args = [
        "render",
        "page.html",
        "--width",
        "2",
        "--background",
        "transparent",
        "--output",
        "image.PNG",
    ];
    let output = work.run(&args);
    assert!(output.status.success(), "{:?}", output.stderr);
    assert!(output.stdout.is_empty());
    let bytes = fs::read(work.0.join("image.PNG")).unwrap();
    assert_eq!(
        png::decode(&bytes),
        (2, 1, vec![255, 0, 0, 136, 0, 0, 0, 0])
    );
    let output = work.run(&[
        "render",
        "page.html",
        "--width",
        "2",
        "--background",
        "#0000",
        "--format",
        "png",
    ]);
    assert!(output.status.success());
    assert_eq!(output.stdout, bytes);
    failure(work.run(&args), "cannot create");
}

#[test]
fn explicit_format_overrides_extension_and_invalid_formats_fail_before_file_creation() {
    let work = Workspace::new();
    work.write("empty.html", "");
    let output = work.run(&[
        "render",
        "empty.html",
        "--width",
        "1",
        "--format",
        "png",
        "--output",
        "image.ppm",
    ]);
    assert!(output.status.success());
    assert_eq!(
        png::decode(&fs::read(work.0.join("image.ppm")).unwrap()),
        (1, 1, vec![255, 255, 255, 255])
    );
    for args in [
        vec!["render", "empty.html", "--format", "jpeg"],
        vec!["render", "empty.html", "--format", "png", "--format", "ppm"],
        vec!["render", "empty.html", "--format"],
    ] {
        failure(work.run(&args), "rwe:");
    }
    failure(
        work.run(&[
            "render",
            "empty.html",
            "--format",
            "ppm",
            "--background",
            "transparent",
            "--output",
            "bad.png",
        ]),
        "background must be opaque",
    );
    assert!(!work.0.join("bad.png").exists());
}
