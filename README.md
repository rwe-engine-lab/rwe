> Static HTML and experimental interactive Vue/React support: see [APPLICATIONS.md](APPLICATIONS.md). JavaScript builds need Rust 1.91+. Arbitrary browser apps are not yet compatible.

# rwe

An experimental pure Rust application framework with its own HTML/CSS renderer and optional JavaScript/native desktop hosts.

This initial release provides working software-painting primitives. The development tree also supports HTML5 parsing through html5ever, author stylesheets, and block and bitmap text rendering. It is **not yet a browser, full HTML/CSS renderer, fully compatible JavaScript/DOM host, or Electron replacement**. The project is at an early development stage; APIs may change before 1.0.

[中文说明](README.zh-CN.md)

## Implemented

- CSS hex color parsing: `#RGB`, `#RGBA`, `#RRGGBB`, `#RRGGBBAA`.
- Eight-bit straight-alpha RGBA source-over compositing.
- An owned software canvas with clipped rectangle painting and checked allocation.
- A local-file rendering CLI with native PNG/PPM export, image examples, and regression tests for parsing, layout, and pixels.
- Mature reusable components: html5ever/cssparser for syntax, Taffy for exposed Flex layout, image-rs/png for PNG, and optional Boa/fontdue/winit/softbuffer. RWE source forbids unsafe code. See [reuse audit](REUSE-AUDIT.md).

Desktop applications use the `window,javascript` features:

```rust
fn main() -> Result<(), String> {
    rwe::DesktopApp::new("index.html").watch(true).run()
}
```

## Run locally

```sh
cargo test --all-targets
cargo test --doc
cargo run -- render examples/static-html/index.html --width 480 --format png > preview.png
cargo run --features window --bin rwe-window -- examples/static-html/index.html --watch
cargo run --features window,javascript --bin rwe-window -- examples/vue-counter/index.html --watch
cargo run --features window,javascript --bin rwe-window -- examples/react-counter/index.html --watch
```

The rendering CLI writes a binary PNG image; redirect stdout to a file. The canvas stores straight RGBA values, not a guaranteed packed byte layout. Compositing works directly on stored channels, without linear-light conversion or color management. Input is a single hex color token; whitespace, named colors, and functional CSS colors are rejected. CSS syntax uses cssparser; supported properties, selectors and cascade remain limited.

## Render local files from the command line (unreleased)

```sh
cargo run -- render examples/static-html/index.html --css examples/static-html/style.css --width 480 --output page.png
cargo run -- render examples/static-html/index.html --css examples/static-html/style.css --width 480 --height 440 > stdout.ppm
cargo run -- --help
```

The `rwe` binary reads one UTF-8 HTML file and optionally one CSS file. Relative
paths are resolved against the working directory; paths with spaces work when
quoted. Use `--` before an input filename beginning with `-`. Inline style and local stylesheet links are loaded automatically. Stdin, URLs and image loading are unsupported.

| Option | Behavior |
| --- | --- |
| `--css PATH` | One author stylesheet; otherwise only inline styles apply |
| `--width N` | Viewport and canvas width, default 800 |
| `--height N` or `auto` | Canvas height, default auto |
| `--background HEX` | CSS hex or `transparent`, default `#fff`; PPM requires opaque |
| `--format png` or `ppm` | Explicit encoding; otherwise `.png` output selects PNG, everything else PPM |
| `--output PATH` or `-` | Create a new image file, or binary stdout (default) |

Auto height uses root flow height, with a one-pixel minimum for empty documents;
it excludes visible child/text overflow. Set an explicit height to capture more
of that overflow. Each dimension must be 1..16384 and the canvas may contain at
most 16,777,216 pixels. HTML and CSS sources are read with a 1 MiB limit each;
HTML also retains default node/depth limits. These are structural/allocation
bounds, not a CPU quota. PNG preserves alpha; PPM requires an opaque background
because it contains RGB only. Transparent backgrounds are accepted only with PNG.

Malformed options, inputs, unsupported syntax, allocation failures, and output
errors produce a diagnostic on stderr and a nonzero exit code. Successful
rendering adds no status text to stdout. Parsing and layout finish before a file
is created; existing output paths are refused, including an input path accidentally
used as output. The CLI does not provide a force-overwrite option. A later write
failure may leave a partial newly created output file. Shell `>` redirection is
managed by the shell and can overwrite an existing file; `--output` is the option
that enforces file creation without overwriting.

