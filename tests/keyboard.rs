//! Public keyboard input, live frameworks and focus painting regressions.
#![cfg(feature = "javascript")]
use rwe::{Application, AssetBundle, Color, KeyEvent, NodeKind, RenderOptions};

fn app(html: &str, css: &str) -> Application {
    let html = format!("<link rel=stylesheet href=style.css>{html}");
    Application::from_assets(
        &AssetBundle::new("index.html", &[("index.html", &html), ("style.css", css)]).unwrap(),
    )
    .unwrap()
}
fn focused(app: &Application) -> Option<String> {
    app.document()
        .nodes()
        .iter()
        .find_map(|node| match &node.kind {
            NodeKind::Element { attributes, .. } if attributes.contains_key("data-rwe-focus") => {
                attributes.get("id").cloned()
            }
            _ => None,
        })
}
fn text(app: &Application) -> String {
    app.document().text_content(0).unwrap()
}
fn key(app: &mut Application, value: &str) {
    assert!(app.keyboard(&KeyEvent::down(value)).unwrap());
}

#[test]
fn tab_and_reverse_tab_follow_priority_skip_hidden_disabled_and_wrap() {
    let mut app = app(
        "<button id=normal>N</button><button id=disabled disabled>D</button><div hidden><button id=hidden>H</button></div><div class=gone><button id=css-hidden>H</button></div><button id=last tabindex=2>L</button><button id=first tabindex=1>F</button><button id=programmatic tabindex=-1>P</button><div inert><button id=inert>I</button></div>",
        ".gone { display: none; }",
    );
    for id in ["first", "last", "normal", "first"] {
        key(&mut app, "Tab");
        assert_eq!(focused(&app).as_deref(), Some(id));
    }
    let mut reverse = KeyEvent::down("Tab");
    reverse.shift_key = true;
    app.keyboard(&reverse).unwrap();
    assert_eq!(focused(&app).as_deref(), Some("normal"));
    app.blur().unwrap();
    app.keyboard(&reverse).unwrap();
    assert_eq!(focused(&app).as_deref(), Some("normal"));
}

#[test]
fn enter_activates_once_and_space_waits_for_release_and_cancels_on_blur() {
    let mut app = app(
        "<button id=go>0</button><button id=other>OTHER</button><script>let count=0;document.getElementById('go').onclick=()=>document.getElementById('go').textContent=String(++count);</script>",
        "",
    );
    key(&mut app, "Tab");
    key(&mut app, "Enter");
    let mut repeat = KeyEvent::down("Enter");
    repeat.repeat = true;
    app.keyboard(&repeat).unwrap();
    assert!(text(&app).starts_with('1'));
    key(&mut app, " ");
    assert!(text(&app).starts_with('1'));
    app.keyboard(&KeyEvent::up(" ")).unwrap();
    assert!(text(&app).starts_with('2'));
    key(&mut app, " ");
    key(&mut app, "Tab");
    app.keyboard(&KeyEvent::up(" ")).unwrap();
    assert!(text(&app).starts_with('2'));
    app.click("go").unwrap();
    assert!(text(&app).starts_with('3'));
    key(&mut app, " ");
    app.blur().unwrap();
    assert!(!app.keyboard(&KeyEvent::up(" ")).unwrap());
    assert!(text(&app).starts_with('3'));
    assert_eq!(focused(&app), None);
}

#[test]
fn focus_keyboard_events_bubble_and_prevention_blocks_defaults() {
    let mut app = app(
        r#"<button id=a>A</button><button id=b>B</button><p id=log></p><script>
      const log=document.getElementById('log');
      document.addEventListener('focusin',e=>log.textContent+='focus:'+e.target.id+';');
      document.addEventListener('focusout',e=>log.textContent+='blur:'+e.target.id+';');
      document.addEventListener('keydown',e=>{if(e.key==='Enter'||e.key==='Tab'&&e.shiftKey){e.preventDefault();log.textContent+='blocked;';}});
      document.getElementById('a').onclick=()=>log.textContent+='click;';
      document.addEventListener('keyup',e=>{if(e.key==='x'&&e.ctrlKey&&e.altKey&&e.metaKey&&e.repeat){e.preventDefault();log.textContent+='modifiers;';}});
    </script>"#,
        "",
    );
    key(&mut app, "Tab");
    key(&mut app, "Enter");
    assert!(text(&app).contains("focus:a;blocked;"));
    assert!(!text(&app).contains("click;"));
    let mut reverse = KeyEvent::down("Tab");
    reverse.shift_key = true;
    app.keyboard(&reverse).unwrap();
    assert_eq!(focused(&app).as_deref(), Some("a"));
    let mut modifiers = KeyEvent::up("x");
    modifiers.ctrl_key = true;
    modifiers.alt_key = true;
    modifiers.meta_key = true;
    modifiers.repeat = true;
    assert!(app.keyboard(&modifiers).unwrap());
    assert!(text(&app).contains("modifiers;"));
    key(&mut app, "Tab");
    assert!(text(&app).contains("blur:a;focus:b;"));
}

