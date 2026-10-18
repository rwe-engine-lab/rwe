# Framework compatibility and next milestones

RWE is an experimental renderer and desktop host, not a browser-compatible WebView.
Mature libraries take priority across all subsystems: see REUSE-AUDIT.md and AGENTS.md.
Flex solving uses Taffy; HTML document/fragment parsing uses html5ever through a shared native bridge.
CSS syntax and JS cssText now use cssparser; property semantics, selectors and cascade remain custom.
Block/text flow and other DOM behavior/serialization remain partly custom.
Passing the bundled applications verifies the following subsets, not arbitrary
Vue/React projects or npm component libraries.

| Area | Implemented and tested | Major missing capabilities |
| --- | --- | --- |
| Application loading | HTML5 parsing/recovery, contextual innerHTML, bounded local CSS/classic JS, embedded assets | ES modules, bundler manifest integration, development server |
| JavaScript | Boa, bounded jobs/timers, official Vue 3.5.13 and React 18.3.1 fixtures | Broad browser DOM/CSSOM, third-party component compatibility |
| Layout | Block flow, box model, clipping, wrapped text, explicit-basis flex rows with growth/shrink | Inline formatting, complete flexbox (intrinsic sizes/wrap), grid, positioning |
| Text | Bitmap ASCII; optional embedded CJK raster font; shared measurements | Font selection, shaping, bidi, full Unicode typography |
| Interaction | Focus, keyboard, buttons, single-line editing, IME protocol | Clipboard, undo, pointer selection, accessibility |
| Forms | Checkbox/radio, labels, local submit/reset, string FormData | Select/textarea/file, constraint validation, HTTP submission |
| Platform | macOS window host, embedded application bundle script | Windows/Linux host audit, signing, installers, update lifecycle |
| Services | Local immutable resource boundaries | fetch/XHR, HTTP, storage, application service APIs |

## Priorities

1. Layout foundation: build on the explicit-basis flex-row subset with intrinsic
   sizing/wrapping, then inline flow, with independent geometry fixtures
   and real navigation/settings/list applications as acceptance cases.
2. Application toolchain: a defined build-output contract for bundled Vue/React
   apps, module loading and useful source errors. Loading an arbitrary Vite project
   is not currently supported.
3. Application services: asynchronous HTTP and explicit storage APIs, including
   cancellation/error handling and resource limits.
4. Interaction completeness: select/textarea, clipboard/selection, validation and
   accessibility, verified against real framework-controlled components.
5. Delivery: platform testing, reproducible packaging/signing and diagnostics.

Each milestone should add public framework APIs, focused regression tests and
a frontend example that contains only business behavior. Examples must not grow
private copies of DOM, rendering, event or window-host implementations.

Automated IME tests exercise the public protocol; native candidate windows and
system input methods still require manual macOS verification. Screenshot exports
verify rendering but do not establish native window compatibility.