`Canvas::write_ppm(writer, opaque_background)` is also available to embedders.
It writes P6 RGB data in row order and flattens alpha against the supplied color.
Zero-sized canvases and translucent backgrounds return `InvalidInput`; writer
errors propagate. The method does not flush, so callers control buffering and
must handle flush failures. Both image examples now use this shared encoder.

## Native PNG export (unreleased)

```sh
cargo run -- render examples/static-html/index.html --css examples/static-html/style.css --width 480 --output page.png
cargo run -- render fragment.html --format png --background transparent > transparent.png
```

`Canvas::write_png(writer)` delegates compression, filtering and PNG chunks to
image-rs/png. It preserves straight RGBA, including hidden RGB under zero alpha.
The adapter streams one RGBA row at a time; encoded bytes may differ between
library versions. Pixel values, rather than the previous stored-DEFLATE bytes,
are the compatibility contract. No color management or animation is added.
Zero dimensions or dimensions above i32::MAX return `InvalidInput` before output;
I/O failures propagate and the caller remains responsible for flushing.

CLI format inference is case-insensitive for `.png` extensions; other names and
stdout keep the previous PPM default. `--format` overrides the extension, so use
`--format png` for PNG on stdout. PNG backgrounds can use `transparent` or alpha
hex colors; white remains the default. Existing-file protection and input/canvas
limits apply to both formats.

## HTML5 document and fragment parsing (unreleased)

`Document::parse` builds an immutable flat DOM arena with a synthetic document root
at index zero. `nodes()`, `node(id)`, and `text_content(id)` expose the tree without
recursive traversal or recursive node destruction.

```rust
use rwe::Document;
let document = Document::parse("<main><p>Hello &amp; 你好</p><br></main>")?;
assert_eq!(document.text_content(0).as_deref(), Some("Hello & 你好"));
# Ok::<(), rwe::HtmlError>(())
```

HTML tokenization, character references and recovery are provided by html5ever
0.40.1. `parse` uses a div fragment context; `parse_page` builds an HTML document
with implied html/head/body; `parse_fragment` accepts a local name and namespace
for contextual innerHTML (including table and SVG contexts).

- Omitted tags, duplicate attributes and invalid character references follow
  HTML5 recovery. A slash does not close a non-void HTML element.
- Comments are represented by `NodeKind::Comment`; they do not paint or contribute
  to descendant text. Element `Node::namespace` retains namespace information.
- The public arena is preorder, excluding parser-only setup/orphan nodes. Template
  contents are projected as children of their template for static consumers;
  the JS host attaches those nodes to `template.content`.
- `ParseLimits` defaults remain 1 MiB, 100,000 output nodes and 256 element levels.
  Input bytes are checked first; construction has a node budget with four setup
  nodes, and a temporary depth allowance for parser context. Parsing is fed in
  UTF-8-aligned 1 KiB chunks and stops after a construction-limit failure. Final
  output is checked against the exact public limits. This is not a total memory
  quota. Limit error offsets indicate the last processed byte position.
- Initial pages and JS innerHTML call the same Rust parser. innerHTML prepares a
  detached result before replacing old children, so a rejected parse preserves
  the old subtree. JSON conversion uses serde_json/Boa rather than hand-escaped JS.

The hand-written Rust parser and JS regex fragment/entity parsers were removed.
This change does not provide complete browser DOM, CSS, SVG painting or scripting
semantics. The static CLI still does not execute scripts.

HTML fragments can now feed the block renderer below. Bitmap text is supported; The optional JavaScript host is described in APPLICATIONS.md.

## Inline styles and block rendering (unreleased)

```rust
use rwe::{BlockLayout, Canvas, Color, Document};
let document = Document::parse(
    "<main style='padding: 8px; background-color: #eef'><div style='height: 24px; background-color: #26d'></div></main>"
)?;
let layout = BlockLayout::new(&document, 120)?;
assert_eq!(layout.height(), 40);
let mut canvas = Canvas::new(120, 40)?;
canvas.clear(Color::WHITE);
layout.paint(&mut canvas);
# Ok::<(), Box<dyn std::error::Error>>(())
```

