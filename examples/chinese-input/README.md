# Chinese input and display

```sh
cargo run --features window,javascript,fonts --example chinese-input
```

`main.rs` only embeds application resources and starts `DesktopApp`. `app.js`
handles greeting/reset/status behavior. Font loading, glyph coverage/rasterization,
text metrics, input/selection painting and IME events stay in the framework.

The optional `fonts` feature embeds an OFL-licensed, renamed regular-weight Noto
Sans SC subset. It uses reviewed Rust rasterization without system font discovery
or a WebView. ASCII keeps the original bitmap glyphs; supported non-ASCII fallback
characters use real advances in text wrapping, input selection and candidate
positioning. The default build remains dependency-free and keeps its bitmap output.

Click or Tab into the field; use a system input method. Preedit is underlined and
the greeting changes only after commit. Rust API/headless tests verify Chinese
commits through real Vue/React apps; actual system candidate windows have not yet
been manually tested. Clipboard, reconversion, full shaping, emoji and all Unicode
coverage are not provided. See `src/assets/fonts/README.md` and APPLICATIONS.md.

Package a movable unsigned macOS app, including font license notices:

```sh
sh scripts/bundle-embedded-macos.sh target/RWE-Chinese-Input.app chinese-input
```

Runtime does not need the source directory, system fonts or a network connection.
FontTools/Python are only needed to regenerate the reviewed font, not to build/run.
