//! Verifies real app interaction using only the public framework API.
use rwe::{Application, ImeEvent, KeyEvent, RenderOptions};
use std::{fs, path::PathBuf};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("provide an output directory")?;
    fs::create_dir_all(&output)?;
    let options = RenderOptions {
        width: 480,
        min_height: 360,
        ..RenderOptions::default()
    };
    for name in ["vue", "react", "vue-template"] {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
            "examples/{}/index.html",
            match name {
                "vue" => "vue-counter",
                "react" => "react-counter",
                other => other,
            }
        ));
        let mut app = Application::load(&path, None)?;
        app.save_png(output.join(format!("{name}-before.png")), options)?;
        app.click("increment")?;
        app.save_png(output.join(format!("{name}-after.png")), options)?;
    }
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/keyboard-buttons/index.html");
    let mut app = Application::load(&path, None)?;
    app.save_png(output.join("keyboard-before.png"), options)?;
    app.keyboard(&KeyEvent::down("Tab"))?;
    app.save_png(output.join("keyboard-focused.png"), options)?;
    app.keyboard(&KeyEvent::down("Enter"))?;
    app.save_png(output.join("keyboard-after.png"), options)?;
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/text-input/index.html");
    let mut app = Application::load(&path, None)?;
    app.save_png(output.join("input-before.png"), options)?;
    app.keyboard(&KeyEvent::down("Tab"))?;
    app.insert_text("Rust")?;
    app.save_png(output.join("input-after.png"), options)?;
    let mut select = KeyEvent::down("a");
    select.meta_key = true;
    app.keyboard(&select)?;
    app.save_png(output.join("input-selected.png"), options)?;
    app.ime(&ImeEvent::Preedit {
        text: "nihao".into(),
        cursor: Some((5, 5)),
    })?;
    app.save_png(output.join("ime-preedit.png"), options)?;
    app.ime(&ImeEvent::Preedit {
        text: String::new(),
        cursor: None,
    })?;
    app.ime(&ImeEvent::Commit("RWE".into()))?;
    app.save_png(output.join("ime-committed.png"), options)?;
    #[cfg(feature = "fonts")]
    {
        let path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/chinese-input/index.html");
        let mut app = Application::load(&path, None)?;
        app.save_png(output.join("chinese-before.png"), options)?;
        app.keyboard(&KeyEvent::down("Tab"))?;
        app.ime(&ImeEvent::Preedit {
            text: "zhongwen".into(),
            cursor: Some((8, 8)),
        })?;
        app.save_png(output.join("chinese-preedit.png"), options)?;
        app.ime(&ImeEvent::Commit("中文世界".into()))?;
        app.save_png(output.join("chinese-after.png"), options)?;
    }
    #[cfg(feature = "fonts")]
    {
        let path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/settings-form/index.html");
        let mut app = Application::load(&path, None)?;
        app.save_png(output.join("settings-before.png"), options)?;
        app.save_png(
            output.join("settings-narrow.png"),
            RenderOptions {
                width: 260,
                ..options
            },
        )?;
        app.click("notify-label")?;
        app.click("advanced-label")?;
        app.click("display-name")?;
        let mut select = KeyEvent::down("a");
        select.meta_key = true;
        app.keyboard(&select)?;
        app.insert_text("框架用户")?;
        app.click("submit")?;
        app.save_png(output.join("settings-submitted.png"), options)?;
        app.click("reset")?;
        app.save_png(output.join("settings-reset.png"), options)?;
    }
    #[cfg(feature = "fonts")]
    {
        let path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/html5-recovery/index.html");
        let mut app = Application::load(&path, None)?;
        app.save_png(output.join("html5-before.png"), options)?;
        app.click("update")?;
        app.save_png(output.join("html5-after.png"), options)?;
    }
    Ok(())
}