`BoxStyle::parse` accepts semicolon-separated inline declarations. Property names
and keywords are ASCII case-insensitive. Supported properties are:

| Property | Supported values | Default |
| --- | --- | --- |
| `width`, `height` | Nonnegative integer `px`, unitless `0`, or `auto` | `auto` |
| `margin`, `padding` | One to four nonnegative integer `px` values or unitless `0` | All sides `0` |
| `margin-*`, `padding-*` | Physical top/right/bottom/left longhands, one length | `0` |
| `border-width` | One nonnegative integer `px` value or unitless `0` | `0` |
| `border-style` | `none`, `solid` | `none` |
| `border-color` | CSS hex, `transparent`, or `currentcolor` | `currentcolor` |
| `border` | Exactly `width style color`, or `none` | See components |
| `background-color` | CSS hex colors or `transparent` | `transparent` |
| `display` | `block`, `none` | `block` |
| `overflow` | `visible`, `hidden` | `visible` |
| `color` | CSS hex colors or `transparent` | Inherited, root black |
| `font-size` | Multiples of `8px` from `8px` through `64px` | Inherited, root `16px` |
| `text-align` | `left`, `center`, `right` | Inherited, root `left` |
| `white-space` | `normal`, `pre` | Inherited, root `normal` |

Repeated declarations use the last value. Unsupported or malformed declarations
return `RenderError`; this is deliberately stricter than browser CSS recovery.
Unlisted shorthand properties, percentages, fractional lengths,
negative margins and `!important` are not implemented. Text color, size, alignment, and whitespace processing inherit. Author stylesheet
selectors and stylesheet comments are supported as described below.

Every visible element is a vertical block. Width and height describe the content
box. Auto width fills the parent's content width after subtracting margins,
padding, and border, with a minimum of zero; auto height fits the child flow. Margins do
not collapse. Fixed dimensions allow visible child overflow unless `overflow: hidden` is set. Backgrounds cover
the entire border box, including beneath borders, and paint parent before child,
using source-over blending.
`display: none` skips the entire subtree, including descendant style validation.
All tags otherwise use the same block behavior: there are no browser default styles.

Text nodes now contribute wrapped line heights and bitmap glyphs. Layout uses
an explicit work stack, supports deep DOM arenas, and rejects geometry exceeding
nonnegative signed 32-bit coordinates. `boxes()` includes the transparent root
box; `height()` reports root flow height and excludes overflowing descendants.
Painting clips at canvas edges and active ancestor overflow clips. The caller chooses canvas dimensions and
allocation limits; layout never allocates a canvas automatically.

## Physical edges and solid borders (unreleased)

Margin and padding shorthands expand in CSS top/right/bottom/left order: one value
sets all sides, two set vertical/horizontal sides, three set top/horizontal/bottom,
and four specify every side. Side longhands and shorthands cascade independently
per affected side, including across specificity and inline overrides. Margins
remain nonnegative and never collapse; `auto` and logical edges are unsupported.

Borders are uniform on all sides. `border: 1px solid #cbd5e1` sets all three
border components; this subset requires width/style/color in that order.
`border: none` resets the width to zero, style to none, and color to currentcolor.
The three component longhands can override them separately. A none style occupies
no space regardless of specified width. Default width is zero (there is no browser
medium-width default). Currentcolor resolves to the element's final inherited
text color; border components themselves do not inherit. Transparent and alpha
hex border colors are supported.

Layout uses content-box sizing: specified dimensions exclude padding/border;
auto width subtracts all horizontal edges and auto height includes child flow
plus vertical padding/border. Background paints across the border box, then solid
border strips, then descendants. Corners belong to exactly one strip, avoiding
duplicate alpha blending. Visible descendant overflow may cover parent borders; hidden overflow preserves
the container border by clipping descendants to the inner border edge.
Per-side border widths/colors, dashed borders, rounded corners, shadows, other clipping
properties, and border-box sizing are not implemented.

**Development API change:** `BoxStyle.margin` and `.padding` now use `Edges`
with `top`, `right`, `bottom`, and `left` fields, rather than a scalar. Use
`Edges::all(n)` to create equal edges. `LayoutBox.rect` now includes borders;
`content_rect` excludes them and padding. Existing border-free geometry is unchanged.
`border_width` and `border_color` expose the effective painted border.

