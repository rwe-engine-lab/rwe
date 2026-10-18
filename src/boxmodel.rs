//! Nonnegative physical box edges and the supported border style.
use crate::RenderError;

/// Physical pixel edges in CSS top/right/bottom/left order.
///
/// ```
/// use rwe::{BoxStyle, Edges};
/// let style = BoxStyle::parse("padding: 1px 2px 3px 4px; margin: 8px")?;
/// assert_eq!(style.padding.left, 4);
/// assert_eq!(style.padding.bottom, 3);
/// assert_eq!(style.margin, Edges::all(8));
/// # Ok::<(), rwe::RenderError>(())
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Edges {
    /// Top edge.
    pub top: u32,
    /// Right edge.
    pub right: u32,
    /// Bottom edge.
    pub bottom: u32,
    /// Left edge.
    pub left: u32,
}
impl Edges {
    /// Creates four equal edges.
    pub const fn all(value: u32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }
    pub(crate) fn parse(source: &str) -> Result<Self, RenderError> {
        let values = source
            .split_ascii_whitespace()
            .map(crate::layout::length)
            .collect::<Result<Vec<_>, _>>()?;
        match values.as_slice() {
            [a] => Ok(Self::all(*a)),
            [a, b] => Ok(Self {
                top: *a,
                right: *b,
                bottom: *a,
                left: *b,
            }),
            [a, b, c] => Ok(Self {
                top: *a,
                right: *b,
                bottom: *c,
                left: *b,
            }),
            [a, b, c, d] => Ok(Self {
                top: *a,
                right: *b,
                bottom: *c,
                left: *d,
            }),
            _ => Err(RenderError("expected one to four pixel lengths")),
        }
    }
}

/// The currently supported uniform border styles.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BorderStyle {
    /// No border; the specified border width does not occupy layout space.
    #[default]
    None,
    /// A solid rectangular border.
    Solid,
}
impl BorderStyle {
    pub(crate) fn parse(value: &str) -> Result<Self, RenderError> {
        match value.to_ascii_lowercase().as_str() {
            "none" => Ok(Self::None),
            "solid" => Ok(Self::Solid),
            _ => Err(RenderError("border-style must be none or solid")),
        }
    }
}

pub(crate) fn border_color(value: &str) -> Result<Option<crate::Color>, RenderError> {
    if value.eq_ignore_ascii_case("currentcolor") {
        Ok(None)
    } else if value.eq_ignore_ascii_case("transparent") {
        Ok(Some(crate::Color::TRANSPARENT))
    } else {
        crate::Color::from_css_hex(value)
            .map(Some)
            .map_err(|_| RenderError("unsupported border color"))
    }
}

/// Whether descendants can paint outside the element's padding box.
///
/// ```
/// use rwe::{BlockLayout, Document, Rect};
/// let document = Document::parse(
///     "<p style='width: 20px; height: 8px; padding: 2px; border: 1px solid #000; overflow: hidden'>TEXT</p>"
/// )?;
/// let layout = BlockLayout::new(&document, 100)?;
/// assert_eq!(layout.boxes()[1].clip_rect, Some(Rect::new(1, 1, 24, 12)));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Overflow {
    /// Descendants may overflow, subject to ancestor and canvas clips.
    #[default]
    Visible,
    /// Clip descendant backgrounds, borders, and text to the padding box.
    Hidden,
}
