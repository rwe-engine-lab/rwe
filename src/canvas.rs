use crate::Color;
use std::{
    error::Error,
    fmt,
    io::{self, Write},
};

/// A rectangle in integer pixel coordinates; right and bottom edges are exclusive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    /// Left edge, which may lie outside the canvas.
    pub x: i32,
    /// Top edge, which may lie outside the canvas.
    pub y: i32,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

impl Rect {
    /// Creates a rectangle. A zero width or height paints nothing.
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
    /// Returns the nonempty intersection, using widened arithmetic for edges.
    /// Touching edges or an empty operand return `None`.
    pub fn intersection(self, other: Self) -> Option<Self> {
        let left = self.x.max(other.x);
        let top = self.y.max(other.y);
        let right = (i64::from(self.x) + i64::from(self.width))
            .min(i64::from(other.x) + i64::from(other.width));
        let bottom = (i64::from(self.y) + i64::from(self.height))
            .min(i64::from(other.y) + i64::from(other.height));
        if right <= i64::from(left) || bottom <= i64::from(top) {
            return None;
        }
        Some(Self::new(
            left,
            top,
            (right - i64::from(left)) as u32,
            (bottom - i64::from(top)) as u32,
        ))
    }
}

/// An owned, row-major, straight-alpha RGBA software canvas.
pub struct Canvas {
    width: u32,
    height: u32,
    pixels: Vec<Color>,
}

impl Canvas {
    /// Creates a transparent canvas, returning an error for overflow or allocation failure.
    ///
    /// Zero-sized canvases are valid. Untrusted callers should also impose their
    /// own application-level size limits before requesting allocation.
    pub fn new(width: u32, height: u32) -> Result<Self, CanvasError> {
        let count = u64::from(width) * u64::from(height);
        let count = usize::try_from(count).map_err(|_| CanvasError)?;
        let mut pixels = Vec::new();
        pixels.try_reserve_exact(count).map_err(|_| CanvasError)?;
        pixels.resize(count, Color::TRANSPARENT);
        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    /// Width in pixels.
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Height in pixels.
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Returns the row-major pixel slice.
    pub fn pixels(&self) -> &[Color] {
        &self.pixels
    }

    /// Reads a pixel, or returns `None` outside the canvas.
    pub fn pixel(&self, x: u32, y: u32) -> Option<Color> {
        if x >= self.width || y >= self.height {
            return None;
        }
        Some(self.pixels[y as usize * self.width as usize + x as usize])
    }

    /// Replaces every pixel without alpha blending.
    pub fn clear(&mut self, color: Color) {
        self.pixels.fill(color);
    }

    /// Writes a binary RGB PPM (P6), compositing pixels over an opaque background.
    ///
    /// Returns `InvalidInput` for a non-opaque background or zero-sized canvas.
    /// Does not flush the writer; the caller controls buffering and flush errors.
    ///
    /// ```
    /// use rwe::{Canvas, Color};
    /// let canvas = Canvas::new(1, 1)?;
    /// let mut bytes = Vec::new();
    /// canvas.write_ppm(&mut bytes, Color::WHITE)?;
    /// assert_eq!(bytes, b"P6\n1 1\n255\n\xff\xff\xff");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn write_ppm(&self, writer: &mut impl Write, background: Color) -> io::Result<()> {
        if background.a != 255 || self.width == 0 || self.height == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "PPM requires nonzero dimensions and an opaque background",
            ));
        }
        writeln!(writer, "P6\n{} {}\n255", self.width, self.height)?;
        for pixel in &self.pixels {
            let pixel = pixel.over(background);
            writer.write_all(&[pixel.r, pixel.g, pixel.b])?;
        }
        Ok(())
    }

    /// Paints through an additional rectangle, also clipped to canvas bounds.
    /// The clip is local to this call and cannot affect later painting.
    pub fn fill_rect_clipped(&mut self, rect: Rect, color: Color, clip: Rect) {
        if let Some(rect) = rect.intersection(clip) {
            self.fill_rect(rect, color);
        }
    }

    /// Paints a clipped rectangle using source-over compositing.
    ///
    /// Coordinates are widened before addition so extreme input cannot overflow.
    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        let left = i64::from(rect.x).clamp(0, i64::from(self.width));
        let top = i64::from(rect.y).clamp(0, i64::from(self.height));
        let right = (i64::from(rect.x) + i64::from(rect.width)).clamp(0, i64::from(self.width));
        let bottom = (i64::from(rect.y) + i64::from(rect.height)).clamp(0, i64::from(self.height));
        for y in top..bottom {
            let row = y as usize * self.width as usize;
            for x in left..right {
                let pixel = &mut self.pixels[row + x as usize];
                *pixel = color.over(*pixel);
            }
        }
    }
}

/// Canvas dimensions overflow addressable storage or allocation cannot be reserved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CanvasError;

impl fmt::Display for CanvasError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("canvas dimensions exceed available addressable storage")
    }
}

impl Error for CanvasError {}
