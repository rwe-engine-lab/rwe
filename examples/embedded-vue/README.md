# Embedded Vue application

Run from the repository or this directory:

```sh
cargo run --features window,javascript --example embedded-vue
```

`main.rs` embeds the existing Vue counter HTML, CSS, application JavaScript and
Vue production runtime with `include_str!`. `AssetBundle` validates those assets;
`DesktopApp::from_assets` owns loading, JavaScript, window events and repainting.
There is no runtime dependency on the repository, current directory or network.
The same API supports static HTML and bundled classic React applications.

Build a movable macOS application from the repository:

```sh
sh scripts/bundle-embedded-macos.sh
open target/RWE-Embedded-Vue.app
```

The script creates an unsigned development app, with dependency and Vue license
notices. It requires already downloaded Cargo dependencies. Copy the whole `.app`
to distribute it locally. Signing/notarization are not provided by this script.

Embedded assets are immutable. Use `DesktopApp::new(path).watch(true)` for file
editing; rebuild an embedded application after changing its assets. External
stylesheet overrides and file watching are rejected for embedded startup. Dropping
an HTML file into the running window switches to the normal file development mode.

Asset paths resolve relative to the entry page within the virtual bundle root.
Each source/combined CSS is limited to 1 MiB; the bundle and loaded page each have
an 8 MiB budget. At most 256 assets and 32 page resource references are accepted.
Missing assets never fall back to filesystem paths. Existing DOM/CSS/JavaScript
compatibility limits still apply: this feature does not add browser APIs or modules.
