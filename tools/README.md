# Development tools

`app-snapshots.rs` is a validation utility, not a sample application or framework
implementation. It loads the real example applications through public RWE APIs,
clicks counter buttons, exercises keyboard/text input and IME transitions, and saves
PNG previews without opening a window. IME previews include `ime-preedit.png` and
`ime-committed.png`; they verify the public protocol, not an actual system input method.

```sh
cargo run --locked --features javascript --bin rwe-app-snapshots -- /tmp/rwe-previews
```

Output files are created exclusively; existing files are not overwritten.
Automated assertions belong in `tests/`. Reusable runtime/render/export behavior
belongs in `src/`, and end-user application examples belong in `examples/`.

With `--features javascript,fonts`, the snapshot tool also emits Chinese initial,
preedit and committed-state PNGs from the real Chinese-input example.
`subset-font.py` is a separate, pinned FontTools developer tool; Cargo never runs it.

The fonts-enabled snapshots also cover the settings form before editing, after
local submission and after controlled-state reset.

`settings-narrow.png` exercises weighted flex shrinking and text reflow at 260px.
