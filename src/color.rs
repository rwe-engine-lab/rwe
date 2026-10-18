use std::{error::Error, fmt};

/// A straight-alpha, eight-bit RGBA color.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Color {
    /// Red channel.
    pub r: u8,
    /// Green channel.
    pub g: u8,
    /// Blue channel.
    pub b: u8,
    /// Alpha: zero is transparent and 255 is opaque.
    pub a: u8,
}

impl Color {
    /// Transparent black.
    pub const TRANSPARENT: Self = Self::rgba(0, 0, 0, 0);
    /// Opaque white.
    pub const WHITE: Self = Self::rgba(255, 255, 255, 255);
    /// Opaque black.
    pub const BLACK: Self = Self::rgba(0, 0, 0, 255);

    /// Creates a color from straight RGBA channels.
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Parses exactly one CSS hex color: `#RGB`, `#RGBA`, `#RRGGBB`, or `#RRGGBBAA`.
    ///
    /// Both ASCII letter cases are accepted. Whitespace, named colors, and
    /// functional notation are outside this parser's scope and are rejected.
    pub fn from_css_hex(value: &str) -> Result<Self, ParseColorError> {
        let digits = value.strip_prefix('#').ok_or(ParseColorError)?;
        if !matches!(digits.len(), 3 | 4 | 6 | 8) {
            return Err(ParseColorError);
        }
        let mut nibbles = [0_u8; 8];
        for (i, byte) in digits.bytes().enumerate() {
            nibbles[i] = match byte {
                b'0'..=b'9' => byte - b'0',
                b'a'..=b'f' => byte - b'a' + 10,
                b'A'..=b'F' => byte - b'A' + 10,
                _ => return Err(ParseColorError),
            };
        }
        let channel = |i: usize| nibbles[i] * 16 + nibbles[i + 1];
        Ok(match digits.len() {
            3 => Self::rgba(nibbles[0] * 17, nibbles[1] * 17, nibbles[2] * 17, 255),
            4 => Self::rgba(
                nibbles[0] * 17,
                nibbles[1] * 17,
                nibbles[2] * 17,
                nibbles[3] * 17,
            ),
            6 => Self::rgba(channel(0), channel(2), channel(4), 255),
            8 => Self::rgba(channel(0), channel(2), channel(4), channel(6)),
            _ => unreachable!(),
        })
    }

    /// Composites this source over `destination`, rounding to eight-bit channels.
    ///
    /// Arithmetic is performed directly on stored channel values. This is not
    /// a color-managed or linear-light compositor.
    pub fn over(self, destination: Self) -> Self {
        if self.a == 0 {
            return destination;
        }
        let sa = u32::from(self.a);
        let da = u32::from(destination.a);
        let inverse = 255 - sa;
        let alpha = sa * 255 + da * inverse;
        let blend = |source: u8, dest: u8| {
            let numerator = u32::from(source) * sa * 255 + u32::from(dest) * da * inverse;
            ((numerator + alpha / 2) / alpha) as u8
        };
        Self::rgba(
            blend(self.r, destination.r),
            blend(self.g, destination.g),
            blend(self.b, destination.b),
            ((alpha + 127) / 255) as u8,
        )
    }
}

/// Input is not a supported CSS hex color.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseColorError;

impl fmt::Display for ParseColorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("expected #RGB, #RGBA, #RRGGBB, or #RRGGBBAA with ASCII hex digits")
    }
}

impl Error for ParseColorError {}