## Overflow clipping (unreleased)

```sh
cargo run -- render tests/fixtures/rendering/overflow.html --css tests/fixtures/rendering/overflow.css --width 480 --output overflow.png
```

`overflow: visible` is the default. `overflow: hidden` clips all descendant
backgrounds, borders, and bitmap text to the element's padding box (inside its
border, including padding). Its own background and border paint first and are not
clipped by its own overflow property. Ancestor clips still apply. Nested clipping
uses intersections, and later siblings resume painting under their own ancestor
clips. Empty clip areas suppress descendant pixels without suppressing layout.

Overflow is not inherited; a child's `visible` cannot escape a hidden ancestor.
Stylesheet specificity, declaration order, and inline overrides apply normally.
The property only controls painting: box and text metrics, root flow/CLI auto
height, and layout-limit errors are unchanged. Descendant overflow does not grow
fixed-size parents. `LayoutBox.clip_rect` exposes a hidden element's local padding
clip; it is `None` for visible elements. This is not the ancestor-intersected clip.

The painter uses an explicit clip stack and can repaint onto differently sized
canvases. `Rect::intersection` provides overflow-safe rectangle intersections;
`Canvas::fill_rect_clipped` applies an additional per-call clip without changing
later canvas operations. Both rectangles and individual scaled glyph pixels use
the same clip path. Scrollbars, scrolling, `overflow-x/y`, `auto`, `scroll`, `clip`,
rounded clipping, and browser block-formatting-context effects are unsupported.

## Author stylesheets and cascade (unreleased)

```rust
use rwe::{BlockLayout, Document, Stylesheet};
let document = Document::parse("<main id='app' class='card'></main>")?;
let css = Stylesheet::parse(".card { padding: 8px } #app { height: 24px }")?;
let layout = BlockLayout::with_stylesheet(&document, &css, 120)?;
assert_eq!(layout.height(), 40);
# Ok::<(), Box<dyn std::error::Error>>(())
```

`Stylesheet::parse` accepts up to 1 MiB of CSS, delegates comments and syntax to cssparser, and validates all rules and declarations eagerly. It uses the same
properties and value syntax as `BoxStyle`. Invalid or unsupported input returns
`RenderError`, even in unmatched rules. Errors currently have messages without
CSS source positions.

Selectors support types (`div`), universal (`*`), classes (`.card`), IDs (`#app`),
compound forms (`section.card.active`, `*.card`, `#app.card`), and comma lists.
Type names begin with an ASCII letter. Class/ID identifiers begin with an ASCII
letter, underscore, or hyphen; remaining characters are ASCII letters, digits,
underscores, or hyphens. Types are ASCII case-insensitive, classes and IDs are
case-sensitive, and class attributes match whole ASCII-whitespace-separated
tokens. Descendant whitespace (`main p`) and child (`main > p`) combinators
can connect compound selectors, including mixed chains such as
`#app > section.card p.note`. Child combinators require the immediate element
parent; descendant combinators match any strict ancestor. The synthetic document
root is never an element match. Whitespace around `>` is optional; stylesheet
comments do not create whitespace: `.a/**/.b` matches two classes on one element.
Sibling combinators (`+`, `~`), attribute selectors, pseudo-classes, escapes,
at-rules, and nested rules are unsupported.

Specificity sums ID, class, and type counts across the entire chain; combinators
and universal selectors add no specificity. Matching uses an iterative dynamic
pass over the candidate ancestor path, retaining alternative ancestors when
mixed child/descendant chains are ambiguous. It uses no recursive or exponential
backtracking. Structural matching costs up to ancestor depth times compound count
per candidate selector (plus individual compound comparisons); the existing 1 MiB
stylesheet bound is not a CPU quota. Applications choose appropriate DOM/input
limits for their workload.

Each property cascades independently: ID count takes precedence over class count,
which takes precedence over type count. Equal specificity uses later source order;
selector lists use the most specific matching member. Later declarations within
a rule win. Inline declarations then override stylesheet declarations, including
explicit resets such as `auto`, `0`, `transparent`, and `display: block`.
Text color, size, alignment, and whitespace processing inherit during layout; there is no `!important`.
`computed_style(&document, node_id)` exposes cascaded element declarations
(inherited text fields remain `None` until layout); it returns `None` for other or invalid nodes.

