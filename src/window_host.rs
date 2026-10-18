//! Native window lifecycle and local-file controls.
use crate::{
    Application as Page, Canvas, Color, RenderOptions, WindowOptions,
    application::{display_pixel_on, validate_canvas_size},
};
use softbuffer::{Context, Surface};
use std::{
    collections::{BTreeSet, hash_map::DefaultHasher},
    env,
    fs::File,
    hash::{Hash, Hasher},
    io::Read,
    num::NonZeroU32,
    path::{Path, PathBuf},
    rc::Rc,
    time::{Duration, Instant},
};
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalSize},
    event::{ElementState, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowId},
};
#[cfg(feature = "javascript")]
use winit::{
    dpi::LogicalPosition,
    event::{Ime, MouseButton},
    keyboard::ModifiersState,
};

const HELP: &str = "RWE native macOS window
Usage: rwe-window [INPUT.html] [OPTIONS]
--css PATH       Additional author CSS
--watch          Reload changed sources automatically
--title TEXT     Application title (default: RWE)
--width N        Initial logical width (default: 800)
--height N       Initial logical height (default: 600)
--min-width N    Minimum logical width (default: 160)
--min-height N   Minimum logical height (default: 120)
--background HEX Opaque CSS hex backdrop (default: #fff)
--fixed-size     Disable user resizing
No arguments opens the framework start page. Local UTF-8 files only, at most 1 MiB each.
R: reload files | wheel/arrows/PageUp/PageDown: scroll | Home/End: jump | Escape: close
W: toggle auto reload (500 ms) | drop HTML then CSS to open local files
Static HTML/CSS plus optional experimental classic JavaScript. No link navigation.
";

#[derive(Clone, Default)]
struct Sources {
    assets: Option<crate::AssetBundle>,
    html: Option<PathBuf>,
    css: Option<PathBuf>,
    watch: bool,
    window: WindowOptions,
}
impl Sources {
    fn parse(args: impl Iterator<Item = std::ffi::OsString>) -> Result<Option<Self>, String> {
        let mut source = Self::default();
        let mut args = args.peekable();
        let mut positional = false;
        let mut seen = BTreeSet::new();
        fn dimension(value: std::ffi::OsString, flag: &str) -> Result<u32, String> {
            let value = value.to_str().and_then(|v| {
                if !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit()) {
                    v.parse::<u32>().ok()
                } else {
                    None
                }
            });
            value
                .filter(|v| (1..=16_384).contains(v))
                .ok_or_else(|| format!("{flag} must be an integer from 1 to 16384"))
        }
        while let Some(arg) = args.next() {
            if !positional && (arg == "--help" || arg == "-h") {
                return Ok(None);
            }
            if !positional && arg == "--" {
                positional = true;
                continue;
            }
            if !positional && arg.to_string_lossy().starts_with('-') {
                let flag = arg.to_str().ok_or("option name must be UTF-8")?;
                if !matches!(
                    flag,
                    "--watch"
                        | "--fixed-size"
                        | "--css"
                        | "--title"
                        | "--width"
                        | "--height"
                        | "--min-width"
                        | "--min-height"
                        | "--background"
                ) {
                    return Err(format!("unknown option: {flag}"));
                }
                if !seen.insert(flag.to_owned()) {
                    return Err(format!("duplicate {flag}"));
                }
                match flag {
                    "--watch" => source.watch = true,
                    "--fixed-size" => source.window.resizable = false,
                    _ => {
                        let value = args
                            .next()
                            .ok_or_else(|| format!("missing value for {flag}"))?;
                        match flag {
                            "--css" => source.css = Some(value.into()),
                            "--title" => {
                                source.window.title =
                                    value.into_string().map_err(|_| "title must be UTF-8")?
                            }
                            "--width" => source.window.width = dimension(value, flag)?,
                            "--height" => source.window.height = dimension(value, flag)?,
                            "--min-width" => source.window.min_width = dimension(value, flag)?,
                            "--min-height" => source.window.min_height = dimension(value, flag)?,
                            "--background" => {
                                source.window.background = Color::from_css_hex(
                                    value.to_str().ok_or("background must be UTF-8")?,
                                )
                                .map_err(|_| "background must be a CSS hex color")?
                            }
                            _ => unreachable!(),
                        }
                    }
                }
            } else if source.html.replace(arg.into()).is_some() {
                return Err("expected one HTML path".into());
            }
        }
        source.window.validate()?;
        if source.css.is_some() && source.html.is_none() {
            return Err("--css requires an HTML path".into());
        }
        Ok(Some(source))
    }

    fn render_page(&self, page: &Page, width: u32, height: u32) -> Result<Canvas, String> {
        if height == 0 {
            return Err("invalid viewport size".into());
        }
        page.render(RenderOptions {
            width,
            min_height: height,
            background: self.window.background,
            ..RenderOptions::default()
        })
    }

    fn load(&self) -> Result<Page, String> {
        if let Some(assets) = &self.assets {
            return Page::from_assets(assets);
        }
        if let Some(path) = &self.html {
            return Page::load(path, self.css.as_deref());
        }
        let html = include_str!("assets/start.html");
        let css = include_str!("assets/start.css");
        Page::parse(html, css)
    }
}

