//! Composition is transient until commit, including real framework bindings.
#![cfg(feature = "javascript")]
use rwe::{Application, AssetBundle, ImeEvent, KeyEvent, NodeKind, RenderOptions};
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
fn text(app: &Application) -> String {
    app.document().text_content(0).unwrap()
}
fn preedit(text: &str, cursor: Option<(usize, usize)>) -> ImeEvent {
    ImeEvent::Preedit {
        text: text.into(),
        cursor,
    }
}
fn focus(app: &mut Application) {
    assert!(app.keyboard(&KeyEvent::down("Tab")).unwrap());
}

#[test]
fn preedit_preview_uses_byte_offsets_preserves_value_and_commits_selection_once() {
    let mut app = app(
        "<input id=field value=AB style='width:120px'><p id=log></p><script>const n=document.getElementById('field'),log=document.getElementById('log');n.oninput=()=>log.textContent+='INPUT;';n.oncompositionstart=()=>log.textContent+='START;';n.oncompositionupdate=e=>log.textContent+='UPDATE:'+e.data+';';n.oncompositionend=e=>log.textContent+='END:'+e.data+';';</script>",
    );
    focus(&mut app);
    let mut shift = KeyEvent::down("ArrowLeft");
    shift.shift_key = true;
    app.keyboard(&shift).unwrap();
    app.ime(&preedit("你😀", Some((3, 7)))).unwrap();
    assert!(app.is_composing().unwrap());
    assert_eq!(value(&app, "field"), "AB");
    assert_eq!(app.layout(140).unwrap().text_runs()[0].text, "A你😀");
    assert_eq!(
        app.ime_cursor_rect(140).unwrap().unwrap().x,
        if cfg!(feature = "fonts") { 40 } else { 36 }
    );
    assert!(!text(&app).contains("INPUT;"));
    app.ime(&preedit("", None)).unwrap();
    assert!(app.is_composing().unwrap());
    app.ime(&ImeEvent::Commit("你好".into())).unwrap();
    assert_eq!(value(&app, "field"), "A你好");
    assert!(!app.is_composing().unwrap());
    assert_eq!(text(&app).matches("INPUT;").count(), 1);
    assert!(text(&app).contains("END:你好;INPUT;"));
}

#[test]
fn cancellation_escape_focus_loss_and_late_commits_never_edit_a_new_field() {
    let mut app = app("<input id=first value=OLD><input id=second>");
    focus(&mut app);
    app.ime(&preedit("draft", Some((5, 5)))).unwrap();
    assert!(app.keyboard(&KeyEvent::down("Enter")).unwrap());
    assert_eq!(value(&app, "first"), "OLD");
    assert!(app.keyboard(&KeyEvent::down("Escape")).unwrap());
    assert!(!app.is_composing().unwrap());
    assert!(!app.ime(&ImeEvent::Commit("late".into())).unwrap());
    app.ime(&preedit("next", Some((4, 4)))).unwrap();
    app.click("second").unwrap();
    assert!(!app.ime(&ImeEvent::Commit("late".into())).unwrap());
    assert_eq!(value(&app, "second"), "");
    app.ime(&preedit("é", Some((2, 2)))).unwrap();
    app.ime(&ImeEvent::Commit("é".into())).unwrap();
    assert_eq!(value(&app, "second"), "é");
    app.ime(&preedit("draft", None)).unwrap();
    app.blur().unwrap();
    assert!(!app.is_composing().unwrap());
    assert_eq!(value(&app, "first"), "OLD");
    assert_eq!(value(&app, "second"), "é");
    assert!(app.ime_cursor_rect(120).unwrap().is_none());
}

