//! Reusable application lifecycle and bounded rendering for every host.
use crate::{BlockLayout, Canvas, Color, Document, LocalPage, Stylesheet};

pub(crate) const MAX_DIMENSION: u32 = 16_384;
const MAX_PIXELS: u64 = 16_777_216;

pub(crate) fn validate_canvas_size(width: u32, height: u32) -> Result<(), String> {
    if width == 0
        || width > MAX_DIMENSION
        || height == 0
        || height > MAX_DIMENSION
        || u64::from(width) * u64::from(height) > MAX_PIXELS
    {
        return Err("canvas exceeds dimension or pixel limit; choose dimensions 1..16384 and at most 16777216 pixels".into());
    }
    Ok(())
}

/// Shared software-rendering configuration.
#[derive(Clone, Copy, Debug)]
pub struct RenderOptions {
    /// Content width in logical pixels (1..=16384).
    pub width: u32,
    /// Exact output height; `None` fits document flow and `min_height`.
    pub height: Option<u32>,
    /// Minimum automatic height, ignored when an exact height is specified.
    pub min_height: u32,
    /// Initial canvas background; PPM export requires an opaque color.
    pub background: Color,
}
impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            width: 800,
            height: None,
            min_height: 1,
            background: Color::WHITE,
        }
    }
}

/// A framework-owned page: resource loading, optional JavaScript execution,
/// live DOM updates, layout and painting are shared by all application hosts.
pub struct Application {
    document: Document,
    stylesheet: Stylesheet,
    #[cfg(feature = "javascript")]
    runtime: Option<crate::ScriptRuntime>,
}

impl Application {
    /// Creates a static page from markup and supplied CSS, without loading
    /// external resources or executing scripts. Use `load` for local applications.
    pub fn from_html(html: &str, css: &str) -> Result<Self, String> {
        Ok(Self {
            document: Document::parse_page(html).map_err(|e| format!("HTML: {e}"))?,
            stylesheet: Stylesheet::parse(css).map_err(|e| format!("CSS: {e}"))?,
            #[cfg(feature = "javascript")]
            runtime: None,
        })
    }

    /// Loads a local application and optional additional author stylesheet.
    /// Scripts execute when the `javascript` feature is enabled; otherwise
    /// pages requiring scripts return an explicit error.
    pub fn load(path: &std::path::Path, css: Option<&std::path::Path>) -> Result<Self, String> {
        Self::from_page(LocalPage::load(path, css)?)
    }

    /// Starts an embedded application using the same script and render lifecycle
    /// as file-backed applications, without reading files at runtime.
    pub fn from_assets(assets: &crate::AssetBundle) -> Result<Self, String> {
        Self::from_page(LocalPage::from_assets(assets)?)
    }

    fn from_page(page: LocalPage) -> Result<Self, String> {
        #[cfg(feature = "javascript")]
        let mut runtime = crate::ScriptRuntime::load(&page)?;
        #[cfg(feature = "javascript")]
        let document = runtime.snapshot()?;
        #[cfg(not(feature = "javascript"))]
        let document = {
            if !page.scripts.is_empty() {
                return Err(
                    "this page needs JavaScript; rebuild with --features window,javascript".into(),
                );
            }
            page.document
        };
        Ok(Self {
            document,
            stylesheet: page.stylesheet,
            #[cfg(feature = "javascript")]
            runtime: Some(runtime),
        })
    }

    #[cfg(feature = "javascript")]
    /// Dispatches a click at page coordinates for the given layout width.
    /// Returns false when no live element is hit.
    pub fn click_at(&mut self, x: u32, y: u32, width: u32) -> Result<bool, String> {
        let Some(runtime) = &mut self.runtime else {
            return Ok(false);
        };
        let layout = BlockLayout::with_stylesheet(&self.document, &self.stylesheet, width)
            .map_err(|e| e.to_string())?;
        let target = layout
            .boxes()
            .iter()
            .rev()
            .find(|b| {
                let rect = b.rect;
                if i64::from(x) < i64::from(rect.x)
                    || i64::from(y) < i64::from(rect.y)
                    || i64::from(x) >= i64::from(rect.x) + i64::from(rect.width)
                    || i64::from(y) >= i64::from(rect.y) + i64::from(rect.height)
                {
                    return false;
                }
                // A child may overflow the box but cannot escape ancestor clipping.
                let mut parent = self.document.node(b.node).and_then(|n| n.parent);
                while let Some(id) = parent {
                    if let Some(clip) = layout
                        .boxes()
                        .iter()
                        .find(|b| b.node == id)
                        .and_then(|b| b.clip_rect)
                    {
                        if i64::from(x) < i64::from(clip.x)
                            || i64::from(y) < i64::from(clip.y)
                            || i64::from(x) >= i64::from(clip.x) + i64::from(clip.width)
                            || i64::from(y) >= i64::from(clip.y) + i64::from(clip.height)
                        {
                            return false;
                        }
                    }
                    parent = self.document.node(id).and_then(|n| n.parent);
                }
                true
            })
            .and_then(|b| self.document.node(b.node))
            .and_then(|n| match &n.kind {
                crate::NodeKind::Element { attributes, .. } => attributes
                    .get("data-rwe-node")
                    .and_then(|s| s.parse::<u64>().ok()),
                _ => None,
            });
        let Some(id) = target else { return Ok(false) };
        runtime.click_node(id)?;
        self.document = runtime.snapshot()?;
        Ok(true)
    }

