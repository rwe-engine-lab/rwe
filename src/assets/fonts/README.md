# Embedded RWE CJK font

Asset: `RWECJK-Regular.ttf` (OFL-1.1), a renamed static regular-weight subset of
Noto Sans SC. Original copyright and complete license: `OFL.txt`. The derivative
uses the family name **RWE CJK** and does not use the reserved font name **Source**.
This font-specific license does not replace RWE's MIT source-code license.

Upstream: https://github.com/google/fonts/tree/a85815a42757630ce188fdad368c2dfc444d4773/ofl/notosanssc

- Revision: `a85815a42757630ce188fdad368c2dfc444d4773`.
- Input: `NotoSansSC[wght].ttf`, instantiated at `wght=400`.
- Input SHA-256: `a3041811a78c361b1de50f953c805e0244951c21c5bd412f7232ef0d899af0da`.
- Derived SHA-256: `31a37745b39970ce88a15271ccb24c64c79c18da9b62bb5d061bd485e04cc363`.
- Size: 9,842,472 bytes; 28,313 mapped Unicode scalars.
- Retained ranges (where upstream has glyphs): U+0020–024F, U+2000–206F,
  U+3000–30FF, U+3400–9FFF, U+FF00–FFEF. Includes common simplified/traditional
  Chinese, CJK Extension A, punctuation and some non-ASCII Latin; not all Unicode.
- Hinting, vertical-layout and shaping tables are removed. This release does not
  implement shaping or font-family/weight selection; ASCII uses RWE's original
  bitmap face, and unsupported characters retain the visible fallback box.

Regenerate using Python 3.11+ with FontTools 4.67.0 (pure Python):

```sh
python tools/subset-font.py path/to/NotoSansSC\[wght\].ttf src/assets/fonts/RWECJK-Regular.ttf
```

Regeneration is a separate developer task. Cargo does not run Python, download
fonts or use system font discovery. See FONT-DEPENDENCIES.md for the dependency
review. The font is embedded only with the optional `fonts` feature.
