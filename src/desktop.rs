//! Framework-owned native application startup and validated window settings.
use crate::{Color, application::validate_canvas_size};
use std::path::PathBuf;

/// Logical window dimensions and opaque software-canvas presentation settings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WindowOptions {
    /// Application title, preserved when the host adds file/status information.
    pub title: String,
    /// Initial content width in logical pixels.
    pub width: u32,
    /// Initial content height in logical pixels.
    pub height: u32,
    /// Smallest user-resizable content width in logical pixels.
    pub min_width: u32,
    /// Smallest user-resizable content height in logical pixels.
    pub min_height: u32,
    /// Opaque background used on startup, resize, reload and page changes.
    pub background: Color,
    /// Whether the user may resize the native window.
    pub resizable: bool,
}
impl Default for WindowOptions {
    fn default() -> Self {
        Self {
            title: "RWE".into(),
            width: 800,
            height: 600,
            min_width: 160,
            min_height: 120,
            background: Color::WHITE,
            resizable: true,
        }
    }
}
impl WindowOptions {
    /// Validates dimensions, minimum/initial-size ordering, title and opacity
    /// before any source loading or native event loop is started.
    pub fn validate(&self) -> Result<(), String> {
        validate_canvas_size(self.width, self.height)
            .map_err(|e| format!("initial window: {e}"))?;
        validate_canvas_size(self.min_width, self.min_height)
            .map_err(|e| format!("minimum window: {e}"))?;
        if self.min_width > self.width || self.min_height > self.height {
            return Err("minimum window size must not exceed the initial size".into());
        }
        if self.title.trim().is_empty()
            || self.title.chars().count() > 256
            || self.title.chars().any(char::is_control)
        {
            return Err(
                "window title must contain 1..256 characters without control characters".into(),
            );
        }
        if self.background.a != 255 {
            return Err("window background must be opaque".into());
        }
        Ok(())
    }
}

/// Configures and starts the native desktop host. Applications supply a page;
/// the framework owns the window, events, reloading, scrolling and repainting.
pub struct DesktopApp {
    html: PathBuf,
    assets: Option<crate::AssetBundle>,
    css: Option<PathBuf>,
    watch: bool,
    options: WindowOptions,
}
impl DesktopApp {
    /// Opens a local HTML application's entry point.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            html: path.into(),
            assets: None,
            css: None,
            watch: false,
            options: WindowOptions::default(),
        }
    }
    /// Opens an embedded application. Embedded assets are immutable; filesystem
    /// watching and external stylesheet overrides are rejected at startup.
    pub fn from_assets(assets: crate::AssetBundle) -> Self {
        Self {
            assets: Some(assets),
            ..Self::new("")
        }
    }
    /// Appends an explicit author stylesheet after the page's own styles.
    pub fn stylesheet(mut self, path: impl Into<PathBuf>) -> Self {
        self.css = Some(path.into());
        self
    }
    /// Enables or disables automatic source reloading.
    pub fn watch(mut self, enabled: bool) -> Self {
        self.watch = enabled;
        self
    }
    /// Replaces all window settings using the shared configuration type.
    pub fn window_options(mut self, options: WindowOptions) -> Self {
        self.options = options;
        self
    }
    /// Sets the application title; source/status suffixes are added by the host.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.options.title = title.into();
        self
    }
    /// Sets the initial logical content size.
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.options.width = width;
        self.options.height = height;
        self
    }
    /// Sets the minimum logical content size.
    pub fn min_size(mut self, width: u32, height: u32) -> Self {
        self.options.min_width = width;
        self.options.min_height = height;
        self
    }
    /// Sets the opaque canvas background.
    pub fn background(mut self, color: Color) -> Self {
        self.options.background = color;
        self
    }
    /// Controls native user resizing.
    pub fn resizable(mut self, enabled: bool) -> Self {
        self.options.resizable = enabled;
        self
    }
    /// Returns the current settings for inspection before startup.
    pub fn options(&self) -> &WindowOptions {
        &self.options
    }
    /// Validates configuration and runs the native event loop on the main thread.
    /// Currently macOS only. Loading/runtime errors propagate to the caller.
    pub fn run(self) -> Result<(), String> {
        self.options.validate()?;
        if self.assets.is_some() && (self.watch || self.css.is_some()) {
            return Err(
                "embedded applications cannot watch files or use external stylesheet overrides"
                    .into(),
            );
        }
        #[cfg(target_os = "macos")]
        {
            crate::window_host::open(self.html, self.css, self.watch, self.options, self.assets)
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = (self.html, self.css, self.watch, self.assets);
            Err("native desktop currently supports macOS only".into())
        }
    }
    /// Command-line adapter used by the bundled `rwe-window` executable.
    /// Embedded applications normally use `new(...).run()` instead.
    pub fn run_cli() -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            crate::window_host::run()
        }
        #[cfg(not(target_os = "macos"))]
        {
            Err("native desktop currently supports macOS only".into())
        }
    }
}