    #[cfg(feature = "javascript")]
    /// Advances pending timers and jobs, returning whether the DOM changed.
    pub fn tick(&mut self) -> Result<bool, String> {
        let Some(runtime) = &mut self.runtime else {
            return Ok(false);
        };
        if !runtime.has_timers()? {
            return Ok(false);
        }
        runtime.pump()?;
        let document = runtime.snapshot()?;
        let changed = document != self.document;
        self.document = document;
        Ok(changed)
    }

    #[cfg(feature = "javascript")]
    /// Whether a host should wake to process pending timers.
    pub fn has_timers(&mut self) -> bool {
        self.runtime
            .as_mut()
            .is_some_and(|r| r.has_timers().unwrap_or(false))
    }

    /// Returns the current DOM for inspection, including script-driven updates.
    pub fn document(&self) -> &Document {
        &self.document
    }

    /// Computes layout using the current DOM and author styles.
    pub fn layout(&self, width: u32) -> Result<BlockLayout, String> {
        if width == 0 || width > MAX_DIMENSION {
            return Err("invalid viewport size".into());
        }
        BlockLayout::with_stylesheet(&self.document, &self.stylesheet, width)
            .map_err(|e| format!("layout: {e}"))
    }

    /// Loads static markup/resources without executing discovered scripts.
    /// Used by image export even in builds that enable JavaScript.
    pub fn load_static(
        path: &std::path::Path,
        css: Option<&std::path::Path>,
    ) -> Result<Self, String> {
        let page = LocalPage::load(path, css)?;
        Ok(Self {
            document: page.document,
            stylesheet: page.stylesheet,
            #[cfg(feature = "javascript")]
            runtime: None,
        })
    }

    /// Dispatches a click by HTML id and updates the framework-owned DOM.
    #[cfg(feature = "javascript")]
    pub fn click(&mut self, id: &str) -> Result<(), String> {
        let runtime = self
            .runtime
            .as_mut()
            .ok_or("page has no JavaScript runtime")?;
        runtime.click(id)?;
        self.document = runtime.snapshot()?;
        Ok(())
    }

    /// Dispatches host-independent keyboard input and refreshes the live document.
    /// Returns whether application/default handling consumed the key. Static pages
    /// without a JavaScript runtime return false.
    #[cfg(feature = "javascript")]
    pub fn keyboard(&mut self, event: &crate::KeyEvent) -> Result<bool, String> {
        event.validate()?;
        let Some(runtime) = &mut self.runtime else {
            return Ok(false);
        };
        let consumed = runtime.keyboard(event)?;
        self.document = runtime.snapshot()?;
        Ok(consumed)
    }

    /// Inserts committed text into the focused input, dispatching input events.
    /// Input is single-line, limited to 256 KiB per call, and rejects controls.
    #[cfg(feature = "javascript")]
    pub fn insert_text(&mut self, text: &str) -> Result<bool, String> {
        crate::keyboard::validate_text(text)?;
        let Some(runtime) = &mut self.runtime else {
            return Ok(false);
        };
        let consumed = runtime.insert_text(text)?;
        self.document = runtime.snapshot()?;
        Ok(consumed)
    }

    /// Updates, commits or cancels input-method composition and refreshes the DOM.
    #[cfg(feature = "javascript")]
    pub fn ime(&mut self, event: &crate::ImeEvent) -> Result<bool, String> {
        event.validate()?;
        let Some(runtime) = &mut self.runtime else {
            return Ok(false);
        };
        let consumed = runtime.ime(event)?;
        self.document = runtime.snapshot()?;
        Ok(consumed)
    }

    /// Returns whether an input-method composition session is active.
    #[cfg(feature = "javascript")]
    pub fn is_composing(&mut self) -> Result<bool, String> {
        self.runtime
            .as_mut()
            .map_or(Ok(false), |r| r.is_composing())
    }