fn read_source(path: &Path) -> Result<String, String> {
    let mut bytes = Vec::new();
    File::open(path)
        .and_then(|file| file.take(1_048_577).read_to_end(&mut bytes))
        .map_err(|e| format!("{}: {e}", path.display()))?;
    if bytes.len() > 1_048_576 {
        return Err(format!("{} exceeds 1 MiB", path.display()));
    }
    String::from_utf8(bytes).map_err(|e| format!("{}: {e}", path.display()))
}

impl Sources {
    fn stamp(&self) -> u64 {
        // Hash bounded source contents, not timestamps: atomic saves and same-size
        // edits must be noticed. Error states are stable until a file changes.
        let mut hash = DefaultHasher::new();
        for path in [&self.html, &self.css] {
            path.hash(&mut hash);
            path.as_deref().map(read_source).hash(&mut hash);
        }
        if let Some(path) = &self.html {
            match crate::LocalPage::load(path, self.css.as_deref()) {
                Ok(page) => {
                    page.css.hash(&mut hash);
                    for script in page.scripts {
                        script.source.hash(&mut hash);
                    }
                }
                Err(error) => error.hash(&mut hash),
            }
        }
        hash.finish()
    }

    fn dropped(&self, path: PathBuf) -> Result<Self, String> {
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let mut next = self.clone();
        match extension.as_str() {
            "html" | "htm" => {
                next.assets = None;
                next.html = Some(path);
                next.css = None;
            }
            "css" if self.html.is_some() => next.css = Some(path),
            "css" => return Err("drop an HTML file before its CSS file".into()),
            _ => return Err("drop a local .html, .htm or .css file".into()),
        }
        next.watch = true;
        Ok(next)
    }
}

#[cfg(feature = "javascript")]
fn ime_screen_rect(rect: crate::Rect, scroll: f64, viewport: (u32, u32)) -> crate::Rect {
    let width = viewport.0.max(1);
    let height = viewport.1.max(1);
    let x = i64::from(rect.x).clamp(0, i64::from(width - 1)) as i32;
    let y = (f64::from(rect.y) - scroll).clamp(0.0, f64::from(height - 1)) as i32;
    crate::Rect::new(
        x,
        y,
        rect.width.max(1).min(width - x as u32),
        rect.height.max(1).min(height - y as u32),
    )
}

struct App {
    sources: Sources,
    page: Page,
    window: Option<Rc<Window>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    frame: Canvas,
    viewport: (u32, u32),
    scroll: f64,
    error: Option<String>,
    source_stamp: u64,
    next_poll: Instant,
    #[cfg(feature = "javascript")]
    cursor: Option<(u32, u32)>,
    #[cfg(feature = "javascript")]
    modifiers: ModifiersState,
    #[cfg(feature = "javascript")]
    ime_allowed: bool,
    #[cfg(feature = "javascript")]
    ime_enabled: bool,
    #[cfg(feature = "javascript")]
    ime_clear_pending: bool,
    #[cfg(feature = "javascript")]
    ime_target: Option<u64>,
    #[cfg(feature = "javascript")]
    pressed: Option<(u32, u32)>,
}
impl App {
    #[cfg(feature = "javascript")]
    fn sync_ime(&mut self) {
        let Some(window) = &self.window else { return };
        let rect = self.page.ime_cursor_rect(self.viewport.0).ok().flatten();
        let allowed = rect.is_some();
        let target = if allowed {
            self.page
                .document()
                .nodes()
                .iter()
                .find_map(|node| match &node.kind {
                    crate::NodeKind::Element { attributes, .. }
                        if attributes.contains_key("data-rwe-focus") =>
                    {
                        attributes
                            .get("data-rwe-node")
                            .and_then(|id| id.parse::<u64>().ok())
                    }
                    _ => None,
                })
        } else {
            None
        };
        // Switching between editable fields must discard the OS marked-text
        // session too, even though both fields allow IME.
        if self.ime_target != target && self.ime_allowed {
            window.set_ime_allowed(false);
            self.ime_allowed = false;
            self.ime_enabled = false;
        }
        self.ime_target = target;
        if self.ime_allowed != allowed {
            self.ime_allowed = allowed;
            window.set_ime_allowed(allowed);
            if !allowed {
                self.ime_enabled = false;
            }
        }
        if let Some(rect) = rect.filter(|_| self.ime_enabled) {
            let rect = ime_screen_rect(rect, self.scroll, self.viewport);
            window.set_ime_cursor_area(
                LogicalPosition::new(rect.x, rect.y),
                LogicalSize::new(rect.width, rect.height),
            );
        }
    }

    #[cfg(feature = "javascript")]
    fn ime(&mut self, event: &crate::ImeEvent) -> Result<bool, String> {
        let consumed = self.page.ime(event)?;
        self.ime_clear_pending = matches!(event, crate::ImeEvent::Preedit { text, .. } if text.is_empty())
            && self.page.is_composing()?;
        self.frame = self
            .sources
            .render_page(&self.page, self.viewport.0, self.viewport.1)?;
        if consumed {
            if let Some(rect) = self.page.focused_rect(self.viewport.0)? {
                let bottom = f64::from(rect.y) + f64::from(rect.height);
                if f64::from(rect.y) < self.scroll {
                    self.scroll = f64::from(rect.y).max(0.0);
                } else if bottom > self.scroll + f64::from(self.viewport.1) {
                    self.scroll = (bottom - f64::from(self.viewport.1)).max(0.0);
                }
            }
        }
        self.scroll_by(0.0);
        Ok(consumed)
    }

