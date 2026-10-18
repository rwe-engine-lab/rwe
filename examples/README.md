# Standalone application examples

Each directory is one application with a thin Rust bootstrap and frontend assets.
The bootstrap calls RWE; it does not implement the window or rendering engine.
These are Cargo examples registered in the repository's Cargo.toml, not separate
Cargo packages. Run commands from the repository root.

| Directory | Run |
| --- | --- |
| static-html/ | `cargo run --features window --example static-html` |
| vue-counter/ | `cargo run --features window,javascript --example vue-counter` |
| vue-template/ | `cargo run --features window,javascript --example vue-template` |
| react-counter/ | `cargo run --features window,javascript --example react-counter` |

```text
vue-counter/
├── main.rs       Application bootstrap: DesktopApp::new(entry).watch(true).run()
├── index.html    Application document
├── style.css     Application styling
├── app.js        Vue component and state
├── vendor/       Official local Vue package and license
└── README.md     Run instructions
```

RWE implements resource loading, JavaScript/DOM, window creation, input dispatch,
layout, painting, scrolling and reload. App bootstrap/configuration is legitimate
application code; copying the framework's runtime implementation is not required.
The shared launcher remains available for frontend-only workflows:

```sh
cargo run --features window,javascript --bin rwe-window -- examples/vue-counter/index.html --watch
cargo run -- render examples/static-html/index.html --width 480 --format png > preview.png
```

Framework assertions/fixtures live in `tests/`. Batch snapshots remain a separate
developer binary under `tools/`, not an application example. Pinned package
sources, hashes and licenses are documented in `THIRD-PARTY.md`.

`embedded-vue/` demonstrates a movable application with HTML/CSS/JavaScript embedded
in the executable. Run `cargo run --features window,javascript --example embedded-vue`.

`keyboard-buttons/` verifies shared native keyboard focus and button activation, with
all assets embedded. Run `cargo run --features window,javascript --example keyboard-buttons`.

`text-input/` verifies framework-owned single-line editing, selection and input events.
Run `cargo run --features window,javascript --example text-input`.

`chinese-input/` uses the optional `fonts` feature to verify actual Chinese glyphs,
input selection and IME with the same framework APIs. Run
`cargo run --features window,javascript,fonts --example chinese-input`.

设置表单案例：`cargo run --features window,javascript,fonts --example settings-form`。
能力范围与下一步：[兼容性清单](../COMPATIBILITY.md)。

`html5-recovery/` is a frontend-only HTML5 recovery/innerHTML acceptance case.
Run it with the framework rwe-window binary; no per-application Rust host is needed.
