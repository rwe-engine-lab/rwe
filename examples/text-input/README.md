# Text input

```sh
cargo run --features window,javascript --example text-input
```

`main.rs` only embeds resources and starts `DesktopApp`; `app.js` owns greeting
and clear-button behavior. Editing, selection, input events, clipping and painting
are framework features. This app can be moved without the source directory.

Tab/Shift+Tab focus text fields and buttons. Type to edit; Backspace/Delete remove
characters; Left/Right/Home/End move the caret; Shift extends selection;
Control/Command+A selects all. Enter or leaving the field commits a change.
The read-only field accepts focus/selection but cannot be edited.

```sh
sh scripts/bundle-embedded-macos.sh target/RWE-Text-Input.app text-input
```

Current support: single-line `input` with absent type, `type=text` or `type=search`,
value/placeholder/maxlength/readonly/disabled, beforeinput/input/change, and a
nonblinking caret. Clicking places the caret at the end. Long values scroll within
the content box. Native IME preedit/commit/cancel and composition events are supported
experimentally; clipboard, undo, pointer selection, multiline textarea, other
input types, form submission and system accessibility are not implemented.

Unicode editing respects code-point boundaries for surrogate pairs, not full
extended grapheme clusters. The bitmap font still has its existing glyph limits.

The native host enables IME only while an editable field has focus, anchors the
candidate window near the caret and cancels preedit on blur. Preedit has an underline
and does not change the greeting/model until committed. The status text reports
composition events. Manual native IME verification remains outstanding; the bitmap
default font does not include Chinese glyphs; enable `fonts` or use `../chinese-input/`. See APPLICATIONS.md for protocol limits.
