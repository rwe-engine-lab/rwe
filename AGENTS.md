# Framework engineering policy

## Reuse established components

- Before extending any foundational subsystem, evaluate maintained libraries and
  existing engine integrations. Prefer a suitable established implementation to
  another RWE implementation of the same algorithm or protocol.
- This applies to ALL modules: parsing, DOM/CSS, layout, text/editing, drawing,
  codecs, JavaScript, loading/networking, storage, windows, watching, diagnostics,
  CLI, accessibility and distribution. It is not limited to layout.
- Zero dependencies and implementing everything in-house are NOT project goals.
- RWE should own its application API, resource limits, capability policy,
  lifecycle, adaptation between libraries and business-facing integration.
- Keep adapters thin. Delete superseded algorithms instead of keeping a parallel
  hand-written fallback that silently becomes the default.
- Evaluate functional fit, maintenance, licensing, transitive/build dependencies,
  MSRV, platforms and operational cost. Candidate names in the audit are not
  evidence of integration, compatibility, or a completed dependency review.
- Update REUSE-AUDIT.md with adopted/pending/rejected status and evidence. When
  retaining custom foundation code, record the specific missing fit and a bounded
  scope; do not use "small", "pure Rust" or "educational" alone as justification.
- Verify library boundaries with observable application/pixel/interaction tests.
  Do not require the byte representation or rounding of an obsolete algorithm.
  Preserve error propagation, resource limits and documented API contracts;
  document intentional compatibility changes.
- Examples contain startup configuration and frontend business behavior only.
  Reusable rendering, DOM, control and event logic belongs in the framework or
  adopted libraries, never private copies in each example.
- RWE source continues to forbid unsafe code. Dependencies may use platform FFI
  or optimized internals; review actual enabled features and record their boundary.
