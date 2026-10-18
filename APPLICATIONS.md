# Framework application API

Standalone examples each contain a thin `main.rs` bootstrap plus index.html,
style.css and optional app.js. The bootstrap selects the application's HTML and
calls DesktopApp; reusable window/runtime/render implementations stay in RWE.
The shared rwe-window launcher remains available for frontend-only workflows.

```rust
fn main() -> Result<(), String> {
    rwe::DesktopApp::new("index.html").watch(true).run()
}
```

Enable `window,javascript` for interactive desktop apps. The host owns its window,
event loop, source reload, input, timers and repainting. The framework-provided `rwe-window` executable runs HTML applications directly;
standalone examples demonstrate the equivalent public Rust startup API.
The no-argument CLI start page lives in
framework assets and no longer depends on example application files.

## Window configuration

Applications configure the framework; they do not implement the window:

```rust
rwe::DesktopApp::new(entry)
    .title("Vue Counter")
    .size(960, 640)
    .min_size(320, 240)
    .background(rwe::Color::rgba(237, 244, 239, 255))
    .resizable(true)
    .watch(true)
    .run()
```

`window_options(WindowOptions { .. })` also accepts a complete configuration;
`options()` allows inspection before startup. Window dimensions are logical
pixels. Defaults are 800x600, minimum 160x120, title RWE, opaque white background
and resizing enabled. The minimum must not exceed the initial size. Logical
initial/minimum buffers and actual physical display buffers use the same
16384-per-dimension / 16777216-pixel checks as application rendering. Native
creation and file loading happen only after configuration validation succeeds.
Titles must contain 1..256 characters without controls; background alpha must be
255. The host preserves the application title and adds file/reload/error status.

The CLI uses the same WindowOptions and validation:

```sh
cargo run --features window,javascript --bin rwe-window -- examples/vue-counter/index.html --title "Vue Counter" --width 960 --height 640 --min-width 320 --min-height 240 --background '#edf4ef' --watch
```

`--fixed-size` disables user resizing. Window backdrop settings apply on startup,
resize, file replacement, reload and live script repaint. CSS backgrounds draw
on top. Headless image rendering uses its separate RenderOptions background.
Failed reload/relayout retains the previous pixels; source changes preserve
window settings. Losing focus cancels a pending pointer click.

For headless rendering/validation:

```rust
let mut app = rwe::Application::load(std::path::Path::new("index.html"), None)?;
app.click("increment")?;
app.save_png("result.png", rwe::RenderOptions::default())?;
```

`Application` owns local resources, optional script execution, the live DOM,
layout, render limits and image export. `load_static` deliberately skips scripts
for static CLI export. `from_html` accepts static markup and supplied CSS without
resource loading or script execution. `document`/`layout` support inspection.
`RenderOptions` configures width, exact or minimum automatic height and background.
PNG file exports refuse existing paths and validate rendering before file creation.

`examples/static-html/`, `vue-counter/`, `vue-template/` and `react-counter/`
each contain main.rs and application resources. Their Cargo example targets are
registered in the repository manifest with the required features.
Use `cargo run --features window,javascript --example vue-counter` to build a
standalone example; `rwe-window` and `rwe render` provide shared-host workflows.
Low-level parsing/painting cases live in `tests/` with automatic assertions;
`tools/app-snapshots.rs` is the separate developer screenshot utility.

---

# Static HTML, Vue and React (experimental)

RWE paints its own DOM using its Rust layout and software canvas. The optional
JavaScript feature runs Boa, not a WebView. Framework support is deliberately a
small tested subset, not compatibility with arbitrary browser applications.

## Run locally

JavaScript builds require Rust 1.91 or newer. The default renderer retains its
Rust 1.85 requirement; default dependencies now include Taffy and png.

```sh
cargo run --locked --features window,javascript --bin rwe-window -- examples/static-html/index.html --watch
cargo run --locked --features window,javascript --bin rwe-window -- examples/vue-counter/index.html --watch
cargo run --locked --features window,javascript --bin rwe-window -- examples/react-counter/index.html --watch
```

