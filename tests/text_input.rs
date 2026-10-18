//! Shared input behavior and real Vue/React controlled forms.
#![cfg(feature = "javascript")]
use rwe::{Application, AssetBundle, KeyEvent, NodeKind, RenderOptions};
fn app(html: &str) -> Application {
    Application::from_assets(&AssetBundle::new("index.html", &[("index.html", html)]).unwrap())
        .unwrap()
}
fn value(app: &Application, id: &str) -> String {
    app.document()
        .nodes()
        .iter()
        .find_map(|n| match &n.kind {
            NodeKind::Element { attributes, .. }
                if attributes.get("id").is_some_and(|v| v == id) =>
            {
                attributes.get("value").cloned()
            }
            _ => None,
        })
        .unwrap()
}
fn key(app: &mut Application, key: &str) {
    assert!(app.keyboard(&KeyEvent::down(key)).unwrap());
}
fn all(app: &Application) -> String {
    app.document().text_content(0).unwrap()
}

#[test]
fn unicode_editing_selection_replacement_and_maxlength_are_bounded() {
    let mut app = app("<input id=name maxlength=6><button>NEXT</button>");
    key(&mut app, "Tab");
    assert!(app.insert_text("A😀BC").unwrap());
    assert_eq!(value(&app, "name"), "A😀BC");
    key(&mut app, "ArrowLeft");
    key(&mut app, "Backspace");
    assert_eq!(value(&app, "name"), "A😀C");
    key(&mut app, "Backspace");
    assert_eq!(value(&app, "name"), "AC");
    let mut shift = KeyEvent::down("ArrowLeft");
    shift.shift_key = true;
    app.keyboard(&shift).unwrap();
    app.insert_text("Z").unwrap();
    assert_eq!(value(&app, "name"), "ZC");
    let mut select = KeyEvent::down("a");
    select.meta_key = true;
    app.keyboard(&select).unwrap();
    app.insert_text("😀😀😀😀").unwrap();
    assert_eq!(value(&app, "name"), "😀😀😀");
    key(&mut app, "Home");
    key(&mut app, "Delete");
    assert_eq!(value(&app, "name"), "😀😀");
    for bad in ["hello\nworld", "\0", &"a".repeat(262145)] {
        assert!(app.insert_text(bad).is_err());
    }
}

#[test]
fn beforeinput_can_cancel_and_change_commits_once_on_enter_or_blur() {
    let mut app = app(
        "<input id=name><button>NEXT</button><p id=log></p><script>const n=document.getElementById('name'),log=document.getElementById('log');n.addEventListener('beforeinput',e=>{if(e.data==='x')e.preventDefault();});n.addEventListener('input',e=>{e.preventDefault();log.textContent+='input:'+e.inputType+';';});n.addEventListener('change',()=>log.textContent+='change;');</script>",
    );
    key(&mut app, "Tab");
    key(&mut app, "x");
    assert_eq!(value(&app, "name"), "");
    key(&mut app, "r");
    key(&mut app, "w");
    assert_eq!(value(&app, "name"), "rw");
    key(&mut app, "Enter");
    key(&mut app, "Tab");
    assert_eq!(all(&app).matches("change;").count(), 1);
    app.click("name").unwrap();
    app.insert_text("!").unwrap();
    app.blur().unwrap();
    assert_eq!(all(&app).matches("change;").count(), 2);
    assert!(all(&app).contains("input:insertText;"));
}

#[test]
fn readonly_disabled_key_cancellation_and_native_text_do_not_leak_shortcuts() {
    let mut app = app(
        "<input id=disabled disabled><input id=readonly readonly value=KEEP><input id=name><script>document.getElementById('name').onkeydown=e=>{if(e.key==='q')e.preventDefault();};</script>",
    );
    key(&mut app, "Tab");
    key(&mut app, "r");
    key(&mut app, "Backspace");
    assert_eq!(value(&app, "readonly"), "KEEP");
    assert!(!app.insert_text("x").unwrap());
    key(&mut app, "Tab");
    key(&mut app, "q");
    assert_eq!(value(&app, "name"), "");
    let mut native = KeyEvent::down("e");
    native.text = Some("é".into());
    app.keyboard(&native).unwrap();
    assert_eq!(value(&app, "name"), "é");
    native.text = Some(String::new());
    app.keyboard(&native).unwrap();
    assert_eq!(value(&app, "name"), "é");
    assert!(!app.keyboard(&KeyEvent::up("Shift")).unwrap());
}