    #[cfg(feature = "javascript")]
    fn flush_ime_clear(&mut self) -> Result<(), String> {
        if self.ime_clear_pending {
            self.ime(&crate::ImeEvent::Cancel)?;
        }
        Ok(())
    }

    #[cfg(feature = "javascript")]
    fn keyboard(&mut self, event: &crate::KeyEvent) -> Result<bool, String> {
        let consumed = self.page.keyboard(event)?;
        let frame = self
            .sources
            .render_page(&self.page, self.viewport.0, self.viewport.1)?;
        let focused = if consumed {
            self.page.focused_rect(self.viewport.0)?
        } else {
            None
        };
        self.frame = frame;
        if let Some(rect) = focused {
            let top = f64::from(rect.y);
            let bottom = top + f64::from(rect.height);
            if top < self.scroll {
                self.scroll = top.max(0.0);
            } else if bottom > self.scroll + f64::from(self.viewport.1) {
                self.scroll = (bottom - f64::from(self.viewport.1)).max(0.0);
            }
        }
        self.scroll_by(0.0);
        Ok(consumed)
    }

    #[cfg(feature = "javascript")]
    fn blur(&mut self) -> Result<(), String> {
        self.pressed = None;
        self.cursor = None;
        self.modifiers = ModifiersState::empty();
        self.ime_clear_pending = false;
        self.page.blur()?;
        self.frame = self
            .sources
            .render_page(&self.page, self.viewport.0, self.viewport.1)?;
        self.scroll_by(0.0);
        Ok(())
    }

    fn fail(&mut self, event_loop: &ActiveEventLoop, error: String) {
        self.error = Some(error);
        event_loop.exit();
    }

