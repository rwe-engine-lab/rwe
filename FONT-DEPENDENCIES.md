# Optional CJK font dependency review

The `fonts` feature uses fontdue 0.9.4 with default features disabled and `std`
enabled. Reviewed the recursive resolved source manifests and build scripts before
adoption. Selected implementation crates are Rust with permissive licenses:

| Crate | Version | SPDX |
| --- | --- | --- |
| fontdue | 0.9.4 | MIT OR Apache-2.0 OR Zlib |
| ttf-parser | 0.25.1 | MIT OR Apache-2.0 |
| core_maths | 0.1.1 | MIT |
| libm | 0.2.16 | MIT |

libm's Rust build/configure scripts emit target/compiler cfg settings and do not
build/link a native math library. ttf-parser ships C++/Qt comparison tools under
`testing-tools/`; these are not Cargo dependencies/build targets and are not built
or adopted. No selected native implementation or native build step is used.
fontdue includes internal unsafe Rust; RWE's own unsafe prohibition remains intact.
This is a language/license review, not a security audit or arbitrary-font sandbox.

The embedded font is an OFL-1.1 font asset, not Rust code or a crate dependency.
It is a renamed/static subset of Noto Sans SC from Google Fonts. Original copyright
and the complete font-specific license are distributed in `src/assets/fonts/`.
The OFL applies to the font asset; RWE's source remains MIT. Generated images and
application source are not relicensed under the font license. The default renderer
continues to use only std and does not embed/parse the font unless fonts is enabled.

`tools/subset-font.py` is a separate developer regeneration tool using MIT-licensed
FontTools 4.67.0 in pure Python mode. It is not invoked by Cargo or at runtime.
Font source revision, hashes, subset coverage and regeneration instructions are in
`src/assets/fonts/README.md`. Other versions/features/targets require a fresh review.
