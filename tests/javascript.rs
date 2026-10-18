//! Official framework execution and state update regressions.
#![cfg(feature = "javascript")]
use rwe::{LocalPage, ScriptRuntime};
use std::path::Path;

fn counter(name: &str, status: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "examples/{}/index.html",
        match name {
            "vue" => "vue-counter",
            "react" => "react-counter",
            other => other,
        }
    ));
    let page = LocalPage::load(&path, None).unwrap();
    let mut runtime = ScriptRuntime::load(&page).unwrap_or_else(|e| panic!("{name}: {e}"));
    assert!(
        runtime
            .snapshot()
            .unwrap()
            .text_content(0)
            .unwrap()
            .contains("COUNT: 0")
    );
    runtime.click("increment").unwrap();
    let text = runtime.snapshot().unwrap().text_content(0).unwrap();
    assert!(text.contains("COUNT: 1"), "{text}");
    assert!(text.contains(status), "{text}");
    runtime.click("increment").unwrap();
    assert!(
        runtime
            .snapshot()
            .unwrap()
            .text_content(0)
            .unwrap()
            .contains("COUNT: 2")
    );
}
#[test]
fn official_vue_mounts_and_updates_after_clicks() {
    counter("vue", "VUE REACTIVITY WORKS");
}
#[test]
fn official_react_mounts_and_updates_after_clicks() {
    counter("react", "REACT STATE WORKS");
}

#[test]
fn official_vue_template_compiler_and_list_updates_work() {
    counter("vue-template", "VUE REACTIVITY WORKS");
}

fn local_runtime(html: &str) -> Result<ScriptRuntime, String> {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "rwe-runtime-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&dir).unwrap();
    let path = dir.join("page.html");
    std::fs::write(&path, html).unwrap();
    let result = LocalPage::load(&path, None).and_then(|page| ScriptRuntime::load(&page));
    std::fs::remove_dir_all(dir).unwrap();
    result
}

#[test]
fn event_capture_bubbling_and_promise_updates_use_the_live_dom() {
    let mut runtime = local_runtime(r#"<div id='outer'><button id='increment'>READY</button></div><script>
      const order = [];
      document.addEventListener('click', () => order.push('document'), true);
      const outer = document.getElementById('outer');
      const button = document.getElementById('increment');
      outer.addEventListener('click', () => order.push('capture'), true);
      button.addEventListener('click', () => order.push('target'));
      outer.addEventListener('click', () => { order.push('bubble'); Promise.resolve().then(() => button.textContent = order.join('/')); });
    </script>"#).unwrap();
    runtime.click("increment").unwrap();
    assert_eq!(
        runtime.snapshot().unwrap().text_content(0).unwrap(),
        "document/capture/target/bubble"
    );
    assert!(runtime.click("missing").is_err());
}

#[test]
fn delayed_timers_wait_for_time_instead_of_running_immediately() {
    let mut runtime = local_runtime(r#"<button id='increment'>WAIT</button><script>
      document.getElementById('increment').onclick = () => setTimeout(() => document.getElementById('increment').textContent = 'DONE', 30);
    </script>"#).unwrap();
    runtime.click("increment").unwrap();
    assert_eq!(runtime.snapshot().unwrap().text_content(0).unwrap(), "WAIT");
    assert!(runtime.has_timers().unwrap());
    std::thread::sleep(std::time::Duration::from_millis(40));
    runtime.pump().unwrap();
    assert_eq!(runtime.snapshot().unwrap().text_content(0).unwrap(), "DONE");
    assert!(!runtime.has_timers().unwrap());
}

#[test]
fn unsupported_modules_and_script_errors_are_explicit() {
    assert!(
        local_runtime("<script type=module>export default 1;</script>")
            .err()
            .unwrap()
            .contains("ES modules")
    );
    assert!(
        local_runtime("<script>throw new Error('TEST_FAILURE')</script>")
            .err()
            .unwrap()
            .contains("TEST_FAILURE")
    );
    assert!(
        local_runtime("<script>console.error('REPORTED_FAILURE')</script>")
            .err()
            .unwrap()
            .contains("REPORTED_FAILURE")
    );
    assert!(
        local_runtime(
            "<script>function repeat(){Promise.resolve().then(repeat)}repeat();</script>"
        )
        .err()
        .unwrap()
        .contains("budget")
    );
}