`BlockLayout::new` retains the inline-only entry point. `with_stylesheet` takes
one parsed author stylesheet. Application examples use `LocalPage` through the public
`Application` API to load inline and local linked CSS; networking is not implemented.

## Bitmap text (unreleased)

The engine includes original MIT-licensed 5-by-7 bitmap patterns, with no font or
system drawing dependency. Latin letters (lowercase uses uppercase shapes),
digits, and common punctuation have glyphs. Space and NBSP are blank. Other
Unicode scalars, including Chinese and emoji, use a visible fallback box; full
Unicode fonts, shaping, bidirectional text, and combining marks are not implemented.

Each text node forms its own vertical flow item. With `white-space: normal`,
ASCII whitespace collapses to one space and is trimmed at node boundaries;
whitespace-only nodes occupy no space. Newlines collapse, not force breaks.
Wrapping prefers whole words separated by ASCII whitespace. Overlong words
split by Unicode scalar count to ensure progress, including at zero content width
(one character per line). No Unicode line-break or grapheme algorithm is used;
NBSP remains inside a word but can split if that word is overlong.

With `white-space: pre`, spaces and explicit line breaks are preserved, CRLF and
CR normalize to LF, and tabs expand to four-column stops. There is no automatic
wrapping; long lines overflow. Empty lines occupy a full cell height. A final LF
terminates its preceding line without creating an additional phantom line; empty
source creates no lines. Leading/trailing spaces are retained. No HTML-specific
initial-newline removal is applied to `pre` elements. `pre` tags need an explicit
`white-space: pre` declaration because the engine has no browser default styles.

`text-align` positions each line at the left edge, center, or right edge of the
content box. Center offsets round down to integer pixels. Alignment uses full
cell advances rather than ink bounds; overflowing lines stay at the left edge.
All four text properties inherit and can be overridden by stylesheet or inline
values, including explicit resets to `left` and `normal`.

Sibling text/element nodes still do not share inline lines, and `br` has no special
line-break behavior beyond the existing block model. `pre-wrap`, `nowrap`,
justification, bidi alignment, and CSS `tab-size` are unsupported.

`font-size` is the cell height: 8, 16, 24, 32, 40, 48, 56, or 64 pixels. Each glyph
uses a scaled 5-by-7 mask in a 6-by-8 cell. Lines occupy one full cell height;
font-size defaults to 16px, text color to black. Color, size, alignment, and whitespace processing inherit through
containing elements, with stylesheet/inline declarations overriding inherited
values. Other properties do not inherit. `inherit`, `initial`, and `unset` keywords
are not supported. In normal whitespace mode at zero or very narrow content width, one character
is placed per line with visible overflow, then clipped by canvas edges.

`BlockLayout::text_runs()` exposes source node IDs, normalized line strings,
resolved color/size, and cell rectangles. Auto-height containers include text
lines; fixed-height containers allow text overflow. Text and backgrounds share
source-ordered paint commands, so later sibling backgrounds cover earlier text
when they overlap, subject to ancestor overflow clips. Pixel masks blend using the existing source-over compositor;
there is no antialiasing or external font loading. The page example now displays
actual glyphs, word-wrapped paragraphs, a centered button label, a right-aligned
footer, and a preserved multiline code block.

## Project direction and implementation policy

Build an application framework by composing established libraries. Reuse mature
implementations across all modules before writing foundational algorithms. RWE
owns application APIs, resource boundaries, lifecycle and thin adapters. Zero
dependencies and maintaining every subsystem in-house are not goals. See
[AGENTS.md](AGENTS.md), [REUSE-AUDIT.md](REUSE-AUDIT.md) and
[CORE-DEPENDENCIES.md](CORE-DEPENDENCIES.md). Existing engine candidates are
not integrated or approved merely because they appear in the audit. Optional
JS/window/font dependencies retain their separate reviews.

Next milestones: full fonts and inline text flow, expanded CSS/DOM support and broader application compatibility. Software-canvas size limits should be enforced by the embedding application for untrusted input.

## License

MIT. See [LICENSE](LICENSE).



## Optional native macOS window

```sh
cargo run --locked --features window --bin rwe-window
cargo run --locked --features window --bin rwe-window -- examples/static-html/index.html --css examples/static-html/style.css --watch
./scripts/bundle-macos.sh
open target/RWE.app
```

