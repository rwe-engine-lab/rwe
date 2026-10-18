# Keyboard buttons

Run from the repository or this directory:

```sh
cargo run --features window,javascript --example keyboard-buttons
```

The application embeds its HTML/CSS/JavaScript. `main.rs` only configures the
framework window. The framework owns focus traversal, button activation, key
routing, modifier state, focus painting, scrolling and cancellation on focus loss.
`app.js` contains application behavior: counter updates and an R shortcut.

- Tab / Shift+Tab moves between enabled buttons (disabled buttons are skipped).
- Enter activates the focused button once per press.
- Space activates on release; moving focus or leaving the window cancels it.
- The blue focus ring follows keyboard navigation. Clicking uses pointer focus.
- R resets the counter. `preventDefault()` suppresses the host's reload shortcut.

This is the current DOM keyboard subset, not full browser accessibility or text
editing. See `../text-input/` for text editing and experimental IME support.
Screen-reader integration and CSS `:focus` remain unsupported.

Build a movable unsigned macOS app:

```sh
sh scripts/bundle-embedded-macos.sh target/RWE-Keyboard.app keyboard-buttons
```
