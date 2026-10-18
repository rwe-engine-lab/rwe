//! Software-painting foundations for an independent Rust Web engine.
//!
//! Implements CSS hex colors, a clipped RGBA canvas, and an html5ever-backed flat DOM arena.
//! Includes author stylesheets, block rendering and bitmap text.
//! Optional JavaScript execution uses an experimental DOM/event host.
//! Mature libraries provide HTML/CSS syntax, flex solving and PNG codecs.
//! RWE source forbids unsafe code.
//! The optional macOS window binary uses platform dependencies (see WINDOW-DEPENDENCIES.md).
//!
//! ```
//! use rwe::{Canvas, Color, Rect};
//!
//! let mut canvas = Canvas::new(16, 16)?;
//! canvas.clear(Color::from_css_hex("#fff")?);
//! canvas.fill_rect(Rect::new(-2, 2, 8, 4), Color::from_css_hex("#f008")?);
//! assert_eq!(canvas.pixel(15, 15), Some(Color::WHITE));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod boxmodel;
mod canvas;
mod color;
mod css;
mod dom;
mod flex;
mod html;
mod layout;
mod png;
mod stylesheet;
mod text;

pub use canvas::{Canvas, CanvasError, Rect};
pub use color::{Color, ParseColorError};

pub use dom::{Document, HtmlError, Node, NodeId, NodeKind, ParseLimits};

pub use flex::{AlignItems, JustifyContent};
pub use layout::{BlockLayout, BoxStyle, LayoutBox, RenderError};

pub use stylesheet::Stylesheet;

pub use text::{TextAlign, TextRun, WhiteSpace};

pub use boxmodel::{BorderStyle, Edges, Overflow};

mod local_page;
pub use local_page::{LocalPage, PageScript, read_local_source};

#[cfg(feature = "javascript")]
mod runtime;
#[cfg(feature = "javascript")]
pub use runtime::ScriptRuntime;

mod application;
pub use application::{Application, RenderOptions};

#[cfg(feature = "window")]
mod desktop;
#[cfg(all(feature = "window", target_os = "macos"))]
mod window_host;
#[cfg(feature = "window")]
pub use desktop::{DesktopApp, WindowOptions};

mod asset_bundle;
pub use asset_bundle::AssetBundle;

#[cfg(feature = "javascript")]
mod keyboard;
#[cfg(feature = "javascript")]
pub use keyboard::KeyEvent;

mod form_control;

#[cfg(feature = "javascript")]
mod ime;
#[cfg(feature = "javascript")]
pub use ime::ImeEvent;

#[cfg(feature = "fonts")]
mod font;
