# vue-counter

Run from the repository root:

```sh
cargo run --features window,javascript --example vue-counter
```

`main.rs` is the application bootstrap: it selects `index.html` and calls the
framework's `DesktopApp` API. HTML, styles and component code are application
content. Window creation, events, JavaScript/DOM, layout, painting and reload
remain in the framework. No runtime/render implementation is copied here.

The entry point also declares title, initial/minimum logical size and backdrop
through DesktopApp builder methods. These are application settings, not window
implementation. The same options are available in the shared launcher's CLI; see
APPLICATIONS.md for constraints and defaults.