#[test]
fn readonly_disabled_and_invalid_preedit_ranges_are_rejected_without_state_changes() {
    let mut app = app("<input disabled><input id=field readonly value=SAFE>");
    focus(&mut app);
    assert!(!app.ime(&preedit("draft", None)).unwrap());
    assert!(app.ime_cursor_rect(120).unwrap().is_none());
    for event in [
        preedit("你", Some((1, 3))),
        preedit("é", Some((2, 1))),
        preedit("x", Some((0, 2))),
        preedit("\n", None),
        preedit(&"a".repeat(65537), None),
        ImeEvent::Commit("\0".into()),
    ] {
        assert!(app.ime(&event).is_err());
    }
    assert_eq!(value(&app, "field"), "SAFE");
    assert!(!app.is_composing().unwrap());
}

#[test]
fn commit_cancellation_maxlength_and_underlined_preview_obey_control_limits() {
    let mut app = app(
        "<input id=field maxlength=2 style='width:120px;padding:4px'><script>document.getElementById('field').onbeforeinput=e=>{if(e.data==='BLOCK')e.preventDefault();};</script>",
    );
    focus(&mut app);
    app.ime(&preedit("abc", Some((3, 3)))).unwrap();
    let canvas = app
        .render(RenderOptions {
            width: 140,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(canvas.pixel(10, 19), Some(rwe::Color::BLACK));
    app.ime(&ImeEvent::Commit("BLOCK".into())).unwrap();
    assert_eq!(value(&app, "field"), "");
    assert!(!app.is_composing().unwrap());
    app.ime(&preedit("😀x", None)).unwrap();
    app.ime(&ImeEvent::Commit("😀x".into())).unwrap();
    assert_eq!(value(&app, "field"), "😀");
}

#[test]
fn actual_vue_and_react_receive_composition_lifecycle_and_final_value() {
    for (framework, source) in [
        (
            "vue-counter",
            "Vue.createApp({data(){return {name:''};},template:'<section><input id=field v-model=\"name\"><p>VALUE: {{name}}</p></section>'}).mount('#app');",
        ),
        (
            "react-counter",
            "function App(){const [name,set]=React.useState('');const [status,ss]=React.useState('');return React.createElement('section',null,React.createElement('input',{id:'field',value:name,onChange:e=>set(e.target.value),onCompositionStart:()=>ss('STARTED'),onCompositionEnd:()=>ss('ENDED')}),React.createElement('p',null,'VALUE: '+name+'; '+status));}ReactDOM.createRoot(document.getElementById('app')).render(React.createElement(App));",
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
        let sources: Vec<_> = files
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
        entries.extend(files.iter().zip(&sources).map(|(f, t)| (*f, t.as_str())));
        let mut app =
            Application::from_assets(&AssetBundle::new("index.html", &entries).unwrap()).unwrap();
        focus(&mut app);
        app.ime(&preedit("nihao", Some((5, 5)))).unwrap();
        assert_eq!(value(&app, "field"), "");
        assert!(!text(&app).contains("VALUE: nihao"));
        if framework == "react-counter" {
            assert!(text(&app).contains("STARTED"));
        }
        app.ime(&preedit("", None)).unwrap();
        app.ime(&ImeEvent::Commit("你好".into())).unwrap();
        assert_eq!(value(&app, "field"), "你好");
        assert!(
            text(&app).contains("VALUE: 你好"),
            "{framework}: {}",
            text(&app)
        );
        if framework == "react-counter" {
            assert!(text(&app).contains("ENDED"));
        }
    }
}

#[test]
fn script_value_changes_cancel_preedit_without_rollback() {
    let mut app = app(
        "<input id=field value=OLD><button id=change>CHANGE</button><script>document.getElementById('field').onkeydown=e=>{if(e.key==='x')document.getElementById('field').value='SCRIPT';};</script>",
    );
    focus(&mut app);
    app.ime(&preedit("draft", None)).unwrap();
    app.keyboard(&KeyEvent::down("x")).unwrap();
    assert!(!app.is_composing().unwrap());
    assert_eq!(value(&app, "field"), "SCRIPTx");
    assert!(!app.ime(&ImeEvent::Commit("late".into())).unwrap());
    app.ime(&preedit("new", None)).unwrap();
    app.ime(&ImeEvent::Cancel).unwrap();
    assert_eq!(value(&app, "field"), "SCRIPTx");
}
