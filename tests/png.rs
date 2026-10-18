//! PNG pixel compatibility, alpha, streaming rows and writer errors.
#[path = "support/png.rs"]
mod png;
use rwe::{Canvas, Color, Rect};
use std::io::{self, Write};

#[test]
fn matches_independent_python_zlib_reference_and_preserves_hidden_rgb() {
    let mut canvas = Canvas::new(1, 1).unwrap();
    canvas.clear(Color::rgba(9, 8, 7, 0));
    let mut encoded = vec![];
    canvas.write_png(&mut encoded).unwrap();
    assert_eq!(
        png::decode(&encoded),
        png::decode(include_bytes!("fixtures/transparent.png"))
    );
    assert_eq!(png::decode(&encoded), (1, 1, vec![9, 8, 7, 0]));
}

#[test]
fn stores_row_major_rgba_without_alpha_flattening() {
    let mut canvas = Canvas::new(2, 2).unwrap();
    canvas.fill_rect(Rect::new(0, 0, 1, 1), Color::rgba(255, 0, 0, 128));
    canvas.fill_rect(Rect::new(1, 1, 1, 1), Color::rgba(0, 0, 255, 255));
    let mut bytes = vec![];
    canvas.write_png(&mut bytes).unwrap();
    assert_eq!(
        png::decode(&bytes),
        (
            2,
            2,
            vec![255, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255]
        )
    );
}

#[test]
fn streams_many_rows_with_valid_decodable_pixels() {
    // Exercise varied image heights without binding tests to DEFLATE choices.
    for height in [254, 255, 256, 510, 511] {
        let mut canvas = Canvas::new(64, height).unwrap();
        canvas.clear(Color::rgba(11, 22, 33, 44));
        let mut bytes = vec![];
        canvas.write_png(&mut bytes).unwrap();
        let (width, decoded_height, rgba) = png::decode(&bytes);
        assert_eq!((width, decoded_height), (64, height));
        assert!(rgba.chunks_exact(4).all(|pixel| pixel == [11, 22, 33, 44]));
    }
}

#[test]
fn rejects_zero_dimensions_before_writing() {
    for canvas in [Canvas::new(0, 1).unwrap(), Canvas::new(1, 0).unwrap()] {
        let mut bytes = vec![];
        assert_eq!(
            canvas.write_png(&mut bytes).unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
        assert!(bytes.is_empty());
    }
}
struct Limited {
    remaining: usize,
}
impl Write for Limited {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.remaining == 0 {
            return Err(io::ErrorKind::BrokenPipe.into());
        }
        let written = bytes.len().min(self.remaining).min(1);
        self.remaining -= written;
        Ok(written)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
#[test]
fn handles_short_writes_and_propagates_errors_at_header_data_and_trailer() {
    let canvas = Canvas::new(1, 1).unwrap();
    let mut bytes = vec![];
    canvas.write_png(&mut bytes).unwrap();
    for remaining in [0, 8, 40, bytes.len() - 1] {
        assert_eq!(
            canvas
                .write_png(&mut Limited { remaining })
                .unwrap_err()
                .kind(),
            io::ErrorKind::BrokenPipe
        );
    }
    canvas
        .write_png(&mut Limited {
            remaining: bytes.len(),
        })
        .unwrap();
}

#[test]
fn retries_interrupted_writes_without_reporting_a_spurious_failure() {
    struct InterruptOnce {
        interrupted: bool,
        data: Vec<u8>,
    }
    impl Write for InterruptOnce {
        fn write(&mut self, data: &[u8]) -> io::Result<usize> {
            if !self.interrupted {
                self.interrupted = true;
                return Err(io::ErrorKind::Interrupted.into());
            }
            self.data.extend_from_slice(data);
            Ok(data.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut output = InterruptOnce {
        interrupted: false,
        data: vec![],
    };
    Canvas::new(1, 1).unwrap().write_png(&mut output).unwrap();
    assert_eq!(png::decode(&output.data), (1, 1, vec![0, 0, 0, 0]));
}