The Vue template variant is `examples/vue-template/index.html`, exercising the official runtime compiler and keyed list updates.

The bundled demo files use unmodified official Vue 3.5.13 and React/ReactDOM 18.3.1
production packages, included locally with MIT licenses and source checksums in
`examples/THIRD-PARTY.md`. Click the counter button to exercise framework
state and DOM reconciliation. Reloading replaces the runtime and resets app state.

The native window accepts HTML/HTM drops, then an optional CSS drop. W toggles
500ms content polling, including linked CSS and scripts. R reloads; scrolling and
resize retain their existing controls. Dropping a different HTML clears explicit
CSS. Parsing/loading failures keep the previous working page. Script failures
retain the last successfully painted frame; mutations already made in a running
JavaScript realm are not rolled back. Reload to create a fresh realm.

## Static HTML

`Document::parse_page` uses html5ever 0.40.1, adding implied html/head/body and
HTML5 recovery. `Document::parse` uses a div fragment context; `parse_fragment`
accepts element/namespace context for table/raw-text/foreign-content parsing.
The JS host's innerHTML delegates to the same native parser instead of regex.
Comments and element namespaces are retained; templates remain inert. Limit
failures reject the update before replacing existing children. Source/node/depth
limits remain; syntax recovery intentionally changes the old strict behavior.
This does not complete the browser DOM, serializer or SVG renderer.

`LocalPage::load` collects inline style and local stylesheet links in source
order; explicit `--css` is appended last. Metadata and inert template subtrees are
not painted. Classic scripts are discovered in document order. The image-export
CLI uses the static DOM and **does not execute scripts**.

Local stylesheet/script references must stay under the HTML file's directory,
including through symlinks. URLs, absolute resource paths, query strings and
fragments are rejected. Sources/combined CSS are limited to 1 MiB, total resource
bytes to 8 MiB and external references to 32. CSS remains the documented RWE
subset: unsupported properties/selectors are errors, not silently approximated.
Images, full fonts, flex/grid and browser default styles are not implemented.

## JavaScript/DOM scope

The host implements document/element/text/comment creation, parent/child/sibling
operations, textContent, attributes, inline styles, simple id/class/tag queries,
cloning, a limited innerHTML parser, DOM serialization and bubbling/capture click
events. Native pointer clicks are hit-tested against painted layout boxes and
ancestor overflow clips. Promise jobs and currently due setTimeout callbacks are
flushed before rendering; requestAnimationFrame uses those timers. String timer
callbacks, setInterval and full browser task ordering are not implemented.

Classic local scripts and IIFE/UMD application bundles are supported. JSX/TSX and
`.vue` source files require precompilation by the application's existing build
setup. Vite development servers, ES module graphs, dynamic import, routing,
fetch/XHR, browser storage, full DOM/CSSOM, general form handling, SVG rendering,
React 19 and broad ecosystem compatibility are not claimed. The keyboard and
single-line text input subsets are documented below.

Execution limits include loop iterations, recursion, 1024 queued promise/timer
callbacks, 32 timer-drain rounds, 100000 allocated DOM nodes and 256 serialization
depth. These are not memory/wall-clock limits or a security sandbox: run trusted
local application code only. Dependency reviews are recorded separately in
`JS-DEPENDENCIES.md` and `WINDOW-DEPENDENCIES.md`.

## Verification and actual rendered output

```sh
cargo test --locked --features window,javascript --all-targets
cargo test --locked --features javascript --doc
cargo clippy --locked --features window,javascript --all-targets -- -D warnings
cargo run --locked --features javascript --bin rwe-app-snapshots -- /tmp/rwe-previews
./scripts/bundle-macos.sh
./scripts/bundle-apps-macos.sh
```

The snapshot example executes the real frameworks, clicks each button, and
writes before/after PNGs using the same DOM/layout/painting path as the window.
Integration tests verify framework state changes, coordinate hit testing, changed
pixels, local asset loading and event propagation without needing UI permissions.

## Embedded applications