#[test]
fn static_and_live_inputs_paint_values_placeholders_caret_and_selection() {
    let mut app = app(
        "<input id=name value=AB placeholder=NAME style='width: 60px; padding: 4px; border: 1px solid #888'><input id=empty placeholder=EMPTY>",
    );
    let options = RenderOptions {
        width: 100,
        ..Default::default()
    };
    let before = app.render(options).unwrap();
    key(&mut app, "Tab");
    let after = app.render(options).unwrap();
    assert_ne!(before.pixel(29, 5), after.pixel(29, 5));
    let mut select = KeyEvent::down("a");
    select.ctrl_key = true;
    app.keyboard(&select).unwrap();
    let selected = app.render(options).unwrap();
    assert_eq!(
        selected.pixel(6, 5),
        Some(rwe::Color::rgba(191, 219, 254, 255))
    );
    let static_app = Application::from_html("<input value=AB>", "").unwrap();
    assert_eq!(static_app.layout(100).unwrap().text_runs()[0].text, "AB");
    assert_eq!(
        app.layout(100).unwrap().text_runs().last().unwrap().text,
        "EMPTY"
    );
    let hidden =
        Application::from_html("<input type=hidden value=SECRET><input value=VISIBLE>", "")
            .unwrap();
    assert!(
        !hidden
            .layout(100)
            .unwrap()
            .text_runs()
            .iter()
            .any(|r| r.text.contains("SECRET"))
    );
}

#[test]
fn actual_vue_v_model_and_react_onchange_update_after_native_editing() {
    for (framework, source) in [
        (
            "vue-counter",
            "Vue.createApp({data(){return {name:''};},template:'<section><input id=field v-model=\"name\"><p id=result>VALUE: {{name}}</p></section>'}).mount('#app');",
        ),
        (
            "react-counter",
            "function App(){const [name,set]=React.useState('');return React.createElement('section',null,React.createElement('input',{id:'field',value:name,onChange:e=>set(e.target.value)}),React.createElement('p',{id:'result'},'VALUE: '+name));}ReactDOM.createRoot(document.getElementById('app')).render(React.createElement(App));",
        ),
    ] {
        let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join(framework);
        let files = if framework == "vue-counter" {
            vec!["vendor/vue.global.prod.js"]
        } else {
            vec![
                "vendor/react.production.min.js",
                "vendor/react-dom.production.min.js",
            ]
        };
        let texts: Vec<_> = files
            .iter()
            .map(|f| std::fs::read_to_string(base.join(f)).unwrap())
            .collect();
        let html = format!(
            "<div id=app></div>{}<script src=app.js></script>",
            files
                .iter()
                .map(|f| format!("<script src='{f}'></script>"))
                .collect::<String>()
        );
        let mut entries = vec![("index.html", html.as_str()), ("app.js", source)];
        entries.extend(files.iter().zip(&texts).map(|(f, t)| (*f, t.as_str())));
        let mut app = Application::from_assets(&AssetBundle::new("index.html", &entries).unwrap())
            .unwrap_or_else(|e| panic!("{framework}: {e}"));
        key(&mut app, "Tab");
        app.insert_text("Rust").unwrap();
        assert_eq!(value(&app, "field"), "Rust");
        assert!(
            all(&app).contains("VALUE: Rust"),
            "{framework}: {}",
            all(&app)
        );
        key(&mut app, "Backspace");
        assert!(all(&app).contains("VALUE: Rus"));
    }
}

#[test]
fn property_assignment_has_no_input_or_change_and_empty_commit_is_a_noop() {
    let mut app = app(
        "<input id=name><p id=log></p><script>const n=document.getElementById('name');n.onfocus=()=>{n.value='SCRIPT';n.select();};n.onchange=()=>document.getElementById('log').textContent='UNEXPECTED';n.oninput=()=>document.getElementById('log').textContent='UNEXPECTED';</script>",
    );
    key(&mut app, "Tab");
    assert!(!app.insert_text("").unwrap());
    assert_eq!(value(&app, "name"), "SCRIPT");
    app.blur().unwrap();
    assert!(!all(&app).contains("UNEXPECTED"));
}
