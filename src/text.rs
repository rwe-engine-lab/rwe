//! Bitmap glyphs, optional TrueType fallback, and shared pixel text metrics.
use crate::{Canvas, Color, NodeId, Rect, RenderError};

/// Horizontal alignment of each text line within the containing content box.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TextAlign {
    /// Align at the left edge.
    #[default]
    Left,
    /// Center using integer pixels, rounding the left offset down.
    Center,
    /// Align at the right edge.
    Right,
}

/// Whitespace processing for the supported text subset.
///
/// ```
/// use rwe::{BlockLayout, Document};
/// let document = Document::parse(
///     "<p style='white-space: pre; font-size: 8px; text-align: right'>A\nB</p>"
/// )?;
/// let layout = BlockLayout::new(&document, 30)?;
/// assert_eq!(layout.text_runs().len(), 2);
/// assert_eq!(layout.text_runs()[0].rect.x, 24);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum WhiteSpace {
    /// Collapse ASCII whitespace and wrap at word boundaries, splitting long words.
    #[default]
    Normal,
    /// Preserve spaces and line breaks, expand tabs at four-column stops, and do not wrap.
    Pre,
}

/// A single line of bitmap text in paint order.
///
/// ```
/// use rwe::{BlockLayout, Document};
/// let document = Document::parse("<p style='font-size: 8px'>ABCD</p>")?;
/// let layout = BlockLayout::new(&document, 12)?;
/// assert_eq!(layout.text_runs().len(), 2);
/// assert_eq!(layout.height(), 16);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextRun {
    /// Source DOM node (a text node or a single-line input control).
    pub node: NodeId,
    /// Line cell bounds, including the advance after the final character.
    pub rect: Rect,
    /// Text after whitespace processing, without newline characters.
    pub text: String,
    /// Resolved inherited foreground color.
    pub color: Color,
    /// Cell height, an integer multiple of eight pixels.
    pub font_size: u32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TextStyle {
    pub color: Color,
    pub size: u32,
    pub align: TextAlign,
    pub white_space: WhiteSpace,
}
impl Default for TextStyle {
    fn default() -> Self {
        Self {
            color: Color::BLACK,
            size: 16,
            align: TextAlign::Left,
            white_space: WhiteSpace::Normal,
        }
    }
}

pub(crate) fn layout_text(
    node: NodeId,
    source: &str,
    x: u32,
    y: u32,
    width: u32,
    style: TextStyle,
) -> Result<Vec<TextRun>, RenderError> {
    let lines = match style.white_space {
        WhiteSpace::Normal => wrap_words(source, width, style.size)?,
        WhiteSpace::Pre => preserved_lines(source),
    };
    let mut runs = Vec::with_capacity(lines.len());
    let mut top = y;
    for text in lines {
        let line_width = measure(&text, style.size)?;
        let remaining = width.saturating_sub(line_width);
        let offset = match style.align {
            TextAlign::Left => 0,
            TextAlign::Center => remaining / 2,
            TextAlign::Right => remaining,
        };
        let left = crate::layout::add(x, offset)?;
        crate::layout::add(left, line_width)?;
        let bottom = crate::layout::add(top, style.size)?;
        runs.push(TextRun {
            node,
            rect: Rect::new(left as i32, top as i32, line_width, style.size),
            text,
            color: style.color,
            font_size: style.size,
        });
        top = bottom;
    }
    Ok(runs)
}