For a movable executable, use `AssetBundle::new("index.html", &[("index.html",
include_str!("index.html")), ...])?`, then `DesktopApp::from_assets(assets).run()`.
Include all linked CSS and classic scripts under their virtual paths. The framework
uses the same resource parser and application lifecycle as local pages, without
runtime filesystem access. `Application::from_assets(&assets)` exposes the same
behavior for headless rendering and interaction tests.

`examples/embedded-vue/main.rs` demonstrates this with the official Vue runtime.
`sh scripts/bundle-embedded-macos.sh` packages it as a movable unsigned macOS app.
See `examples/embedded-vue/README.md` for limits and development-mode behavior.

## Keyboard and focus

With `javascript` enabled, local/embedded application loading creates a DOM runtime
also for script-free pages, allowing native buttons to participate in keyboard
navigation. Explicit static rendering (`from_html`, `load_static`) does not create
an interactive runtime.

`Application::keyboard(&KeyEvent::down("Tab"))` dispatches host-independent input;
`KeyEvent::up(" ")` releases Space. Modifier/repeat fields are public. The return
value tells the host whether the application or a built-in action consumed the key.
The DOM receives bubbling `keydown`/`keyup`, with `key`, `repeat`, `shiftKey`,
`ctrlKey`, `altKey`, `metaKey` and `preventDefault()`. Vue `onKeydown` and React
`onKeyDown` use the same event path.

Tab and Shift+Tab traverse enabled buttons and elements with nonnegative `tabindex`,
positive values first, then document order; traversal wraps within this single
window. `tabindex=-1` supports programmatic focus only. Disabled, hidden, inert,
`display:none` elements and their descendants are excluded. `focus()`/`blur()` and
`document.activeElement` are supported for this focusable subset, with `focus`,
`blur`, `focusin`, `focusout` and `relatedTarget` events. Enter clicks a focused
button once per press; Space clicks on release. Focus changes, removed/disabled
controls and native window focus loss cancel pending Space activation.

Keyboard focus gets a two-pixel blue inset ring, clipped by ancestors. The native
host scrolls consumed keyboard input's focused box into view. Pointer focus does
not show this ring. A prevented key suppresses both built-in activation and host
shortcuts. Unhandled Escape/scroll/R/W controls retain their existing behavior;
Control/Alt/Meta combinations are delivered without triggering these shortcuts.

`examples/keyboard-buttons` validates this flow with application-only counter logic.
Physical key codes, CSS pseudo-classes, full browser focus
semantics and accessibility APIs are not yet implemented. Single-line text editing
is described below. This feature does not
claim complete browser or accessibility compatibility.

## Single-line text input

Text and search inputs (including omitted `type`) participate in focus traversal,
render `value` or `placeholder`, and clip to one line. A caret and selection highlight
are drawn in the content box, with horizontal scrolling to keep the caret visible.
Static render paths also display input values. `type=hidden` is excluded from layout.

`KeyEvent` accepts optional native `text`: `None` uses a single-character logical key,
while an empty string suppresses insertion (dead keys). The native host forwards
`winit` key text. `Application::insert_text(text)` accepts committed single-line text
for other hosts and headless tests; it validates a 256 KiB call budget and rejects
control characters. Values have a 65536 UTF-16-unit limit; user edits also respect
`maxlength`, without splitting surrogate pairs. Logical selection offsets use UTF-16.

The DOM implements `value`, `defaultValue`, `type`, `readOnly`, `selectionStart`,
`selectionEnd`, `setSelectionRange` and `select` for this editing subset.
Programmatic value assignment does not emit editing events. User edits dispatch a
cancelable `beforeinput`, then a noncancelable bubbling `input` with `data` and
`inputType`. A changed user-edited value emits `change` on Enter or focus loss;
committing twice without another edit does not emit twice. Vue template `v-model`
and React controlled `value`/`onChange` are verified against their official runtimes.

Backspace/Delete, Left/Right/Home/End, Shift selection and Control/Command+A are
supported. Read-only fields remain focusable/selectable; disabled controls are
skipped. Printable letters and editing keys in the field are consumed before host
R/W/scroll shortcuts. A canceled key or beforeinput prevents the default edit.