#[test]
fn pointer_focus_disabled_controls_and_detached_focus_do_not_activate() {
    let mut app = app(
        "<button id=disabled disabled><span id=child>D</span></button><button id=remove>REMOVE</button><button id=next>NEXT</button><script>document.getElementById('remove').onclick=()=>document.getElementById('remove').remove();document.getElementById('disabled').onclick=()=>{throw new Error('disabled click');};</script>",
        "button { height: 20px; }",
    );
    app.click("child").unwrap();
    assert_eq!(focused(&app), None);
    key(&mut app, "Tab");
    assert_eq!(focused(&app).as_deref(), Some("remove"));
    key(&mut app, "Enter");
    assert_eq!(focused(&app), None);
    key(&mut app, "Tab");
    assert_eq!(focused(&app).as_deref(), Some("next"));
    let mut modified = KeyEvent::down("Enter");
    modified.meta_key = true;
    assert!(!app.keyboard(&modified).unwrap());
}

#[test]
fn focus_ring_is_visible_only_for_keyboard_and_respects_ancestor_clipping() {
    let mut app = app(
        "<div style='height: 8px; overflow: hidden'><button id=go style='height: 20px'></button></div>",
        "",
    );
    let options = RenderOptions {
        width: 20,
        height: Some(24),
        ..Default::default()
    };
    let unfocused = app.render(options).unwrap();
    key(&mut app, "Tab");
    let focused = app.render(options).unwrap();
    assert_eq!(focused.pixel(0, 0), Some(Color::rgba(37, 99, 235, 255)));
    assert_eq!(focused.pixel(0, 8), unfocused.pixel(0, 8));
    assert_eq!(app.focused_rect(20).unwrap().unwrap().height, 20);
    app.click("go").unwrap();
    assert_eq!(
        app.render(options).unwrap().pixel(0, 0),
        unfocused.pixel(0, 0)
    );
}

#[test]
fn invalid_keys_are_rejected_and_plain_exports_remain_static() {
    let mut app = Application::from_html("<button id=go>GO</button>", "").unwrap();
    for name in ["", "\n", &"x".repeat(65)] {
        assert!(app.keyboard(&KeyEvent::down(name)).is_err());
    }
    assert!(!app.keyboard(&KeyEvent::down("Tab")).unwrap());
    assert_eq!(focused(&app), None);
}

#[test]
fn official_vue_and_react_accept_keyboard_activation_and_handlers() {
    for example in ["vue-counter", "react-counter"] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join(example)
            .join("index.html");
        let mut app = Application::load(&path, None).unwrap();
        key(&mut app, "Tab");
        assert_eq!(focused(&app).as_deref(), Some("increment"));
        key(&mut app, "Enter");
        assert!(text(&app).contains("COUNT: 1"));
        key(&mut app, " ");
        app.keyboard(&KeyEvent::up(" ")).unwrap();
        assert!(text(&app).contains("COUNT: 2"));
    }
}

#[test]
fn official_framework_keydown_handlers_can_prevent_default_activation() {
    for (framework, source) in [
        (
            "vue-counter",
            "const {createApp,h,ref}=Vue;createApp({setup(){const n=ref(0);return()=>h('button',{id:'go',onClick:()=>n.value++,onKeydown:e=>{if(e.key==='Enter'){e.preventDefault();n.value+=10;}}},'COUNT: '+n.value);}}).mount('#app');",
        ),
        (
            "react-counter",
            "function App(){const [n,set]=React.useState(0);return React.createElement('button',{id:'go',onClick:()=>set(v=>v+1),onKeyDown:e=>{if(e.key==='Enter'){e.preventDefault();set(v=>v+10);}}},'COUNT: '+n);}ReactDOM.createRoot(document.getElementById('app')).render(React.createElement(App));",
        ),
    ] {
        let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join(framework);
        let names = if framework == "vue-counter" {
            vec!["vendor/vue.global.prod.js"]
        } else {
            vec![
                "vendor/react.production.min.js",
                "vendor/react-dom.production.min.js",
            ]
        };
        let texts: Vec<_> = names
            .iter()
            .map(|n| std::fs::read_to_string(base.join(n)).unwrap())
            .collect();
        let html = format!(
            "<div id=app></div>{}<script src=app.js></script>",
            names
                .iter()
                .map(|n| format!("<script src='{n}'></script>"))
                .collect::<String>()
        );
        let mut entries = vec![("index.html", html.as_str()), ("app.js", source)];
        entries.extend(names.iter().zip(&texts).map(|(n, t)| (*n, t.as_str())));
        let assets = AssetBundle::new("index.html", &entries).unwrap();
        let mut app = Application::from_assets(&assets).unwrap();
        key(&mut app, "Tab");
        key(&mut app, "Enter");
        assert!(text(&app).contains("COUNT: 10"), "{}", text(&app));
        key(&mut app, " ");
        app.keyboard(&KeyEvent::up(" ")).unwrap();
        assert!(text(&app).contains("COUNT: 11"));
    }
}

#[test]
fn hiding_focused_control_flushes_focusout_jobs_and_cancels_space() {
    let mut app = app(
        "<button id=go>GO</button><p id=log></p><script>const go=document.getElementById('go');go.addEventListener('keydown',e=>{if(e.key==='x')go.className='gone';});go.addEventListener('focusout',()=>Promise.resolve().then(()=>document.getElementById('log').textContent='BLURRED'));go.onclick=()=>{throw new Error('unexpected click');};</script>",
        ".gone { display: none; }",
    );
    key(&mut app, "Tab");
    key(&mut app, " ");
    assert!(!app.keyboard(&KeyEvent::down("x")).unwrap());
    assert_eq!(focused(&app), None);
    assert!(text(&app).contains("BLURRED"));
    assert!(!app.keyboard(&KeyEvent::up(" ")).unwrap());
}
