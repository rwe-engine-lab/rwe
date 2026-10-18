# macOS window dependency review

Scope: `aarch64-apple-darwin`, `window` feature, winit 0.30.13 (`rwh_06` only), softbuffer 0.4.8 (no default features). Other targets are not approved by this review. The default renderer has no third-party dependencies.

Reviewed the recursively resolved Cargo metadata, activated features, SPDX license declarations and bundled license files where present, implementation source extensions, and build scripts (including build dependencies and procedural macros). All selected implementations and macros are Rust; licenses are permissive as listed below. Dependency internals contain unsafe platform FFI; RWE's own unsafe-code prohibition remains enabled. This is a source/language/license review, not a security audit.

`objc-sys` contains `extern/exception.m`, but its `unstable-exception` feature and `cc` build dependency are absent from the selected graph. The file is not compiled. Enabling additional features requires another review. Remaining build scripts configure Rust/platform linking or probe rustc, not third-party C/C++ builds. `foreign-types-macros` is a Rust procedural macro; its syn/quote/proc-macro2/unicode-ident chain is included.

AppKit, CoreGraphics, QuartzCore, Foundation, IOSurface, Objective-C runtime, libSystem and the platform SDK/linker remain external host/toolchain boundaries. No WebView or external browser engine is used. Pin direct dependencies and retain Cargo.lock; changes to resolved versions or features require review. Cargo.lock also records inactive packages for other platforms; those are not compiled or approved here. This review covers the selected Apple Silicon target; Intel macOS is not tested.

| Package | Version | SPDX license expression |
| --- | --- | --- |
| bitflags | 1.3.2 | MIT/Apache-2.0 |
| bitflags | 2.13.2 | MIT OR Apache-2.0 |
| block2 | 0.5.1 | MIT |
| cfg_aliases | 0.2.2 | MIT |
| core-foundation | 0.9.4 | MIT OR Apache-2.0 |
| core-foundation-sys | 0.8.7 | MIT OR Apache-2.0 |
| core-graphics | 0.23.2 | MIT OR Apache-2.0 |
| core-graphics-types | 0.1.3 | MIT OR Apache-2.0 |
| cursor-icon | 1.2.0 | MIT OR Apache-2.0 OR Zlib |
| dispatch | 0.2.0 | MIT |
| dispatch2 | 0.3.1 | Zlib OR Apache-2.0 OR MIT |
| dpi | 0.1.2 | Apache-2.0 AND MIT |
| foreign-types | 0.5.0 | MIT/Apache-2.0 |
| foreign-types-macros | 0.2.4 | MIT/Apache-2.0 |
| foreign-types-shared | 0.3.1 | MIT/Apache-2.0 |
| libc | 0.2.191 | MIT OR Apache-2.0 |
| objc-sys | 0.3.5 | MIT |
| objc2 | 0.5.3 | MIT |
| objc2 | 0.6.5 | MIT |
| objc2-app-kit | 0.2.2 | MIT |
| objc2-core-data | 0.2.2 | MIT |
| objc2-core-foundation | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-core-graphics | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-core-image | 0.2.2 | MIT |
| objc2-encode | 4.1.0 | MIT |
| objc2-foundation | 0.2.2 | MIT |
| objc2-foundation | 0.3.2 | MIT |
| objc2-io-surface | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| objc2-metal | 0.2.2 | MIT |
| objc2-quartz-core | 0.2.2 | MIT |
| objc2-quartz-core | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| pin-project-lite | 0.2.17 | Apache-2.0 OR MIT |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 |
| quote | 1.0.47 | MIT OR Apache-2.0 |
| raw-window-handle | 0.6.2 | MIT OR Apache-2.0 OR Zlib |
| serde | 1.0.229 | MIT OR Apache-2.0 |
| serde_core | 1.0.229 | MIT OR Apache-2.0 |
| smol_str | 0.2.2 | MIT OR Apache-2.0 |
| softbuffer | 0.4.8 | MIT OR Apache-2.0 |
| syn | 3.0.9 | MIT OR Apache-2.0 |
| tracing | 0.1.44 | MIT |
| tracing-core | 0.1.36 | MIT |
| unicode-ident | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 |
| winit | 0.30.13 | Apache-2.0 |

Some objc2-family and dispatch crate archives omit repository-level license files; their SPDX declarations were checked in Cargo manifests. Upstream license references: https://github.com/madsmtm/objc2/blob/master/LICENSE.md and https://github.com/SSheldon/rust-dispatch (MIT). This local development bundle is not a signed/public release.