`examples/text-input` is an embedded greeting application. It contains only startup
configuration and greeting/reset logic; editing stays in the framework. Package it
with `sh scripts/bundle-embedded-macos.sh target/RWE-Text-Input.app text-input`.

This is single-line text/search support. Clipboard,
undo/redo, pointer caret positioning/selection, multiline textarea, input types beyond text/search/checkbox/radio,
HTTP submission, validation APIs and accessibility integration remain unsupported.
The caret does not blink; clicking places it at the end. Unicode editing uses code
points, not full grapheme clusters, and the existing bitmap font glyph limits apply.

## Input-method composition

`Application::ime(&ImeEvent::Preedit { text, cursor })` displays transient text in
an editable focused text/search control. `cursor` is an optional ordered UTF-8 byte
range into `text`, matching winit; the DOM bridge converts offsets to UTF-16. None
hides the preedit caret. The committed `value` and framework model stay unchanged
while the renderer shows the replacement preview with an underline. Composition
emits `compositionstart` once and `compositionupdate` for each update.

`ImeEvent::Commit(text)` replaces the original selection, respecting `maxlength`
and cancelable `beforeinput`, then emits `compositionend` and the final noncomposing
`input`. Official Vue v-model and React controlled inputs/composition callbacks are
tested with Chinese commits. `ImeEvent::Cancel`, focus loss, removal/disable,
read-only changes, programmatic value/selection changes and pointer activation
cancel the preview. Late commits after cancellation are ignored until a new
nonempty preedit starts. `insert_text` does not bypass an active composition.
`is_composing()` reports the session; keyboard events include `isComposing`.

An empty preedit clears the preview while preserving the session for an immediate
commit. The macOS host drains same-batch native IME events before idle cancellation:
clear+commit is retained, while a clear with no commit cancels at idle. Escape
cancels an active composition before the host's normal close shortcut.

The native host enables winit IME only for a focused editable input and disables it
on blur or when focus moves elsewhere. It positions the system candidate area from
`ime_cursor_rect(width)` in logical page coordinates, subtracting scroll and
clamping to the viewport. Cursor requests start after native `Ime::Enabled` and
stop after `Disabled`. macOS winit already separates marked-text commits from normal
key text, so the host does not inject a second keyboard edit for an IME commit.

Preedit is bounded to 65536 UTF-16 units/256 KiB; text controls and UTF-8 cursor
boundaries are validated before modifying state. The text-input example now shows
composition status. This is an experimental bridge with headless protocol and
rendering tests; actual system IME/candidate-window behavior still needs manual
validation. Surrounding-text/reconversion APIs, platform-specific selection clauses,
IME styling and full cross-platform IME compatibility are not implemented. The
default bitmap font lacks CJK glyph coverage. Enable the optional `fonts` feature
for the embedded CJK fallback described below.

## Optional embedded CJK font

Enable `fonts` to use RWE's embedded regular-weight CJK fallback:

```sh
cargo run --features window,javascript,fonts --example chinese-input
cargo run --features javascript,fonts --bin rwe-app-snapshots -- /tmp/rwe-chinese
```

The core renderer uses html5ever, cssparser, Taffy and png by default. With fonts enabled, supported
non-ASCII characters use the reviewed Rust fontdue rasterizer and the OFL-licensed,
renamed static Noto Sans SC subset under `src/assets/fonts/`. ASCII and NBSP keep
the existing bitmap output; unsupported characters keep the visible fallback box.
The font covers 28313 mapped scalars, including common simplified/traditional
Chinese, CJK Extension A and punctuation, not all Unicode or emoji.

Glyph advances are shared by line wrapping/alignment, input horizontal scrolling,
selection/caret painting, composition underlines and candidate cursor positioning.
Outline alpha multiplies the text color alpha and uses existing source-over and
ancestor clips. This does not use system fonts, browser APIs or a network request.
The parsed embedded face is initialized lazily, with a bounded FIFO of 512 rasters.
Only the reviewed embedded font is accepted; arbitrary user font loading is not
exposed. The font adds about 9.8 MB of uncompressed asset data to enabled binaries.