    /// Returns the editable input caret in logical page coordinates, or None when
    /// native text input should be disabled. Hosts subtract scroll for screen use.
    #[cfg(feature = "javascript")]
    pub fn ime_cursor_rect(&self, width: u32) -> Result<Option<crate::Rect>, String> {
        let layout = self.layout(width)?;
        for b in layout.boxes() {
            if let crate::NodeKind::Element { name, attributes } =
                &self.document.node(b.node).unwrap().kind
            {
                if crate::form_control::is_text_input(name, attributes)
                    && attributes.contains_key("data-rwe-focus")
                    && !attributes.contains_key("readonly")
                    && !attributes.contains_key("disabled")
                {
                    return Ok(Some(layout.input_caret_rect(b.node).unwrap_or(
                        crate::Rect::new(
                            b.content_rect.x,
                            b.content_rect.y,
                            1,
                            b.content_rect.height.max(1),
                        ),
                    )));
                }
            }
        }
        Ok(None)
    }

    /// Clears keyboard focus and pending activation when the host loses focus.
    #[cfg(feature = "javascript")]
    pub fn blur(&mut self) -> Result<(), String> {
        if let Some(runtime) = &mut self.runtime {
            runtime.blur()?;
            self.document = runtime.snapshot()?;
        }
        Ok(())
    }

    /// Returns the focused element's content-space box for host scroll-to-focus.
    #[cfg(feature = "javascript")]
    pub fn focused_rect(&self, width: u32) -> Result<Option<crate::Rect>, String> {
        let layout = self.layout(width)?;
        Ok(layout.boxes().iter().find(|b| matches!(
            &self.document.node(b.node).unwrap().kind,
            crate::NodeKind::Element { attributes, .. } if attributes.contains_key("data-rwe-focus")
        )).map(|b| b.rect))
    }

    /// Renders the current page, applying shared dimension/allocation limits.
    pub fn render(&self, options: RenderOptions) -> Result<Canvas, String> {
        let layout = self.layout(options.width)?;
        let height = options
            .height
            .unwrap_or(layout.height().max(options.min_height));
        validate_canvas_size(options.width, height)?;
        let mut canvas = Canvas::new(options.width, height).map_err(|e| e.to_string())?;
        canvas.clear(options.background);
        layout.paint(&mut canvas);
        #[cfg(feature = "javascript")]
        self.paint_focus(&layout, &mut canvas);
        Ok(canvas)
    }

    #[cfg(feature = "javascript")]
    fn paint_focus(&self, layout: &BlockLayout, canvas: &mut Canvas) {
        for b in layout.boxes() {
            let Some(node) = self.document.node(b.node) else {
                continue;
            };
            if !matches!(&node.kind, crate::NodeKind::Element { attributes, .. } if attributes.contains_key("data-rwe-focus-visible"))
            {
                continue;
            }
            let rect = b.rect;
            let mut clip = Some(crate::Rect::new(0, 0, canvas.width(), canvas.height()));
            let mut parent = node.parent;
            while let Some(id) = parent {
                if let Some(bound) = layout
                    .boxes()
                    .iter()
                    .find(|b| b.node == id)
                    .and_then(|b| b.clip_rect)
                {
                    clip = clip.and_then(|c| c.intersection(bound));
                }
                parent = self.document.node(id).and_then(|n| n.parent);
            }
            let Some(clip) = clip else { continue };
            let thickness = 2.min(rect.width).min(rect.height);
            let right = (i64::from(rect.x) + i64::from(rect.width) - i64::from(thickness))
                .clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32;
            let bottom = (i64::from(rect.y) + i64::from(rect.height) - i64::from(thickness))
                .clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32;
            for edge in [
                crate::Rect::new(rect.x, rect.y, rect.width, thickness),
                crate::Rect::new(rect.x, bottom, rect.width, thickness),
                crate::Rect::new(rect.x, rect.y, thickness, rect.height),
                crate::Rect::new(right, rect.y, thickness, rect.height),
            ] {
                canvas.fill_rect_clipped(edge, Color::rgba(37, 99, 235, 255), clip);
            }
        }
    }

    /// Renders and writes PNG bytes, flushing the supplied writer.
    pub fn write_png(
        &self,
        mut writer: impl std::io::Write,
        options: RenderOptions,
    ) -> Result<(), String> {
        self.render(options)?
            .write_png(&mut writer)
            .map_err(|e| e.to_string())?;
        writer.flush().map_err(|e| e.to_string())
    }

    /// Renders and writes PPM bytes; the background must be opaque.
    pub fn write_ppm(
        &self,
        mut writer: impl std::io::Write,
        options: RenderOptions,
    ) -> Result<(), String> {
        self.render(options)?
            .write_ppm(&mut writer, options.background)
            .map_err(|e| e.to_string())?;
        writer.flush().map_err(|e| e.to_string())
    }