pub(crate) fn advance(ch: char, size: u32) -> u32 {
    #[cfg(feature = "fonts")]
    if let Some(width) = crate::font::advance(ch, size) {
        return width;
    }
    let _ = ch;
    6 * (size / 8)
}
pub(crate) fn measure(source: &str, size: u32) -> Result<u32, RenderError> {
    source.chars().try_fold(0u32, |width, ch| {
        width
            .checked_add(advance(ch, size))
            .ok_or(RenderError("text width overflow"))
    })
}
fn wrap_words(source: &str, width: u32, size: u32) -> Result<Vec<String>, RenderError> {
    let mut lines = vec![];
    let mut line = String::new();
    let mut used = 0u32;
    let space = advance(' ', size);
    for word in source.split_ascii_whitespace() {
        let word_width = measure(word, size)?;
        if !line.is_empty() {
            if word_width
                .checked_add(space)
                .is_some_and(|w| w <= width.saturating_sub(used))
            {
                line.push(' ');
                used += space;
            } else {
                lines.push(std::mem::take(&mut line));
                used = 0;
            }
        }
        for ch in word.chars() {
            let next = advance(ch, size);
            if !line.is_empty() && next > width.saturating_sub(used) {
                lines.push(std::mem::take(&mut line));
                used = 0;
            }
            line.push(ch);
            used = used
                .checked_add(next)
                .ok_or(RenderError("text width overflow"))?;
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    Ok(lines)
}

fn preserved_lines(source: &str) -> Vec<String> {
    let normalized = source.replace("\r\n", "\n").replace('\r', "\n");
    normalized
        .split_terminator('\n')
        .map(|line| {
            let mut text = String::new();
            let mut column = 0;
            for ch in line.chars() {
                if ch == '\t' {
                    let spaces = 4 - column % 4;
                    text.extend(std::iter::repeat_n(' ', spaces));
                    column += spaces;
                } else {
                    text.push(ch);
                    column += 1;
                }
            }
            text
        })
        .collect()
}

impl TextRun {
    pub(crate) fn paint(&self, canvas: &mut Canvas, clip: Rect) {
        let scale = self.font_size / 8;
        let mut pen = i64::from(self.rect.x);
        for ch in self.text.chars() {
            let x = pen;
            pen += i64::from(advance(ch, self.font_size));
            if x >= i64::from(clip.x) + i64::from(clip.width) {
                break;
            }
            if pen < i64::from(clip.x) {
                continue;
            }
            #[cfg(feature = "fonts")]
            if crate::font::paint(
                ch,
                self.font_size,
                x,
                i64::from(self.rect.y),
                self.color,
                canvas,
                clip,
            ) {
                continue;
            }
            for (row, bits) in glyph(ch).into_iter().enumerate() {
                for bit in 0..5 {
                    if bits & (1 << (4 - bit)) != 0 {
                        let x = x + i64::from(bit * scale);
                        let y = i64::from(self.rect.y) + row as i64 * i64::from(scale);
                        canvas.fill_rect_clipped(
                            Rect::new(x as i32, y as i32, scale, scale),
                            self.color,
                            clip,
                        );
                    }
                }
            }
        }
    }
}

// These original 5-by-7 patterns intentionally share uppercase/lowercase shapes.
fn glyph(ch: char) -> [u8; 7] {
    match ch.to_ascii_uppercase() {
        'A' => [14, 17, 17, 31, 17, 17, 17],
        'B' => [30, 17, 17, 30, 17, 17, 30],
        'C' => [14, 17, 16, 16, 16, 17, 14],
        'D' => [30, 17, 17, 17, 17, 17, 30],
        'E' => [31, 16, 16, 30, 16, 16, 31],
        'F' => [31, 16, 16, 30, 16, 16, 16],
        'G' => [14, 17, 16, 23, 17, 17, 14],
        'H' => [17, 17, 17, 31, 17, 17, 17],
        'I' => [14, 4, 4, 4, 4, 4, 14],
        'J' => [7, 2, 2, 2, 18, 18, 12],
        'K' => [17, 18, 20, 24, 20, 18, 17],
        'L' => [16, 16, 16, 16, 16, 16, 31],
        'M' => [17, 27, 21, 21, 17, 17, 17],
        'N' => [17, 25, 21, 19, 17, 17, 17],
        'O' => [14, 17, 17, 17, 17, 17, 14],
        'P' => [30, 17, 17, 30, 16, 16, 16],
        'Q' => [14, 17, 17, 17, 21, 18, 13],
        'R' => [30, 17, 17, 30, 20, 18, 17],
        'S' => [15, 16, 16, 14, 1, 1, 30],
        'T' => [31, 4, 4, 4, 4, 4, 4],
        'U' => [17, 17, 17, 17, 17, 17, 14],
        'V' => [17, 17, 17, 17, 17, 10, 4],
        'W' => [17, 17, 17, 21, 21, 21, 10],
        'X' => [17, 17, 10, 4, 10, 17, 17],
        'Y' => [17, 17, 10, 4, 4, 4, 4],
        'Z' => [31, 1, 2, 4, 8, 16, 31],
        '0' => [14, 17, 19, 21, 25, 17, 14],
        '1' => [4, 12, 4, 4, 4, 4, 14],
        '2' => [14, 17, 1, 2, 4, 8, 31],
        '3' => [30, 1, 1, 14, 1, 1, 30],
        '4' => [2, 6, 10, 18, 31, 2, 2],
        '5' => [31, 16, 16, 30, 1, 1, 30],
        '6' => [14, 16, 16, 30, 17, 17, 14],
        '7' => [31, 1, 2, 4, 8, 8, 8],
        '8' => [14, 17, 17, 14, 17, 17, 14],
        '9' => [14, 17, 17, 15, 1, 1, 14],
        ' ' | '\u{a0}' => [0; 7],
        '.' => [0, 0, 0, 0, 0, 12, 12],
        ',' => [0, 0, 0, 0, 0, 4, 8],
        ':' => [0, 4, 4, 0, 4, 4, 0],
        ';' => [0, 4, 4, 0, 4, 4, 8],
        '!' => [4, 4, 4, 4, 4, 0, 4],
        '?' => [14, 17, 1, 2, 4, 0, 4],
        '-' => [0, 0, 0, 31, 0, 0, 0],
        '_' => [0, 0, 0, 0, 0, 0, 31],
        '+' => [0, 4, 4, 31, 4, 4, 0],
        '=' => [0, 0, 31, 0, 31, 0, 0],
        '/' => [1, 2, 2, 4, 8, 8, 16],
        '(' => [2, 4, 8, 8, 8, 4, 2],
        ')' => [8, 4, 2, 2, 2, 4, 8],
        '[' => [14, 8, 8, 8, 8, 8, 14],
        ']' => [14, 2, 2, 2, 2, 2, 14],
        '<' => [1, 2, 4, 8, 4, 2, 1],
        '>' => [16, 8, 4, 2, 4, 8, 16],
        '\'' => [4, 4, 8, 0, 0, 0, 0],
        '"' => [10, 10, 0, 0, 0, 0, 0],
        '#' => [10, 31, 10, 10, 31, 10, 0],
        '*' => [0, 21, 14, 31, 14, 21, 0],
        '&' => [12, 18, 20, 8, 21, 18, 13],
        '%' => [25, 25, 2, 4, 8, 19, 19],
        _ => [31, 17, 21, 21, 21, 17, 31],
    }
}