    fn scroll_by(&mut self, amount: f64) {
        self.scroll = (self.scroll + amount).clamp(
            0.0,
            f64::from(self.frame.height().saturating_sub(self.viewport.1)),
        );
        #[cfg(feature = "javascript")]
        self.sync_ime();
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn draw(&mut self) -> Result<(), String> {
        let Some(window) = &self.window else {
            return Ok(());
        };
        let size = window.inner_size();
        let (Some(width), Some(height)) =
            (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
        else {
            return Ok(()); // Minimized windows have no drawable surface.
        };
        if validate_canvas_size(size.width, size.height).is_err() {
            self.report("window exceeds the display buffer limit; resize smaller");
            return Ok(());
        }
        let scale = window.scale_factor();
        let logical = size.to_logical::<u32>(scale);
        let viewport = (logical.width.max(1), logical.height.max(1));
        if self.viewport != viewport {
            self.resize_page(viewport);
        }
        self.scroll = self.scroll.clamp(
            0.0,
            f64::from(self.frame.height().saturating_sub(viewport.1)),
        );
        let surface = self.surface.as_mut().ok_or("missing window surface")?;
        surface.resize(width, height).map_err(|e| e.to_string())?;
        let mut buffer = surface.buffer_mut().map_err(|e| e.to_string())?;
        for (index, pixel) in buffer.iter_mut().enumerate() {
            let x = ((index as u32 % size.width) as f64 / scale) as u32;
            let y = ((index as u32 / size.width) as f64 / scale) as u32;
            *pixel = display_pixel_on(
                &self.frame,
                x,
                y,
                self.scroll as u32,
                self.sources.window.background,
            );
        }
        buffer.present().map_err(|e| e.to_string())
    }

    fn title(&self, status: Option<&str>) {
        if let Some(window) = &self.window {
            let name = self
                .sources
                .html
                .as_ref()
                .and_then(|p| p.file_name())
                .map(|s| s.to_string_lossy())
                .unwrap_or_else(|| {
                    self.sources
                        .assets
                        .as_ref()
                        .map_or("Demo", |a| a.entry())
                        .into()
                });
            let watch = if self.sources.assets.is_some() {
                "embedded assets"
            } else if self.sources.watch {
                "auto reload on"
            } else {
                "auto reload off"
            };
            window.set_title(&format!(
                "{} — {name} · {watch} · {}",
                self.sources.window.title,
                status.unwrap_or("R: reload · W: watch")
            ));
        }
    }

    fn report(&self, error: &str) {
        eprintln!("rwe-window: {error}");
        self.title(Some(&format!("{error} · R: retry")));
    }

    fn resize_page(&mut self, viewport: (u32, u32)) {
        self.viewport = viewport;
        match self.sources.render_page(&self.page, viewport.0, viewport.1) {
            Ok(frame) => {
                self.frame = frame;
                self.title(None);
            }
            Err(error) => self.report(&error),
        }
        #[cfg(feature = "javascript")]
        self.sync_ime();
        // A failed relayout retains the previous pixels; another resize or
        // reload can recover without destroying the window.
    }

    fn open_sources(&mut self, sources: Sources, reset_scroll: bool) -> Result<(), String> {
        // Read the stamp before loading: an edit during loading is detected on
        // the next poll, rather than accidentally acknowledged as already shown.
        let stamp = sources.stamp();
        let page = sources.load()?;
        let frame = sources.render_page(&page, self.viewport.0, self.viewport.1)?;
        self.sources = sources;
        self.page = page;
        self.frame = frame;
        self.source_stamp = stamp;
        #[cfg(feature = "javascript")]
        {
            self.pressed = None;
            self.ime_clear_pending = false;
        }
        if reset_scroll {
            self.scroll = 0.0;
        }
        self.scroll_by(0.0);
        self.title(None);
        Ok(())
    }

    fn reload(&mut self) {
        if let Err(error) = self.open_sources(self.sources.clone(), false) {
            self.report(&error);
        }
    }

    fn poll_sources(&mut self, now: Instant) {
        if !self.sources.watch || self.sources.html.is_none() || now < self.next_poll {
            return;
        }
        self.next_poll = now + Duration::from_millis(500);
        let stamp = self.sources.stamp();
        if stamp != self.source_stamp {
            // Record failures too, avoiding repeated errors until the next edit.
            self.source_stamp = stamp;
            self.reload();
        }
    }
}
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let result = (|| {
            let window = Rc::new(
                event_loop
                    .create_window(
                        Window::default_attributes()
                            .with_title(&self.sources.window.title)
                            .with_inner_size(LogicalSize::new(
                                self.sources.window.width,
                                self.sources.window.height,
                            ))
                            .with_min_inner_size(LogicalSize::new(
                                self.sources.window.min_width,
                                self.sources.window.min_height,
                            ))
                            .with_resizable(self.sources.window.resizable),
                    )
                    .map_err(|e| e.to_string())?,
            );
            let context = Context::new(window.clone()).map_err(|e| e.to_string())?;
            let surface = Surface::new(&context, window.clone()).map_err(|e| e.to_string())?;
            window.request_redraw();
            self.surface = Some(surface);
            self.window = Some(window);
            #[cfg(feature = "javascript")]
            self.sync_ime();
            self.title(None);
            Ok(())
        })();
        if let Err(error) = result {
            self.fail(event_loop, error);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        if id != window.id() {
            return;
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            #[cfg(feature = "javascript")]
            WindowEvent::Ime(event) => {
                let result = match event {
                    Ime::Enabled => {
                        self.ime_enabled = self.ime_allowed;
                        self.sync_ime();
                        Ok(false)
                    }
                    Ime::Disabled => {
                        self.ime_enabled = false;
                        self.ime(&crate::ImeEvent::Cancel)
                    }
                    Ime::Preedit(text, cursor) if self.ime_allowed && self.ime_enabled => {
                        self.ime(&crate::ImeEvent::Preedit { text, cursor })
                    }
                    Ime::Commit(text) if self.ime_allowed && self.ime_enabled => {
                        self.ime(&crate::ImeEvent::Commit(text))
                    }
                    _ => Ok(false),
                };
                if let Err(error) = result {
                    self.report(&error);
                }
            }
            #[cfg(feature = "javascript")]
            WindowEvent::CursorMoved { position, .. } => {
                let p = position.to_logical::<f64>(window.scale_factor());
                self.cursor = if p.x >= 0.0 && p.y >= 0.0 {
                    Some((p.x as u32, p.y as u32))
                } else {
                    None
                };
            }
            #[cfg(feature = "javascript")]
            WindowEvent::ModifiersChanged(modifiers) => self.modifiers = modifiers.state(),
            #[cfg(feature = "javascript")]
            WindowEvent::Focused(false) => {
                if let Err(error) = self.blur() {
                    self.report(&error);
                }
            }
            #[cfg(feature = "javascript")]
            WindowEvent::CursorLeft { .. } => {
                self.cursor = None;
                self.pressed = None;
            }
            #[cfg(feature = "javascript")]
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                if state == ElementState::Pressed {
                    self.pressed = self.cursor;
                } else if let Some((down, up)) = self.pressed.take().zip(self.cursor) {
                    if down.0.abs_diff(up.0) <= 4 && down.1.abs_diff(up.1) <= 4 {
                        let result = self
                            .page
                            .click_at(
                                up.0,
                                up.1.saturating_add(self.scroll as u32),
                                self.viewport.0,
                            )
                            .and_then(|changed| {
                                if changed {
                                    self.frame = self.sources.render_page(
                                        &self.page,
                                        self.viewport.0,
                                        self.viewport.1,
                                    )?;
                                    self.scroll_by(0.0);
                                }
                                Ok(())
                            });
                        if let Err(error) = result {
                            self.report(&error);
                        }
                    }
                }
            }
            WindowEvent::DroppedFile(path) => {
                match self
                    .sources
                    .dropped(path)
                    .and_then(|sources| self.open_sources(sources, true))
                {
                    Ok(()) => {}
                    Err(error) => self.report(&error),
                }
            }
            WindowEvent::RedrawRequested => {
                if let Err(error) = self.draw() {
                    self.fail(event_loop, error);
                }
            }
            WindowEvent::Resized(PhysicalSize { width, height }) if width != 0 && height != 0 => {
                window.request_redraw()
            }
            WindowEvent::ScaleFactorChanged { .. } => window.request_redraw(),
            WindowEvent::MouseWheel { delta, .. } => {
                let amount = match delta {
                    MouseScrollDelta::LineDelta(_, y) => -f64::from(y) * 40.0,
                    MouseScrollDelta::PixelDelta(p) => -p.y / window.scale_factor(),
                };
                self.scroll_by(amount);
            }
            WindowEvent::KeyboardInput {
                event,
                is_synthetic: false,
                ..
            } => {
                #[cfg(feature = "javascript")]
                {
                    let key = match &event.logical_key {
                        Key::Character(value) => value.to_string(),
                        Key::Named(NamedKey::Space) => " ".into(),
                        Key::Named(value) => format!("{value:?}"),
                        Key::Dead(_) => "Dead".into(),
                        Key::Unidentified(_) => "Unidentified".into(),
                    };
                    let input = crate::KeyEvent {
                        key,
                        text: Some(
                            event
                                .text
                                .as_ref()
                                .map_or("", |t| t.as_str())
                                .chars()
                                .filter(|c| !c.is_control())
                                .collect(),
                        ),
                        pressed: event.state == ElementState::Pressed,
                        repeat: event.repeat,
                        shift_key: self.modifiers.shift_key(),
                        ctrl_key: self.modifiers.control_key(),
                        alt_key: self.modifiers.alt_key(),
                        meta_key: self.modifiers.super_key(),
                    };
                    match self.keyboard(&input) {
                        Ok(true) => return,
                        Ok(false) => {}
                        Err(error) => {
                            self.report(&error);
                            return;
                        }
                    }
                    if input.ctrl_key || input.alt_key || input.meta_key {
                        return;
                    }
                }
                if event.state != ElementState::Pressed {
                    return;
                }
                match event.logical_key {
                    Key::Named(NamedKey::Escape) => event_loop.exit(),
                    Key::Named(NamedKey::ArrowDown) => self.scroll_by(40.0),
                    Key::Named(NamedKey::ArrowUp) => self.scroll_by(-40.0),
                    Key::Named(NamedKey::PageDown) => {
                        self.scroll_by(f64::from(self.viewport.1) * 0.9)
                    }
                    Key::Named(NamedKey::PageUp) => {
                        self.scroll_by(-f64::from(self.viewport.1) * 0.9)
                    }
                    Key::Named(NamedKey::Home) => self.scroll_by(-f64::MAX),
                    Key::Named(NamedKey::End) => self.scroll_by(f64::MAX),
                    Key::Character(key) if key.eq_ignore_ascii_case("w") && !event.repeat => {
                        self.sources.watch = self.sources.assets.is_none() && !self.sources.watch;
                        self.next_poll = Instant::now();
                        self.title(None);
                    }
                    Key::Character(key) if key.eq_ignore_ascii_case("r") && !event.repeat => {
                        self.reload()
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        #[cfg(feature = "javascript")]
        if let Err(error) = self.flush_ime_clear() {
            self.report(&error);
        }
        self.poll_sources(Instant::now());
        #[cfg(feature = "javascript")]
        if let Err(error) = self.page.tick().and_then(|changed| {
            if changed {
                self.frame =
                    self.sources
                        .render_page(&self.page, self.viewport.0, self.viewport.1)?;
                self.scroll_by(0.0);
            }
            Ok(())
        }) {
            self.report(&error);
        }
        #[cfg(feature = "javascript")]
        if self.page.has_timers() {
            let tick = Instant::now() + Duration::from_millis(16);
            event_loop.set_control_flow(ControlFlow::WaitUntil(
                if self.sources.watch && self.sources.html.is_some() {
                    tick.min(self.next_poll)
                } else {
                    tick
                },
            ));
            return;
        }
        event_loop.set_control_flow(if self.sources.watch && self.sources.html.is_some() {
            ControlFlow::WaitUntil(self.next_poll)
        } else {
            ControlFlow::Wait
        });
    }
}

pub(crate) fn run() -> Result<(), String> {
    let Some(sources) = Sources::parse(env::args_os().skip(1))? else {
        print!("{HELP}");
        return Ok(());
    };
    run_sources(sources)
}

pub(crate) fn open(
    html: PathBuf,
    css: Option<PathBuf>,
    watch: bool,
    window: WindowOptions,
    assets: Option<crate::AssetBundle>,
) -> Result<(), String> {
    run_sources(Sources {
        html: if assets.is_some() { None } else { Some(html) },
        assets,
        css,
        watch,
        window,
    })
}

fn run_sources(sources: Sources) -> Result<(), String> {
    sources.window.validate()?;
    let viewport = (sources.window.width, sources.window.height);
    let source_stamp = sources.stamp();
    let page = sources.load()?;
    let frame = sources.render_page(&page, viewport.0, viewport.1)?;
    let mut app = App {
        sources,
        page,
        window: None,
        surface: None,
        frame,
        viewport,
        scroll: 0.0,
        error: None,
        source_stamp,
        next_poll: Instant::now(),
        #[cfg(feature = "javascript")]
        cursor: None,
        #[cfg(feature = "javascript")]
        modifiers: ModifiersState::empty(),
        #[cfg(feature = "javascript")]
        ime_allowed: false,
        #[cfg(feature = "javascript")]
        ime_enabled: false,
        #[cfg(feature = "javascript")]
        ime_clear_pending: false,
        #[cfg(feature = "javascript")]
        ime_target: None,
        #[cfg(feature = "javascript")]
        pressed: None,
    };
    let event_loop = EventLoop::new().map_err(|e| e.to_string())?;
    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop.run_app(&mut app).map_err(|e| e.to_string())?;
    app.error.map_or(Ok(()), Err)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::display_pixel;
    fn parse(args: &[&str]) -> Result<Option<Sources>, String> {
        Sources::parse(args.iter().map(std::ffi::OsString::from))
    }
    #[test]
    fn embedded_host_loads_without_files_and_drop_switches_to_file_mode() {
        let assets = crate::AssetBundle::new(
            "index.html",
            &[("index.html", "<div style='height: 4px'></div>")],
        )
        .unwrap();
        let sources = Sources {
            assets: Some(assets),
            ..Sources::default()
        };
        let page = sources.load().unwrap();
        assert!(sources.render_page(&page, 20, 20).is_ok());
        let next = sources.dropped(PathBuf::from("other.html")).unwrap();
        assert!(next.assets.is_none());
        assert_eq!(next.html, Some(PathBuf::from("other.html")));
        assert!(next.watch);
        assert_eq!(next.window, sources.window);
    }

    #[cfg(feature = "javascript")]
    #[test]
    fn keyboard_reveals_focus_and_native_blur_cancels_pending_activation() {
        let assets = crate::AssetBundle::new("index.html", &[("index.html", "<div style='height: 40px'></div><button id=go style='height: 20px'>READY</button><script>document.getElementById('go').onclick=()=>document.getElementById('go').textContent='CLICKED';</script>")]).unwrap();
        let mut app = app(Sources {
            assets: Some(assets),
            ..Sources::default()
        });
        assert!(app.keyboard(&crate::KeyEvent::down("Tab")).unwrap());
        assert_eq!(app.scroll, 40.0);
        app.scroll_by(-40.0);
        assert!(!app.keyboard(&crate::KeyEvent::down("ArrowDown")).unwrap());
        assert_eq!(app.scroll, 0.0);
        app.keyboard(&crate::KeyEvent::down(" ")).unwrap();
        app.modifiers = ModifiersState::SHIFT;
        app.blur().unwrap();
        assert!(app.modifiers.is_empty());
        assert!(!app.keyboard(&crate::KeyEvent::up(" ")).unwrap());
        assert!(
            app.page
                .document()
                .text_content(0)
                .unwrap()
                .contains("READY")
        );
        assert!(
            !app.page
                .document()
                .text_content(0)
                .unwrap()
                .contains("CLICKED")
        );
    }

    #[cfg(feature = "javascript")]
    #[test]
    fn native_ime_cancel_and_cursor_coordinates_respect_scroll_and_viewport() {
        let assets = crate::AssetBundle::new(
            "index.html",
            &[(
                "index.html",
                "<div style='height:40px'></div><input id=field style='width:80px'>",
            )],
        )
        .unwrap();
        let mut app = app(Sources {
            assets: Some(assets),
            ..Sources::default()
        });
        app.keyboard(&crate::KeyEvent::down("Tab")).unwrap();
        assert_eq!(app.scroll, 36.0);
        app.ime(&crate::ImeEvent::Preedit {
            text: "draft".into(),
            cursor: Some((5, 5)),
        })
        .unwrap();
        assert!(app.page.is_composing().unwrap());
        let cursor = app.page.ime_cursor_rect(20).unwrap().unwrap();
        let screen = ime_screen_rect(cursor, app.scroll, app.viewport);
        assert_eq!(screen.x, 19);
        assert_eq!(screen.y, 4);
        assert_eq!(screen.width, 1);
        assert_eq!(screen.height, 16);
        app.blur().unwrap();
        assert!(!app.page.is_composing().unwrap());
        assert!(!app.ime(&crate::ImeEvent::Commit("late".into())).unwrap());
        let invalid = ime_screen_rect(crate::Rect::new(-100, 999, 0, 999), 0.0, (0, 0));
        assert_eq!(invalid, crate::Rect::new(0, 0, 1, 1));
    }

    #[cfg(feature = "javascript")]
    #[test]
    fn native_empty_preedit_waits_for_same_batch_commit_or_cancels_at_idle() {
        let assets =
            crate::AssetBundle::new("index.html", &[("index.html", "<input id=field>")]).unwrap();
        let mut app = app(Sources {
            assets: Some(assets),
            ..Sources::default()
        });
        app.keyboard(&crate::KeyEvent::down("Tab")).unwrap();
        app.ime(&crate::ImeEvent::Preedit {
            text: "draft".into(),
            cursor: None,
        })
        .unwrap();
        app.ime(&crate::ImeEvent::Preedit {
            text: String::new(),
            cursor: None,
        })
        .unwrap();
        assert!(app.ime_clear_pending);
        app.ime(&crate::ImeEvent::Commit("é".into())).unwrap();
        assert!(!app.ime_clear_pending);
        app.flush_ime_clear().unwrap();
        app.ime(&crate::ImeEvent::Preedit {
            text: "other".into(),
            cursor: None,
        })
        .unwrap();
        app.ime(&crate::ImeEvent::Preedit {
            text: String::new(),
            cursor: None,
        })
        .unwrap();
        app.flush_ime_clear().unwrap();
        assert!(!app.page.is_composing().unwrap());
        assert!(!app.ime_clear_pending);
        app.keyboard(&crate::KeyEvent::down("r")).unwrap();
        let field = app
            .page
            .document()
            .nodes()
            .iter()
            .find_map(|n| match &n.kind {
                crate::NodeKind::Element { attributes, .. }
                    if attributes.get("id").is_some_and(|s| s == "field") =>
                {
                    attributes.get("value")
                }
                _ => None,
            })
            .unwrap();
        assert_eq!(field, "ér");
    }

    #[test]
    fn failed_reload_preserves_page_and_scroll_is_bounded() {
        let page = Page::parse(
            "<div style=\"height: 100px; background-color: #123456\"></div>",
            "",
        )
        .unwrap();
        let frame = page.rasterize(20, 20).unwrap();
        let mut app = App {
            sources: Sources {
                assets: None,
                html: Some(PathBuf::from("\0invalid-path")),
                css: None,
                watch: false,
                window: WindowOptions::default(),
            },
            page,
            frame,
            viewport: (20, 20),
            scroll: 0.0,
            window: None,
            surface: None,
            error: None,
            source_stamp: 0,
            next_poll: Instant::now(),
            #[cfg(feature = "javascript")]
            cursor: None,
            #[cfg(feature = "javascript")]
            modifiers: ModifiersState::empty(),
            #[cfg(feature = "javascript")]
            ime_allowed: false,
            #[cfg(feature = "javascript")]
            ime_enabled: false,
            #[cfg(feature = "javascript")]
            ime_clear_pending: false,
            #[cfg(feature = "javascript")]
            ime_target: None,
            #[cfg(feature = "javascript")]
            pressed: None,
        };
        app.scroll_by(f64::MAX);
        assert_eq!(app.scroll, 80.0);
        app.reload();
        assert_eq!(app.frame.height(), 100);
        assert_eq!(display_pixel(&app.frame, 0, 0, 0), 0x123456);
        assert_eq!(app.scroll, 80.0);
        app.scroll_by(-f64::MAX);
        assert_eq!(app.scroll, 0.0);
    }

    struct TestFiles(PathBuf);
    impl TestFiles {
        fn new() -> Self {
            static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let id = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let path = env::temp_dir().join(format!("rwe-window-{}-{id}", std::process::id()));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for TestFiles {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    fn app(sources: Sources) -> App {
        let page = sources.load().unwrap();
        let frame = sources.render_page(&page, 20, 20).unwrap();
        App {
            source_stamp: sources.stamp(),
            sources,
            page,
            frame,
            viewport: (20, 20),
            scroll: 0.0,
            window: None,
            surface: None,
            error: None,
            next_poll: Instant::now(),
            #[cfg(feature = "javascript")]
            cursor: None,
            #[cfg(feature = "javascript")]
            modifiers: ModifiersState::empty(),
            #[cfg(feature = "javascript")]
            ime_allowed: false,
            #[cfg(feature = "javascript")]
            ime_enabled: false,
            #[cfg(feature = "javascript")]
            ime_clear_pending: false,
            #[cfg(feature = "javascript")]
            ime_target: None,
            #[cfg(feature = "javascript")]
            pressed: None,
        }
    }

    #[test]
    fn watches_same_size_edits_and_recovers_from_invalid_deleted_files() {
        let files = TestFiles::new();
        let html = files.0.join("page.html");
        let css = files.0.join("page.css");
        std::fs::write(&html, "<div></div>").unwrap();
        std::fs::write(&css, "div { height: 100px; background-color: #123456; }").unwrap();
        let mut app = app(Sources {
            assets: None,
            html: Some(html.clone()),
            css: Some(css.clone()),
            watch: true,
            window: WindowOptions::default(),
        });
        let now = Instant::now();
        std::fs::write(&css, "div { height: 100px; background-color: #abcdef; }").unwrap();
        app.poll_sources(now + Duration::from_secs(1));
        assert_eq!(display_pixel(&app.frame, 0, 0, 0), 0xabcdef);
        std::fs::write(&html, "<div>".repeat(300)).unwrap();
        app.poll_sources(now + Duration::from_secs(2));
        assert_eq!(display_pixel(&app.frame, 0, 0, 0), 0xabcdef);
        let failed_stamp = app.source_stamp;
        app.poll_sources(now + Duration::from_secs(3));
        assert_eq!(app.source_stamp, failed_stamp);
        std::fs::remove_file(&html).unwrap();
        app.poll_sources(now + Duration::from_secs(4));
        assert_eq!(display_pixel(&app.frame, 0, 0, 0), 0xabcdef);
        std::fs::write(&html, "<div style=\"background-color: #fedcba\"></div>").unwrap();
        app.scroll = 80.0;
        app.poll_sources(now + Duration::from_secs(5));
        assert_eq!(display_pixel(&app.frame, 0, 0, 0), 0xfedcba);
        app.sources.watch = false;
        std::fs::write(&html, "<div>".repeat(300)).unwrap();
        let stamp = app.source_stamp;
        app.poll_sources(now + Duration::from_secs(6));
        assert_eq!(app.source_stamp, stamp);
    }

    #[test]
    fn drops_switch_files_transactionally_and_relayout_can_recover() {
        let files = TestFiles::new();
        let html = files.0.join("new.HTML");
        let css = files.0.join("new.CSS");
        std::fs::write(&html, "<div style=\"height: 100px\"></div>").unwrap();
        std::fs::write(&css, "div { background-color: #123456; }").unwrap();
        let mut app = app(Sources::default());
        assert!(app.sources.dropped(css.clone()).is_err());
        assert!(app.sources.dropped(files.0.join("unknown.js")).is_err());
        let next = app.sources.dropped(html.clone()).unwrap();
        assert!(next.watch);
        assert!(next.css.is_none());
        app.open_sources(next, true).unwrap();
        app.scroll = 50.0;
        let next = app.sources.dropped(css.clone()).unwrap();
        app.open_sources(next, true).unwrap();
        assert_eq!(app.scroll, 0.0);
        assert_eq!(display_pixel(&app.frame, 0, 0, 0), 0x123456);
        std::fs::write(&html, "<div>".repeat(300)).unwrap();
        assert!(
            app.open_sources(app.sources.dropped(html.clone()).unwrap(), true)
                .is_err()
        );
        assert_eq!(app.sources.css, Some(css));
        app.resize_page((16_385, 20));
        assert_eq!(app.frame.width(), 20);
        assert!(app.error.is_none());
        app.resize_page((40, 20));
        assert_eq!(app.frame.width(), 40);
        assert_eq!(display_pixel(&app.frame, 0, 0, 0), 0x123456);
    }

    #[test]
    fn bounded_reads_reject_oversized_and_non_utf8_files() {
        let files = TestFiles::new();
        let path = files.0.join("input.html");
        std::fs::write(&path, vec![b' '; 1_048_576]).unwrap();
        assert_eq!(read_source(&path).unwrap().len(), 1_048_576);
        std::fs::write(&path, vec![b' '; 1_048_577]).unwrap();
        assert!(read_source(&path).unwrap_err().contains("exceeds 1 MiB"));
        std::fs::write(&path, [0xff]).unwrap();
        assert!(read_source(&path).is_err());
    }

    #[test]
    fn cli_and_builder_share_the_same_validated_window_configuration() {
        let source = parse(&[
            "page.html",
            "--title",
            "My App",
            "--width",
            "960",
            "--height",
            "640",
            "--min-width",
            "320",
            "--min-height",
            "240",
            "--background",
            "#234567",
            "--fixed-size",
            "--watch",
        ])
        .unwrap()
        .unwrap();
        let app = crate::DesktopApp::new("page.html")
            .title("My App")
            .size(960, 640)
            .min_size(320, 240)
            .background(Color::from_css_hex("#234567").unwrap())
            .resizable(false)
            .watch(true);
        assert_eq!(&source.window, app.options());
        source.window.validate().unwrap();
        assert!(source.watch);
        let literal = parse(&["--", "--title"]).unwrap().unwrap();
        assert_eq!(literal.html, Some(PathBuf::from("--title")));
        assert_eq!(literal.window, WindowOptions::default());
    }

    #[test]
    fn cli_rejects_invalid_or_duplicate_window_settings() {
        for args in [
            vec!["--title", ""],
            vec!["--title", "a\nb"],
            vec!["--width", "-1"],
            vec!["--height", "16385"],
            vec!["--width", "8192", "--height", "4096"],
            vec!["--min-width", "1000"],
            vec!["--background", "#0000"],
            vec!["--background", "red"],
            vec!["--width", "800", "--width", "960"],
            vec!["--title"],
            vec!["--fixed-size", "--fixed-size"],
        ] {
            assert!(parse(&args).is_err(), "{args:?}");
        }
    }

    #[test]
    fn window_configuration_and_backdrop_survive_resize_reload_and_drop() {
        let files = TestFiles::new();
        let html = files.0.join("page.html");
        let other = files.0.join("other.html");
        std::fs::write(&html, "<main></main>").unwrap();
        std::fs::write(&other, "<p></p>").unwrap();
        let background = Color::from_css_hex("#234567").unwrap();
        let settings = WindowOptions {
            title: "My App".into(),
            background,
            resizable: false,
            ..WindowOptions::default()
        };
        let mut app = app(Sources {
            html: Some(html.clone()),
            window: settings.clone(),
            ..Sources::default()
        });
        assert_eq!(app.frame.pixel(19, 19), Some(background));
        app.resize_page((40, 30));
        assert_eq!(app.frame.pixel(39, 29), Some(background));
        app.reload();
        #[cfg(feature = "javascript")]
        {
            app.pressed = Some((5, 5));
        }
        app.open_sources(app.sources.dropped(other).unwrap(), true)
            .unwrap();
        assert_eq!(app.frame.pixel(39, 29), Some(background));
        assert_eq!(app.sources.window, settings);
        #[cfg(feature = "javascript")]
        assert!(app.pressed.is_none());
        std::fs::write(app.sources.html.as_ref().unwrap(), "<div>".repeat(300)).unwrap();
        app.reload();
        assert_eq!(app.frame.pixel(39, 29), Some(background));
        assert_eq!(app.sources.window, settings);
        assert_eq!(
            display_pixel_on(&app.frame, 100, 100, 0, background),
            0x234567
        );
    }

    #[test]
    fn file_arguments_reject_ambiguity_and_support_dash_paths() {
        assert!(parse(&["--css", "x.css"]).is_err());
        assert!(parse(&["x.html", "y.html"]).is_err());
        assert!(parse(&["x.html", "--css"]).is_err());
        assert!(parse(&["x.html", "--css", "a", "--css", "b"]).is_err());
        assert!(parse(&["--wat"]).is_err());
        assert!(parse(&["--watch", "--watch"]).is_err());
        assert!(parse(&["x.html", "--watch"]).unwrap().unwrap().watch);
        assert!(parse(&["--help"]).unwrap().is_none());
        let sources = parse(&["--", "-page.html"]).unwrap().unwrap();
        assert_eq!(sources.html.unwrap(), PathBuf::from("-page.html"));
    }
}