    /// Renders before creating a new PNG file; existing paths are never replaced.
    pub fn save_png(
        &self,
        path: impl AsRef<std::path::Path>,
        options: RenderOptions,
    ) -> Result<(), String> {
        use std::io::Write;
        let canvas = self.render(options)?;
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path.as_ref())
            .map_err(|e| format!("cannot create {}: {e}", path.as_ref().display()))?;
        let mut writer = std::io::BufWriter::new(file);
        canvas.write_png(&mut writer).map_err(|e| e.to_string())?;
        writer.flush().map_err(|e| e.to_string())
    }

    #[cfg(all(feature = "window", target_os = "macos"))]
    pub(crate) fn parse(html: &str, css: &str) -> Result<Self, String> {
        Self::from_html(html, css)
    }

    #[cfg(test)]
    pub(crate) fn rasterize(&self, width: u32, viewport_height: u32) -> Result<Canvas, String> {
        if viewport_height == 0 {
            return Err("invalid viewport size".into());
        }
        self.render(RenderOptions {
            width,
            min_height: viewport_height,
            ..RenderOptions::default()
        })
    }
}

/// Samples logical pixels for an opaque native display.
#[cfg(any(test, all(feature = "window", target_os = "macos")))]
pub(crate) fn display_pixel_on(
    canvas: &Canvas,
    x: u32,
    y: u32,
    scroll: u32,
    background: Color,
) -> u32 {
    let color = canvas
        .pixel(x, y.saturating_add(scroll))
        .unwrap_or(background);
    (u32::from(color.r) << 16) | (u32::from(color.g) << 8) | u32::from(color.b)
}

#[cfg(test)]
pub(crate) fn display_pixel(canvas: &Canvas, x: u32, y: u32, scroll: u32) -> u32 {
    display_pixel_on(canvas, x, y, scroll, Color::WHITE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resizing_reflows_text_and_keeps_a_white_viewport() {
        let page = Application::from_html("<p>ONE TWO THREE FOUR</p>", "").unwrap();
        let wide = page.rasterize(240, 80).unwrap();
        let narrow = page.rasterize(24, 1).unwrap();
        assert_eq!(wide.height(), 80);
        assert!(narrow.height() > 16);
        assert_eq!(wide.pixel(239, 79), Some(Color::WHITE));
    }

    #[test]
    fn scroll_samples_rows_in_rgb_order() {
        let page = Application::from_html("<div style=\"height: 2px; background-color: #123456\"></div><div style=\"height: 2px; background-color: #abcdef\"></div>", "").unwrap();
        let canvas = page.rasterize(2, 1).unwrap();
        assert_eq!(display_pixel(&canvas, 0, 0, 0), 0x123456);
        assert_eq!(display_pixel(&canvas, 0, 0, 2), 0xabcdef);
        assert_eq!(display_pixel(&canvas, 0, 0, u32::MAX), 0xffffff);
    }

    #[cfg(feature = "javascript")]
    #[test]
    fn native_coordinates_update_real_framework_dom_and_pixels() {
        for name in ["vue", "react", "vue-template"] {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
                "examples/{}/index.html",
                match name {
                    "vue" => "vue-counter",
                    "react" => "react-counter",
                    other => other,
                }
            ));
            let mut page = Application::load(&path, None).unwrap();
            let before = page.rasterize(480, 440).unwrap();
            let layout =
                BlockLayout::with_stylesheet(&page.document, &page.stylesheet, 480).unwrap();
            let button = layout.boxes().iter().find(|b| matches!(&page.document.node(b.node).unwrap().kind,
                crate::NodeKind::Element { attributes, .. } if attributes.get("id").is_some_and(|s| s == "increment"))).unwrap();
            let x = button.content_rect.x as u32 + 1;
            let y = button.content_rect.y as u32 + 1;
            assert!(page.click_at(x, y, 480).unwrap());
            assert!(page.document.text_content(0).unwrap().contains("COUNT: 1"));
            let after = page.rasterize(480, 440).unwrap();
            assert_ne!(
                before.pixels(),
                after.pixels(),
                "{name} pixels did not change"
            );
            assert!(!page.click_at(900, 900, 480).unwrap());
        }
    }

    #[test]
    fn rejects_unbounded_page_allocations_and_invalid_viewports() {
        let page = Application::from_html("<div style=\"height: 16385px\"></div>", "").unwrap();
        assert!(page.rasterize(10, 10).is_err());
        let page = Application::from_html("", "").unwrap();
        assert!(page.rasterize(0, 1).is_err());
        assert!(page.rasterize(MAX_DIMENSION + 1, 1).is_err());
        assert!(page.rasterize(MAX_DIMENSION, MAX_DIMENSION).is_err());
    }
}
