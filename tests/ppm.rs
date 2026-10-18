//! PPM encoding flattens transparency explicitly and propagates writer errors.
use rwe::{Canvas, Color, Rect};
use std::io::{self, Write};

#[test]
fn encodes_rgb_in_row_order_and_flattens_alpha_over_opaque_background() {
    let mut canvas = Canvas::new(2, 1).unwrap();
    let red = Color::rgba(255, 0, 0, 128);
    canvas.fill_rect(Rect::new(0, 0, 1, 1), red);
    let mut bytes = vec![];
    canvas.write_ppm(&mut bytes, Color::WHITE).unwrap();
    let expected = red.over(Color::WHITE);
    assert_eq!(&bytes[..11], b"P6\n2 1\n255\n");
    assert_eq!(
        &bytes[11..],
        &[expected.r, expected.g, expected.b, 255, 255, 255]
    );
}

#[test]
fn invalid_export_parameters_do_not_write_a_partial_header() {
    for (canvas, background) in [
        (Canvas::new(1, 1).unwrap(), Color::TRANSPARENT),
        (Canvas::new(0, 1).unwrap(), Color::WHITE),
        (Canvas::new(1, 0).unwrap(), Color::WHITE),
    ] {
        let mut bytes = vec![];
        assert_eq!(
            canvas.write_ppm(&mut bytes, background).unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
        assert!(bytes.is_empty());
    }
}

struct Failing;
impl Write for Failing {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::ErrorKind::BrokenPipe.into())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
#[test]
fn export_propagates_output_errors() {
    let canvas = Canvas::new(1, 1).unwrap();
    assert_eq!(
        canvas
            .write_ppm(&mut Failing, Color::WHITE)
            .unwrap_err()
            .kind(),
        io::ErrorKind::BrokenPipe
    );
}
