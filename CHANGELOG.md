# Changelog

## Unreleased

- Add optional embedded CJK outline glyphs with reviewed Rust rasterization,
  font-specific license/provenance and pinned subset regeneration.
- Share pixel advances across wrapping, alignment, text controls and IME caret
  positions; preserve default/ASCII bitmap output and bound the glyph cache.
- Add a standalone Chinese-input example, pixel/selection/cache regressions and
  macOS packaging with font notices.

- Added validated immutable `AssetBundle`, shared file/embedded resource parsing,
  `Application::from_assets` and `DesktopApp::from_assets`, plus a relocatable Vue
  example and macOS app packaging script.

- Added host-independent `KeyEvent`, keyboard focus traversal, focus events,
  Enter/Space button activation, clipped focus rings and native focus-loss
  cancellation, with Vue/React keyboard regressions and an embedded button demo.

- Added single-line text/search controls, static/live value rendering, caret and
  selection, bounded committed text, beforeinput/input/change, Unicode editing and
  verified Vue v-model/React onChange bindings, with an embedded input example.

- Added bounded ImeEvent preedit/commit/cancel, transient underlined previews,
  composition events and real Vue/React commit regressions. The macOS host toggles
  native IME by editable focus, positions candidates and handles idle cancellation.

- Add shared WindowOptions and DesktopApp configuration for application title,
  logical initial/minimum dimensions, opaque backdrop and user resizing.
- Add matching native-host CLI flags; reject invalid settings before resource
  loading or window creation, using shared rendering size/pixel limits.
- Preserve configured background through resize/reload/live updates, and preserve
  application titles in status changes; cancel pending clicks when focus is lost.
- Cover configuration boundaries, CLI/API parity, actual launcher failures and
  backdrop persistence without opening native windows during tests.

- Organize examples as static-html, vue-counter, vue-template and react-counter
  application directories with thin main.rs bootstraps, separate frontend assets,
  self-contained vendor files and explicit Cargo feature requirements.
- Retain shared rwe-window/rwe render workflows and development tools under tools/;
  update application tests, snapshots and packaging for the standalone directories.
- Move low-level print/drawing demos into DOM/pixel assertion tests and relocate
  reference rendering documents to tests/fixtures with end-to-end clipping checks.
- Move batch screenshots into the rwe-app-snapshots development binary under
  tools/; update commands and application/framework documentation accordingly.

- Promote the shared Application lifecycle and RenderOptions to public library
  APIs; centralize local loading, live updates, layout, bounded painting and export.
- Add DesktopApp startup so consumers do not implement native event loops.
- Move the native host into the library and make window/CLI/examples reuse the
  same application facade; remove runtime/layout/export assembly from examples.
- Keep the CLI start page in framework assets, independent of example apps.

- Add parse_page and bounded LocalPage loading for full explicit HTML documents,
  raw style/script, RCDATA, unquoted attributes and local linked CSS/scripts.
- Add optional Boa JavaScript execution, an original DOM/event subset, bounded
  promise/timer scheduling, native click hit testing and live DOM repainting.
- Test unmodified Vue 3.5.13 and React/ReactDOM 18.3.1 fixtures with state-changing
  clicks and actual pixels, without WebView or networking.
- Track linked CSS/JS during window reload; document classic-script scope,
  resource boundaries, dependencies and experimental application compatibility.

- Add an optional macOS native window host with resizing, Retina presentation,
  bounded scrolling, local file reload and a development application bundle script.
- Add drag-and-drop HTML/CSS opening and optional 500ms content-based auto reload
  (`--watch` / W), preserving the last working sources and page on errors.
- Keep windows alive after oversized buffers or failed relayouts so resizing or
  reloading can recover; show active file, watch state and errors in the title.
- Cover same-size edits, missing/invalid file recovery, transactional file drops,
  source limits, resizing, scroll bounds and argument handling.

- Add overflow visible/hidden author declarations and padding-box descendant clips.
- Intersect nested clips on an explicit paint stack for backgrounds, borders, and
  bitmap glyphs; restore sibling clips without changing layout metrics.
- Expose local clip rectangles, safe rectangle intersections, and per-call clipped
  canvas painting; add overflow comparison assets and pixel/deep-tree regressions.

- Add one-to-four-value margin/padding shorthands and physical side longhands,
  with per-side cascade behavior and checked asymmetric content-box geometry.
- Add uniform solid/none borders, component declarations, strict border shorthand,
  currentcolor resolution, and nonoverlapping alpha-composited border strips.
- Change development BoxStyle margin/padding fields to Edges and expose content
  rectangles and resolved borders in LayoutBox; extend geometry/pixel regressions.

- Add bounded streaming native RGBA PNG export with stored DEFLATE blocks, CRC-32,
  Adler-32, straight-alpha preservation, and no third-party or system encoder.
- Add CLI PNG format selection/inference and transparent canvas backgrounds while
  retaining PPM defaults and existing output-file protection.
