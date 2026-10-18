# Settings form

```sh
cargo run --features window,javascript,fonts --example settings-form
```

The thin Rust entry embeds assets and starts DesktopApp. Vue owns application
state; the framework owns windows, rendering, editing, focus, checkbox/radio
activation, labels and form events. Submit previews string FormData locally;
reset updates controlled Vue state. No HTTP request or persistence is performed.
The pinned Vue distribution and its MIT notice are in vendor/.

Choice rows and equal-width action buttons use the framework's explicit-basis
flex layout. Resize the window to see free space redistributed without application
geometry code. Buttons have a 180px content basis and shrink at narrow widths; controls opt out.
Intrinsic sizing and flex wrapping remain unsupported.
