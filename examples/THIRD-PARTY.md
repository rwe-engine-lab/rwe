# Pinned application-side framework fixtures

These are unmodified official npm distribution builds. They are executed as classic scripts by the experimental RWE host; they are not replacements, mocks or renderer dependencies. No CDN is contacted at runtime. Vue runtime/renderer/compiler and React/ReactDOM UMD distribution are JavaScript, licensed MIT. The distributed ReactDOM bundle includes React's scheduler, also under the React project MIT license. License texts and original copyright banners are retained.

| File | Official source | SHA-256 |
| --- | --- | --- |
| vue.global.prod.js | https://unpkg.com/vue@3.5.13/dist/vue.global.prod.js | c459ba7cc8db65c982589fa5d64c7ff478877e8e5b0fd75683207cec6a4e89e8 |
| react.production.min.js | https://unpkg.com/react@18.3.1/umd/react.production.min.js | d949f1c3687aedadcedac85261865f29b17cd273997e7f6b2bfc53b2f9d4c4dd |
| react-dom.production.min.js | https://unpkg.com/react-dom@18.3.1/umd/react-dom.production.min.js | 35f4f974f4b2bcd44da73963347f8952e341f83909e4498227d4e26b98f66f0d |

Vue files are copied unchanged into vue-counter/vendor, vue-template/vendor and settings-form/vendor.
React/ReactDOM files are in react-counter/vendor. Each application keeps its
resources inside its own HTML directory to satisfy the local resource boundary.
