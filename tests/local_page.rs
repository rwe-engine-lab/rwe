//! Full-document parsing, local resource ordering and loader boundaries.
use rwe::{BlockLayout, Canvas, Color, Document, LocalPage};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Files(PathBuf);
impl Files {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "rwe-page-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn write(&self, path: &str, source: &str) {
        fs::write(self.0.join(path), source).unwrap();
    }
}
impl Drop for Files {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn documents_support_html5_raw_text_rcdata_and_recovery() {
    let html = "<!DOCTYPE html><html lang=en><head><title>A &amp; B</title><style>p { color: #123; }</style></head><body><textarea>&lt;x&gt;</textarea><script>if (1 < 2) { '&amp;'; }</script></body></html>";
    let doc = Document::parse_page(html).unwrap();
    let text = doc.text_content(0).unwrap();
    assert!(text.contains("A & B"));
    assert!(text.contains("<x>"));
    assert!(text.contains("if (1 < 2) { '&amp;'; }"));
    assert!(Document::parse_page("<!doctypehtml><p></p>").is_ok());
    assert!(Document::parse_page("<!doctype html><!doctype html>").is_ok());
    assert_eq!(
        Document::parse_page("<style>p { color: #fff }")
            .unwrap()
            .text_content(0)
            .unwrap(),
        "p { color: #fff }"
    );
    assert_eq!(
        Document::parse_page("<script>x</scriptx>")
            .unwrap()
            .text_content(0)
            .unwrap(),
        "x</scriptx>"
    );
}

#[test]
fn stylesheets_follow_document_order_and_metadata_does_not_paint() {
    let files = Files::new();
    files.write("first.css", "p { height: 2px; background-color: #f00; }");
    files.write("override.css", "p { background-color: #00f; }");
    files.write("page.html","<!doctype html><html><head><title>HIDDEN</title><link rel=stylesheet href=first.css><style>p { background-color: #0f0; }</style></head><body><p></p></body></html>");
    let page = LocalPage::load(&files.0.join("page.html"), None).unwrap();
    let layout = BlockLayout::with_stylesheet(&page.document, &page.stylesheet, 2).unwrap();
    assert_eq!(layout.height(), 2);
    assert!(layout.text_runs().is_empty());
    let mut canvas = Canvas::new(2, 2).unwrap();
    layout.paint(&mut canvas);
    assert_eq!(
        canvas.pixel(0, 0),
        Some(Color::from_css_hex("#0f0").unwrap())
    );
    let page = LocalPage::load(
        &files.0.join("page.html"),
        Some(&files.0.join("override.css")),
    )
    .unwrap();
    BlockLayout::with_stylesheet(&page.document, &page.stylesheet, 2)
        .unwrap()
        .paint(&mut canvas);
    assert_eq!(
        canvas.pixel(0, 0),
        Some(Color::from_css_hex("#00f").unwrap())
    );
}

#[test]
fn scripts_keep_source_order_and_references_stay_inside_the_page_directory() {
    let files = Files::new();
    files.write("app.js", "window.test = 2;");
    files.write(
        "page.html",
        "<script>window.test = 1;</script><script src=app.js></script>",
    );
    let page = LocalPage::load(&files.0.join("page.html"), None).unwrap();
    assert_eq!(page.scripts.len(), 2);
    assert!(page.scripts[0].source.contains("= 1"));
    assert!(page.scripts[1].source.contains("= 2"));
    for reference in [
        "https://example.com/app.js",
        "../outside.js",
        "/tmp/outside.js",
        "app.js?v=1",
    ] {
        files.write("page.html", &format!("<script src='{reference}'></script>"));
        assert!(LocalPage::load(&files.0.join("page.html"), None).is_err());
    }
    files.write("page.html", &"<script src=app.js></script>".repeat(33));
    assert!(
        LocalPage::load(&files.0.join("page.html"), None)
            .err()
            .unwrap()
            .contains("32 resource")
    );
}