Regeneration uses the pinned input hash and FontTools version in
`tools/subset-font.py`, outside the Cargo build. Font license, source revision,
subset, hashes and dependency reviews are in `src/assets/fonts/README.md`, `OFL.txt`
and FONT-DEPENDENCIES.md. Packaging the chinese-input example includes these notices.

Font-family/weight selection, shaping, kerning, bidi, full Unicode line-breaking,
combining/grapheme algorithms and emoji are not implemented. Font-size remains
8..64 px in multiples of 8, with the existing fixed line cell and 7/8-em baseline;
outline overhang is clipped to its cell. Preserved tabs retain the earlier
four-scalar-column expansion. Native IME/candidate-window behavior still needs
manual validation; Chinese rendering, model updates and pixel cursor geometry are
automatically tested through the public framework API.

## Local forms

Checkboxes and radios support checked/defaultChecked, disabled state, labels,
click cancellation, input/change events and Space activation. Named radios share
a group by form owner; arrow keys navigate enabled members. Tab visits one member
per group. Rendering and activation belong to the framework, not example code.

Forms expose elements, requestSubmit and cancelable submit/reset events. Submit
buttons and Enter in text inputs dispatch submit with a submitter. Native reset
restores default values without input/change events. Controlled Vue/React state
should be reset in the application's reset handler. String-only FormData collects
named enabled fields and selected toggles, supports duplicate keys and iteration.
It performs no HTTP request or persistence. Official Vue v-model and React
checked/onChange are regression tested.

Run the embedded Chinese settings example:

```sh
cargo run --features window,javascript,fonts --example settings-form
```

Select, textarea, files, constraint validation, image coordinates, the disabled
fieldset first-legend exception and full browser form behavior are not implemented.
See [COMPATIBILITY.md](COMPATIBILITY.md) for the wider framework roadmap.

## Explicit-basis flex rows

`display: flex` now lays out one horizontal row. Visible direct element items
require integer-pixel `width` or `flex-basis`; basis overrides width. Whitespace-only
text and hidden elements do not consume space. Wrap nonempty text in an element.

Supported properties are `gap`, nonnegative integer `flex-grow`, `flex-basis`
(px/auto), `flex-shrink` (nonnegative integer, default 1), `justify-content` (flex-start/center/flex-end/
space-between) and `align-items` (stretch/flex-start/center/flex-end).
The default alignment is stretch. Taffy computes growth/shrink and alignment from mapped CSS styles. Growth divides remaining content width after
margins, padding, borders and gaps; integer remainders are distributed
deterministically. Text wraps after the allocated width is known. Nested rows,
ancestor clips, caret geometry and hit testing share the resulting layout.

```css
.actions { display: flex; gap: 12px; align-items: center; }
.actions > button { flex-basis: 0; flex-grow: 1; padding: 10px; }
```

Intrinsic auto/min-content sizing, automatic min-size constraints,
wrapping, column/reverse directions, order, align-self and the flex shorthand are
not implemented. Content widths may shrink to zero without browser automatic min-content floors.
The adapter feeds actual subtree heights back to Taffy for cross-axis alignment;
block flow and text wrapping still use the existing renderer. RWE only maps styles,
measurements and paint records. Negative geometry and geometry above 2^24 pixels
are rejected by the current integer-pixel adapter. Rounding follows Taffy.
See REUSE-AUDIT.md for the full-tree and text-stack migration plan.

The settings example now uses CSS rows for labels/controls and equal-width action
buttons. The application does not contain geometry or rendering implementation.

## Shared CSS syntax

cssparser 0.38.0 parses style rules and declaration lists for author CSS, inline
BoxStyle and JS style.cssText via a native Boa bridge. Strings/nested blocks and
escaped tokens are handled by the library, not string splitting. A failed cssText
parse preserves existing declarations. Inputs are limited to 1 MiB; library
nested-block limits remain enabled. EOF comments/blocks follow CSS recovery.
Comments do not introduce descendant whitespace. Supported properties and the
existing selector matcher/cascade remain an explicit subset; at-rules, nested
rules, !important and functional values are not enabled by adopting the tokenizer.