The default renderer uses html5ever, cssparser, Taffy and png. The optional window uses winit and softbuffer, displaying RWE's own pixels without a WebView. Reviewed and tested on Apple Silicon macOS only; see [WINDOW-DEPENDENCIES.md](WINDOW-DEPENDENCIES.md). Dependency internals use platform FFI; RWE's own unsafe-code prohibition remains in force.

Resize to reflow; wheel/arrows/PageUp/PageDown/Home/End scroll; R reloads local files, retaining the last working page on failure; Escape closes. No arguments opens the bundled demo. File paths are relative to the working directory; sources are UTF-8 and limited to 1 MiB each. Retina presentation uses logical layout pixels. Idle windows wait for events.

The logical page and physical surface each have a 16384-per-dimension and 16777216-pixel bound. Scrolling caches the whole page, uses root flow height, and excludes visible overflow beyond it. No scrollbar, browser navigation or networking. The optional `javascript` feature adds the documented DOM keyboard and single-line text editing subset. Bundles produced by the script are unsigned development builds.


Pass `--watch` or press W to toggle automatic reload. Every 500 ms the host hashes bounded local file contents, detecting same-size edits and atomic saves without relying on timestamps. Invalid or missing sources retain the last working page; each unchanged failure is reported once, and the next edit can recover. No new dependencies are required.

Drop `.html`/`.htm` to open a local page, then `.css` to attach a stylesheet. Opening a different HTML clears the previous CSS. Successful drops reset scroll and enable watching; failed drops preserve all previous sources and pixels. The title shows the active file, watch state and errors. Failed relayouts preserve the previous pixels; oversized display buffers wait for a smaller resize instead of closing the window.

## Project layout

- `src/`: reusable framework implementation and public Application/DesktopApp APIs.
- `examples/`: per-application folders with thin Rust bootstraps and frontend assets.
- `tests/`: automatic assertions; `tests/fixtures/` contains deterministic inputs.
- `tools/`: development utilities such as batch application snapshots.
- `scripts/`: local development packaging.

See [application examples](examples/README.md), [test guide](tests/README.md) and
[development tools](tools/README.md) for commands and responsibility boundaries.

Standalone Vue example: `cargo run --features window,javascript --example vue-counter`.
Its entry point is `examples/vue-counter/main.rs`; window/runtime/rendering remain in the framework.

Window startup settings are shared by DesktopApp and rwe-window: title, initial/minimum
logical dimensions, opaque backdrop and resizing. See [Window configuration](APPLICATIONS.md#window-configuration)
for the builder and matching CLI flags. Invalid settings fail before page loading
or native startup.

Single-line text example: `cargo run --features window,javascript --example text-input`.
The framework renders values/placeholders, handles caret/selection and editing,
and dispatches beforeinput/input/change; official Vue v-model and React controlled
onChange are regression-tested. See [single-line input](APPLICATIONS.md#single-line-text-input)
for APIs and current limitations, including clipboard support. Experimental IME
preedit/commit/cancel and candidate positioning are documented under
[input-method composition](APPLICATIONS.md#input-method-composition); native manual
validation remains outstanding; optional CJK font coverage is available through `fonts`.

Chinese display/input example: `cargo run --features window,javascript,fonts --example chinese-input`.
The optional embedded CJK fallback shares advances with text layout, selection and
IME positioning, without changing default bitmap output. See
[font review](FONT-DEPENDENCIES.md) and [font provenance/license](src/assets/fonts/README.md).

设置表单案例：`cargo run --features window,javascript,fonts --example settings-form`。
能力范围与下一步：[兼容性清单](COMPATIBILITY.md)。

布局进展：支持显式像素基准的单行横向 Flex、gap、整数 flex-grow、
主轴分布和交叉轴对齐；范围及报错边界见 APPLICATIONS.md 的 Flex 章节。

CSS declarations/rules now use cssparser 0.38.0, including the JS cssText bridge.
Comments do not create selector whitespace: `.a/**/.b` is a compound selector.
EOF comment/block recovery follows the library. Unsupported property semantics,
at-rules and selectors still return errors. This is not a complete CSSOM/Stylo
integration. See REUSE-AUDIT.md and CORE-DEPENDENCIES.md, including MPL-2.0 notices.
