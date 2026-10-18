# Framework verification

Tests assert behavior; they are not application startup code.

- `html.rs`, `local_page.rs`: DOM structure, parsing and local resource boundaries.
- `stylesheet.rs`, `selectors.rs`, `layout.rs`, `text*.rs`: styles and layout.
- `painting.rs`, `png.rs`, `ppm.rs`, `overflow.rs`: pixel and encoder regressions.
- `render_fixtures.rs`: reference pages and visible/hidden clipping end to end.
- `application.rs`, `javascript.rs`, `cli.rs`: public API, frameworks and CLI contracts.
- `fixtures/`: deterministic input documents and image data, not runnable examples.
- `support/`: test-only helpers.

Run `cargo test --all-targets` for the default renderer and
`cargo test --all-targets --all-features` for the complete audited
host configuration. Tests validate application resources through public framework
APIs without opening windows. Standalone example entry points compile during all-target checks but do not
open windows during tests. Run instructions are in `../examples/README.md`.

`fonts.rs` covers real CJK glyphs, pixel advances, outline alpha/clipping, cache
eviction, mixed-width input/selection and composition caret positions.

`forms.rs` covers activation cancellation, labels, radio grouping/navigation,
local FormData submission/reset, official Vue/React bindings and the settings app.

`flex.rs` validates independent flex geometry, growth and wrapping, hidden items,
alignment with nested text/clips, overflow failures, painting and resized hit tests.

Flex regressions also cover scaled shrink weights, zero-width freezing, fixed
edges/items, conservation across viewport sizes and narrow-width hit testing.

`html_runtime.rs` verifies the shared native parser: table/entity/comment/template
contexts, initial-versus-innerHTML recovery, foreign namespaces, raw text and
transactional rejection of byte/node/depth limits. `html.rs` includes adoption
agency/foster parenting and arena-link consistency.

`css_syntax.rs` checks shared parsing across inline/author styles, token boundaries,
comment semantics, EOF recovery, native cssText strings and transactional
byte/nesting-limit failures. Existing property and selector tests retain semantic
coverage without requiring the old splitter's behavior.
