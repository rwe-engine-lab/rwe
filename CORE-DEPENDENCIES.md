# Default HTML, CSS, layout and PNG dependencies

The default renderer adopts cssparser 0.38.0 (MPL-2.0), html5ever 0.40.1 (MIT OR Apache-2.0), Taffy 0.15.0 (MIT) and png 0.18.1
(MIT OR Apache-2.0), pinned in Cargo.toml/Cargo.lock. This replaces the former
zero-dependency policy. Source manifests, enabled dependency tree, build scripts
and license files were inspected; this is not a full security or source-code audit.

Taffy enables only std, taffy_tree and flexbox, with defaults disabled. Grid,
block/float layout, CSS parsing and serde features are not enabled or exposed.
RWE delegates flex solving and cross alignment; full-tree/block/text integration
remains work. png defaults are disabled; its active flate2 backend is Rust
miniz_oxide. No native libpng/zlib is compiled. Lockfile-only optional packages
are not evidence that their code is enabled.

| Crate | Locked version | SPDX from manifest |
| --- | --- | --- |
| adler2 | 2.0.1 | 0BSD OR MIT OR Apache-2.0 |
| bitflags | 2.13.2 | MIT OR Apache-2.0 |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 |
| crc32fast | 1.5.2 | MIT OR Apache-2.0 |
| cssparser | 0.38.0 | MPL-2.0 |
| dtoa | 1.0.11 | MIT OR Apache-2.0 |
| dtoa-short | 0.3.5 | MPL-2.0 |
| fastrand | 2.5.0 | Apache-2.0 OR MIT |
| fdeflate | 0.3.7 | MIT OR Apache-2.0 |
| flate2 | 1.1.10 | MIT OR Apache-2.0 |
| html5ever | 0.40.1 | MIT OR Apache-2.0 |
| itoa | 1.0.18 | MIT OR Apache-2.0 |
| libc | 0.2.191 | MIT OR Apache-2.0 |
| lock_api | 0.4.14 | MIT OR Apache-2.0 |
| log | 0.4.34 | MIT OR Apache-2.0 |
| markup5ever | 0.40.0 | MIT OR Apache-2.0 |
| memchr | 2.8.3 | Unlicense OR MIT |
| miniz_oxide | 0.8.9 | MIT OR Zlib OR Apache-2.0 |
| miniz_oxide | 0.9.1 | MIT OR Zlib OR Apache-2.0 |
| new_debug_unreachable | 1.0.6 | MIT |
| parking_lot | 0.12.5 | MIT OR Apache-2.0 |
| parking_lot_core | 0.9.12 | MIT OR Apache-2.0 |
| phf | 0.14.0 | MIT |
| phf_codegen | 0.14.0 | MIT |
| phf_generator | 0.14.0 | MIT |
| phf_shared | 0.14.0 | MIT |
| png | 0.18.1 | MIT OR Apache-2.0 |
| precomputed-hash | 0.1.1 | MIT |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 |
| quote | 1.0.47 | MIT OR Apache-2.0 |
| scopeguard | 1.2.0 | MIT OR Apache-2.0 |
| simd-adler32 | 0.3.10 | MIT |
| siphasher | 1.0.4 | MIT OR Apache-2.0 |
| slotmap | 1.1.1 | Zlib |
| smallvec | 1.16.3 | MIT OR Apache-2.0 |
| string_cache | 0.11.0 | MIT OR Apache-2.0 |
| string_cache_codegen | 0.11.2 | MIT OR Apache-2.0 |
| taffy | 0.15.0 | MIT |
| tendril | 0.5.1 | MIT OR Apache-2.0 |
| unicode-ident | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 |
| version_check | 0.9.5 | MIT/Apache-2.0 |
| web_atoms | 0.3.0 | MIT OR Apache-2.0 |

slotmap and crc32fast build scripts query rustc and set cfg flags; they do not
compile native sources. version_check is a Rust build dependency. Enabled source
implementations are Rust. SIMD/CPU dispatch and slotmap internals may use unsafe;
RWE's own unsafe_code=forbid does not make claims about dependency internals.
Upstream license texts are included under licenses/core/ and copied into desktop
bundles. Both miniz_oxide versions are resolved by png/flate2; do not hand-edit the
lockfile to remove one. Optional JS/window/font reviews remain separate.

Taffy uses f32; current adapter geometry is bounded to exact integer range 2^24
and rejects negative geometry. PNG output is compressed; encoded bytes are not
stable, while decoded RGBA and caller IO errors remain the compatibility contract.

Reproduce the active default closure with:

```sh
cargo tree --locked --offline --no-default-features --edges normal,build
```

Upstream: https://github.com/DioxusLabs/taffy and https://github.com/image-rs/image-png

The taffy crate archive omits its LICENSE file. The bundled MIT license was
retrieved from the exact published VCS commit recorded in .cargo_vcs_info.json:
https://raw.githubusercontent.com/DioxusLabs/taffy/eb6450694f33e2b7413e40677f230ff63f0fc194/LICENSE
Other texts were copied from their resolved crate archives.

## HTML5 integration

html5ever defaults are disabled. Its markup5ever/tendril/web_atoms stack provides
HTML tokens, names and entity tables. web_atoms generates Rust tables at build
time via phf_codegen/string_cache_codegen; proc-macro2/quote probe compiler
features; parking_lot_core sets sanitizer cfgs; libc sets target cfgs. Manifests,
build-script source and licenses were inspected. This is not a full security audit.
The selected implementations/build tooling are Rust, with OS-locking FFI through
parking_lot/libc on this macOS target. Other platform closures require their own
validation. No HTML parser C/C++ library is compiled.

RWE implements only the arena TreeSink and public projection, with byte limits,
construction allocation/depth guards, 1 KiB UTF-8 input chunks and final exact
output limits. It delegates HTML tokenization/recovery/entities to the library.
The unsupported markup5ever_rcdom test DOM is not a dependency.

The optional JavaScript feature now directly uses serde_json 1.0.154 (already
in the Boa closure) for string/record conversion; native values are built with
Boa's safe APIs. There are no new unsafe closures or parser callbacks in RWE.

## Shared CSS syntax

cssparser 0.38.0 is pinned with default features disabled; optional fast-match
macros/color tables, serde and malloc_size_of are not enabled. New default crates
are cssparser, dtoa-short 0.3.5 and dtoa 1.0.11 (itoa already existed). The new
manifests declare no build scripts and compile Rust implementations.

cssparser and dtoa-short are MPL-2.0, unlike the permissive licenses of many other
dependencies. This distinction is recorded explicitly; these crates have not
been relabeled MIT. Their original license texts, exact unmodified source archives,
source URLs and archive hashes are under licenses/core/ and included in bundles.
RWE's own source license remains MIT. No copies of their algorithms were added
to RWE's source; the adapter uses public parser/token/serializer traits.

The adapter delegates rules, declarations, comments, escaping, strings and nested
blocks. RWE still validates its supported properties, rejects unsupported rules
and uses its existing selector matcher/cascade. JS style.cssText uses the same
native syntax path and prepares a new declaration list before replacing values.
Inputs are bounded to 1 MiB; cssparser's default nested-block limit is retained.
This is syntax migration, not adoption of Stylo or complete browser CSSOM.