- Cover independent PNG reference bytes, block boundaries, checksums, alpha,
  short writes, format overrides, and file/stdout output with regressions.

- Add the rwe render CLI for bounded local HTML/CSS inputs, configurable dimensions,
  auto height, opaque canvas colors, and stdout or newly created PPM files.
- Add shared Canvas::write_ppm export with explicit alpha flattening and I/O errors.
- Reuse the encoder in painting/page examples and cover CLI pixels, argument/input
  errors, output preservation, source limits, auto sizing, and PPM export failures.

- Add descendant and child selector combinators with mixed compound chains.
- Sum specificity across selector chains and match ancestor alternatives with an
  iterative dynamic pass rather than recursive backtracking.
- Add structural-selector parsing, ambiguous-ancestor, cascade, deep-DOM, and pixel
  regressions; migrate the page stylesheet to structural selectors.

- Prefer word boundaries for normal text wrapping, splitting overlong words safely.
- Add inherited text-align and white-space declarations, including centered/right
  lines and preserved spaces/newlines/tabs without wrapping in pre mode.
- Update text metrics regression expectations and add whitespace, alignment,
  cascade-reset, clipping, and overflow coverage; extend the page demo with code.

- Add original bitmap glyphs, character-wrapped text flow, and inspectable text runs.
- Add inherited color/font-size declarations in author and inline styles.
- Integrate text into automatic block sizing and ordered background/text painting.
- Update the page demo with real glyphs and cover text metrics, inheritance,
  fallback glyphs, clipping, alpha blending, and overlap with regression tests.

- Add strict author stylesheets with compound type/class/ID selectors, selector
  lists, block comments, per-property specificity, and inline overrides.
- Add computed-style inspection and stylesheet-aware block layout; split the page
  example into HTML and CSS and test cascade and rendered pixels.

- Connect inline-style HTML blocks to the software canvas with iterative layout.
- Support content-box dimensions, uniform margin/padding, background colors,
  and hidden subtrees with checked geometry and visible overflow.
- Add an HTML-to-PPM page example and geometry/pixel regression tests.

- Add an immutable flat DOM arena with parent/child indices and text traversal.
- Add bounded strict HTML fragment parsing, quoted/boolean attributes, comments,
  void elements, and a documented character-reference subset.
- Add a runnable HTML example and regression coverage for syntax and limits.
- Retain zero third-party dependencies and forbid unsafe code.

## 0.1.0

- Implement CSS hex color parsing and source-over RGBA compositing.
- Implement a software canvas with clipping and checked allocation.
- Add a PPM painting example, API documentation, and regression tests.
- Establish a zero-third-party-dependency, safe Rust starting point.

### Local form workflow

- Framework checkbox/radio rendering and activation, associated labels, grouped
  radio keyboard navigation, submit/reset events and string FormData.
- Regression coverage for official Vue/React controlled toggles and form submission.
- Embedded Chinese settings example and explicit compatibility/milestone matrix.

### Explicit-basis flex rows

- Single horizontal flex rows with pixel bases, integer growth, gap, distribution
  and cross alignment; nested layout, clipping and hit testing use shared geometry.
- Unsupported intrinsic sizing and negative alignment overflow fail explicitly.
- Settings form uses framework CSS rows for controls and action buttons.

### Flex shrink

- Nonnegative integer shrink factors distribute deficits by scaled original bases.
- Zero-width freezing redistributes remaining deficits in O(n log n); box edges,
  gaps and opted-out items remain fixed. Text reflows at the resulting width.
- Settings buttons now exercise shrink at narrow widths; intrinsic min-size
  constraints and flex wrapping remain unsupported.

### Mature-library-first architecture

- Added a full module reuse audit and repository-wide engineering policy.
- Delegated exposed Flex solving/alignment to Taffy 0.15.0 and PNG codecs to
  image-rs/png 0.18.1; removed the replaced custom algorithms.
- Documented remaining block/text/DOM/CSS migrations, float geometry bounds,
  pixel-rounding and encoded-byte changes; packaged dependency licenses.

### Shared HTML5 parsing

- Adopted html5ever 0.40.1 for documents and contextual fragments; removed the
  handwritten Rust parser and JavaScript regex/entity parsing.
- Added comment nodes, element namespace metadata, template-content projection
  and a bounded arena TreeSink with chunked input and final output validation.
- innerHTML uses a native Boa bridge and prepares new fragments before replacing
  children; JSON conversion uses serde_json.
- Recovery/implied page structure intentionally replace old strict syntax errors.

### Shared CSS syntax

- Adopted cssparser 0.38.0 for style rules/declaration lists and a shared native
  JS cssText bridge; removed handwritten delimiter/comment splitting.
- Correct token boundaries, string/comment handling, escaped names and bounded
  nested syntax; cssText syntax failures preserve old values.
- Documented EOF recovery and comment semantics, remaining selector/cascade work,
  and MPL-2.0 dependency licenses with original source archives.
