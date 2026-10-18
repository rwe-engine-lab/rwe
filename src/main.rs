//! A local-file command-line host for the strict rwe rendering subset.
use rwe::{Application, Color, RenderOptions};
use std::{
    collections::BTreeSet,
    env,
    ffi::OsString,
    fs::OpenOptions,
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

const MAX_DIMENSION: u32 = 16_384;
const MAX_PIXELS: u64 = 16_777_216;
const HELP: &str = "rwe - local HTML/CSS software renderer

Usage:
  rwe render INPUT.html [OPTIONS]
  rwe --help

Options:
  --css PATH          Read one author stylesheet (optional)
  --width PIXELS      Viewport width (default: 800)
  --height PIXELS     Canvas height, or auto (default: auto)
  --background HEX    CSS hex or transparent (default: #fff; PPM needs opaque)
  --format FORMAT     png or ppm; .png output selects PNG, otherwise PPM
  --output PATH       Write a new image file, or - for stdout (default: -)
  --                  Treat remaining arguments as input paths
  -h, --help          Show this help

Only the documented HTML/CSS/bitmap text subset is supported.
Inputs must be UTF-8, up to 1 MiB each. Local style/link CSS is loaded; no network or JavaScript execution.
Dimensions: 1..16384; at most 16777216 pixels. Auto height uses root flow,
excluding visible overflow, with a one-pixel minimum. Existing outputs are refused.
";

#[derive(Clone, Copy)]
enum Format {
    Ppm,
    Png,
}

struct Options {
    input: PathBuf,
    css: Option<PathBuf>,
    width: u32,
    height: Option<u32>,
    background: Color,
    output: PathBuf,
    format: Format,
}
enum Command {
    Help,
    Render(Options),
}

fn parse(mut args: impl Iterator<Item = OsString>) -> Result<Command, String> {
    let first = args.next();
    match first.as_deref() {
        None => return Ok(Command::Help),
        Some(arg) if arg == "--help" || arg == "-h" => return Ok(Command::Help),
        Some(arg) if arg == "render" => {}
        _ => return Err("expected 'render'; use --help for usage".into()),
    }
    let mut input = None;
    let mut css = None;
    let mut width = 800;
    let mut height = None;
    let mut background = Color::WHITE;
    let mut output = PathBuf::from("-");
    let mut format = None;
    let mut seen = BTreeSet::new();
    let mut positional = false;
    while let Some(arg) = args.next() {
        if !positional && arg == "--" {
            positional = true;
            continue;
        }
        if !positional && (arg == "--help" || arg == "-h") {
            return Ok(Command::Help);
        }
        if !positional && arg.to_str().is_some_and(|arg| arg.starts_with('-')) {
            let flag = arg.to_str().unwrap();
            if !matches!(
                flag,
                "--css" | "--width" | "--height" | "--background" | "--output" | "--format"
            ) {
                return Err(format!("unknown option: {flag}"));
            }
            if !seen.insert(flag.to_owned()) {
                return Err(format!("duplicate option: {flag}"));
            }
            let value = args
                .next()
                .ok_or_else(|| format!("missing value for {flag}"))?;
            match flag {
                "--css" => css = Some(PathBuf::from(value)),
                "--output" => output = PathBuf::from(value),
                "--width" => width = dimension(&value, flag)?,
                "--height" => {
                    height = if value == "auto" {
                        None
                    } else {
                        Some(dimension(&value, flag)?)
                    }
                }
                "--background" => {
                    let value = value.to_str().ok_or("background must be UTF-8")?;
                    background = if value.eq_ignore_ascii_case("transparent") {
                        Color::TRANSPARENT
                    } else {
                        Color::from_css_hex(value)
                            .map_err(|_| "background must be a CSS hex color or transparent")?
                    };
                }
                "--format" => {
                    format = Some(match value.to_str() {
                        Some("png") => Format::Png,
                        Some("ppm") => Format::Ppm,
                        _ => return Err("format must be png or ppm".into()),
                    })
                }
                _ => unreachable!(),
            }
        } else if input.replace(PathBuf::from(arg)).is_some() {
            return Err("expected exactly one HTML input path".into());
        }
    }
    let format = match format {
        Some(format) => format,
        None if output == Path::new("-") => Format::Ppm,
        None => match output
            .extension()
            .and_then(|extension| extension.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("png") => Format::Png,
            Some("ppm") => Format::Ppm,
            _ => Format::Ppm,
        },
    };
    if matches!(format, Format::Ppm) && background.a != 255 {
        return Err("PPM background must be opaque; use --format png to preserve alpha".into());
    }
    if let Some(height) = height {
        check_pixels(width, height)?;
    }
    Ok(Command::Render(Options {
        input: input.ok_or("missing HTML input path")?,
        css,
        width,
        height,
        background,
        output,
        format,
    }))
}
fn dimension(value: &std::ffi::OsStr, flag: &str) -> Result<u32, String> {
    let value = value
        .to_str()
        .ok_or_else(|| format!("{flag} requires an integer"))?;
    let size = if !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()) {
        value.parse::<u32>().ok()
    } else {
        None
    };
    size.filter(|&size| (1..=MAX_DIMENSION).contains(&size))
        .ok_or_else(|| format!("{flag} must be an integer from 1 to {MAX_DIMENSION}"))
}
fn check_pixels(width: u32, height: u32) -> Result<(), String> {
    if height > MAX_DIMENSION || u64::from(width) * u64::from(height) > MAX_PIXELS {
        return Err(
            "canvas exceeds dimension or pixel limit; choose a smaller width/height".into(),
        );
    }
    Ok(())
}
fn render(options: Options) -> Result<(), String> {
    let app = Application::load_static(&options.input, options.css.as_deref())?;
    let canvas = app.render(RenderOptions {
        width: options.width,
        height: options.height,
        background: options.background,
        ..RenderOptions::default()
    })?;
    let write = |writer: &mut BufWriter<Box<dyn Write>>| -> io::Result<()> {
        match options.format {
            Format::Ppm => canvas.write_ppm(writer, options.background)?,
            Format::Png => canvas.write_png(writer)?,
        }
        writer.flush()
    };
    if options.output == Path::new("-") {
        write(&mut BufWriter::new(Box::new(io::stdout())))
            .map_err(|error| format!("cannot write stdout: {error}"))?;
    } else {
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&options.output)
            .map_err(|error| format!("cannot create {}: {error}", options.output.display()))?;
        write(&mut BufWriter::new(Box::new(file)))
            .map_err(|error| format!("cannot write {}: {error}", options.output.display()))?;
    }
    Ok(())
}
fn main() -> ExitCode {
    let result = parse(env::args_os().skip(1)).and_then(|command| match command {
        Command::Help => io::stdout()
            .write_all(HELP.as_bytes())
            .map_err(|error| error.to_string()),
        Command::Render(options) => render(options),
    });
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("rwe: {error}");
            ExitCode::FAILURE
        }
    }
}
