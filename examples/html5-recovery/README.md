# Shared HTML5 parsing

```sh
cargo run --features window,javascript,fonts --bin rwe-window -- examples/html5-recovery/index.html
```

The initial document deliberately omits closing paragraph/body/html tags. The
update button inserts another fragment with omitted paragraph endings and a
comment. Both use html5ever through the same framework parser. This example has
only HTML/CSS/JS business code and uses the framework's window entry directly.
It demonstrates HTML parsing, not complete CSS/DOM browser compatibility.
