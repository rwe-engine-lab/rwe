//! PNG export delegated to image-rs/png; RWE only adapts its straight RGBA canvas.
use crate::Canvas;
use std::io::{self, Write};
fn png_error(error: png::EncodingError) -> io::Error {
    match error {
        png::EncodingError::IoError(error) => error,
        other => io::Error::other(other),
    }
}
// png's streaming API may wrap IO errors into ErrorKind::Other. Preserve the
// caller's original error at the adapter boundary, including its payload.
struct Output<'a, W> {
    writer: &'a mut W,
    error: &'a mut Option<io::Error>,
}
impl<W: Write> Write for Output<'_, W> {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        match self.writer.write(data) {
            Ok(size) => Ok(size),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => Err(error),
            Err(error) => {
                let kind = error.kind();
                if self.error.is_none() {
                    *self.error = Some(error);
                }
                Err(kind.into())
            }
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        match self.writer.flush() {
            Ok(()) => Ok(()),
            Err(error) => {
                let kind = error.kind();
                if self.error.is_none() {
                    *self.error = Some(error);
                }
                Err(kind.into())
            }
        }
    }
}
impl Canvas {
    /// Writes an eight-bit RGBA PNG, preserving straight alpha and hidden RGB.
    ///
    /// Compression, filtering, checksums and chunks are handled by image-rs/png.
    /// A single RGBA row is buffered by this adapter. Zero dimensions are rejected
    /// before output. Writer error kinds propagate; the caller owns final flushing.
    ///
    /// ```
    /// use rwe::Canvas;
    /// let mut bytes = Vec::new();
    /// Canvas::new(1, 1)?.write_png(&mut bytes)?;
    /// assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn write_png(&self, writer: &mut impl Write) -> io::Result<()> {
        if self.width() == 0
            || self.height() == 0
            || self.width() > i32::MAX as u32
            || self.height() > i32::MAX as u32
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "PNG dimensions must be from 1 to i32::MAX",
            ));
        }
        let mut output_error = None;
        let result = (|| {
            let mut encoder = png::Encoder::new(
                Output {
                    writer,
                    error: &mut output_error,
                },
                self.width(),
                self.height(),
            );
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut image = encoder.write_header().map_err(png_error)?;
            {
                let mut stream = image.stream_writer().map_err(png_error)?;
                let mut bytes = Vec::with_capacity(self.width() as usize * 4);
                for row in self.pixels().chunks_exact(self.width() as usize) {
                    bytes.clear();
                    for pixel in row {
                        bytes.extend_from_slice(&[pixel.r, pixel.g, pixel.b, pixel.a]);
                    }
                    stream.write_all(&bytes)?;
                }
                stream.finish().map_err(png_error)?;
            }
            image.finish().map_err(png_error)
        })();
        match output_error {
            Some(error) => Err(error),
            None => result,
        }
    }
}
