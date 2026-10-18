//! Native form activation and live framework bindings through public APIs.
#![cfg(feature = "javascript")]
use rwe::{Application, AssetBundle, KeyEvent, NodeKind};
fn app(html: &str) -> Application {
    Application::from_assets(&AssetBundle::new("index.html", &[("index.html", html)]).unwrap())
        .unwrap()
}
fn checked(app: &Application, id: &str) -> bool {
    app.document()
        .nodes()
        .iter()
        .find_map(|n| match &n.kind {
            NodeKind::Element { attributes, .. }
                if attributes.get("id").is_some_and(|v| v == id) =>
            {
                Some(attributes.contains_key("checked"))
            }
            _ => None,
        })
        .unwrap()
}
fn text(app: &Application) -> String {
    app.document().text_content(0).unwrap()
}
fn focus(app: &Application) -> Option<String> {
    app.document().nodes().iter().find_map(|n| match &n.kind {
        NodeKind::Element { attributes, .. } if attributes.contains_key("data-rwe-focus") => {
            attributes.get("id").cloned()
        }
        _ => None,
    })
}
#[test]
fn checkbox_activation_observes_pre_toggle_and_prevention_restores_state() {
    let mut app = app(
        "<input id=c type=checkbox><input id=cancel type=checkbox checked><input id=disabled type=checkbox disabled><p id=log></p><script>const c=document.getElementById('c'),log=document.getElementById('log');c.onclick=()=>log.textContent+='click:'+c.checked+';';c.oninput=()=>log.textContent+='input;';c.onchange=()=>log.textContent+='change;';document.getElementById('cancel').onclick=e=>e.preventDefault();</script>",
    );
    app.click("c").unwrap();
    assert!(checked(&app, "c"));
    assert!(text(&app).contains("click:true;input;change;"));
    app.click("cancel").unwrap();
    assert!(checked(&app, "cancel"));
    app.click("disabled").unwrap();
    assert!(!checked(&app, "disabled"));
    app.keyboard(&KeyEvent::down(" ")).unwrap(); // focus stays on previous non-disabled control
    app.keyboard(&KeyEvent::up(" ")).unwrap();
    assert!(checked(&app, "cancel"));
    app.click("c").unwrap();
    assert!(!checked(&app, "c"));
}
#[test]
fn radio_groups_use_form_owner_names_and_arrow_navigation() {
    let mut app = app(
        "<form id=f><input id=a type=radio name=choice checked><input id=skip type=radio name=choice disabled><input id=b type=radio name=choice><input id=end type=checkbox></form><form><input id=other type=radio name=choice checked></form><input id=outside type=radio form=f name=choice>",
    );
    app.keyboard(&KeyEvent::down("Tab")).unwrap();
    assert_eq!(focus(&app).as_deref(), Some("a"));
    app.keyboard(&KeyEvent::down("ArrowRight")).unwrap();
    assert_eq!(focus(&app).as_deref(), Some("b"));
    assert!(checked(&app, "b"));
    assert!(!checked(&app, "a"));
    assert!(checked(&app, "other"));
    app.keyboard(&KeyEvent::down("ArrowRight")).unwrap();
    assert_eq!(focus(&app).as_deref(), Some("outside"));
    app.keyboard(&KeyEvent::down("Tab")).unwrap();
    assert_eq!(focus(&app).as_deref(), Some("end"));
    app.click("a").unwrap();
    app.click("a").unwrap();
    assert!(checked(&app, "a"));
    assert!(!checked(&app, "outside"));
}
#[test]
fn associated_labels_forward_clicks_and_default_state_resets_without_input_events() {
    let mut app = app(
        "<form id=f><label id=label for=c>ENABLE</label><input id=c name=enabled type=checkbox checked><label id=nested><input id=inner type=checkbox>INNER</label><button id=reset type=reset>RESET</button></form><p id=log></p><script>let events=0;document.getElementById('c').onchange=()=>document.getElementById('log').textContent=String(++events);</script>",
    );
    app.click("label").unwrap();
    assert!(!checked(&app, "c"));
    assert_eq!(focus(&app).as_deref(), Some("c"));
    app.click("nested").unwrap();
    assert!(checked(&app, "inner"));
    app.click("reset").unwrap();
    assert!(checked(&app, "c"));
    assert!(!checked(&app, "inner"));
    assert!(text(&app).ends_with('1'));
}
#[test]
fn submit_formdata_and_reset_events_support_real_local_form_workflow() {
    let mut app = app(
        "<form id=f><input id=name name=name value=RWE><input id=c type=checkbox name=notify checked><input type=checkbox name=ignored><input type=hidden name=hidden value=secret><input disabled name=disabled value=no><fieldset disabled><input name=blocked value=no></fieldset><input type=radio name=plan value=basic checked><input id=pro type=radio name=plan value=pro><button id=save name=action value=save>SAVE</button><button id=reset type=reset>RESET</button></form><p id=result></p><script>const f=document.getElementById('f');f.onsubmit=e=>{e.preventDefault();const data=new FormData(f,e.submitter);document.getElementById('result').textContent=JSON.stringify(Array.from(data));};</script>",
    );
    app.click("pro").unwrap();
    app.click("name").unwrap();
    app.insert_text("!").unwrap();
    app.keyboard(&KeyEvent::down("Enter")).unwrap();
    let result = text(&app);
    assert!(result.contains("[\"name\",\"RWE!\"]"));
    assert!(result.contains("[\"notify\",\"on\"]"));
    assert!(result.contains("[\"plan\",\"pro\"]"));
    assert!(result.contains("[\"action\",\"save\"]"));
    assert!(result.contains("[\"hidden\",\"secret\"]"));
    assert!(!result.contains("blocked"));
    assert!(!result.contains("ignored"));
    app.click("reset").unwrap();
    app.click("save").unwrap();
    assert!(text(&app).contains("[\"name\",\"RWE\"]"));
    assert!(text(&app).contains("[\"plan\",\"basic\"]"));
}
#[test]
fn programmatic_checked_assignments_do_not_emit_changes_and_formdata_methods_preserve_duplicates() {
    let app = app(
        "<input id=c type=checkbox><p id=result></p><script>const c=document.getElementById('c');c.onchange=()=>{throw new Error('programmatic change');};c.checked=true;c.defaultChecked=false;const d=new FormData();d.append('x',1);d.append('x',2);d.append('y',3);if(d.getAll('x').join()!=='1,2')throw new Error('duplicates');d.set('x',4);if(d.get('x')!=='4'||Array.from(d.keys()).join()!=='x,y')throw new Error('set order');d.delete('y');document.getElementById('result').textContent=JSON.stringify(Array.from(d));</script>",
    );
    assert!(checked(&app, "c"));
    assert!(text(&app).contains("[[\"x\",\"4\"]]"));
}
#[test]
fn actual_vue_and_react_bind_checkbox_radio_and_submit() {
    for (framework, source) in [
        (
            "vue-counter",
            r#"Vue.createApp({data(){return {enabled:false,plan:'basic',submitted:''};},methods:{save(e){this.submitted=Array.from(new FormData(e.target)).map(p=>p.join('=')).join(';');}},template:'<form @submit.prevent="save"><input id=enabled type=checkbox name=enabled v-model="enabled"><input id=basic type=radio name=plan value=basic v-model="plan"><input id=pro type=radio name=plan value=pro v-model="plan"><button id=save>SAVE</button><p>STATE: {{enabled}}/{{plan}}</p><p>DATA: {{submitted}}</p></form>'}).mount('#app');"#,
        ),
        (
            "react-counter",
            r#"function App(){const [enabled,se]=React.useState(false),[plan,sp]=React.useState('basic'),[data,sd]=React.useState('');return React.createElement('form',{onSubmit:e=>{e.preventDefault();sd(Array.from(new FormData(e.currentTarget)).map(p=>p.join('=')).join(';'));}},React.createElement('input',{id:'enabled',name:'enabled',type:'checkbox',checked:enabled,onChange:e=>se(e.target.checked)}),React.createElement('input',{id:'basic',name:'plan',type:'radio',value:'basic',checked:plan==='basic',onChange:()=>sp('basic')}),React.createElement('input',{id:'pro',name:'plan',type:'radio',value:'pro',checked:plan==='pro',onChange:()=>sp('pro')}),React.createElement('button',{id:'save'},'SAVE'),React.createElement('p',null,'STATE: '+enabled+'/'+plan),React.createElement('p',null,'DATA: '+data));}ReactDOM.createRoot(document.getElementById('app')).render(React.createElement(App));"#,
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
        entries.extend(files.iter().zip(&sources).map(|(f, s)| (*f, s.as_str())));
        let mut app = Application::from_assets(&AssetBundle::new("index.html", &entries).unwrap())
            .unwrap_or_else(|e| panic!("{framework}: {e}"));
        app.click("enabled").unwrap();
        app.click("pro").unwrap();
        assert!(checked(&app, "enabled"));
        assert!(checked(&app, "pro"));
        assert!(!checked(&app, "basic"));
        assert!(
            text(&app).contains("STATE: true/pro"),
            "{framework}: {}",
            text(&app)
        );
        app.click("save").unwrap();
        assert!(
            text(&app).contains("DATA: enabled=on;plan=pro"),
            "{framework}: {}",
            text(&app)
        );
    }
}

#[test]
fn settings_example_submits_edited_values_and_resets_controlled_state() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/settings-form/index.html");
    let mut app = Application::load(&path, None).unwrap();
    app.click("notify-label").unwrap();
    app.click("advanced-label").unwrap();
    app.click("display-name").unwrap();
    let mut select = KeyEvent::down("a");
    select.meta_key = true;
    app.keyboard(&select).unwrap();
    app.insert_text("框架用户").unwrap();
    app.click("submit").unwrap();
    let result = text(&app);
    assert!(
        result.contains("displayName=框架用户; mode=advanced"),
        "{result}"
    );
    assert!(result.contains("已在应用内提交表单"));
    assert!(!checked(&app, "notify"));
    app.click("reset").unwrap();
    assert!(checked(&app, "notify"));
    assert!(checked(&app, "simple"));
    assert!(!checked(&app, "advanced"));
    assert!(text(&app).contains("已恢复默认设置"));
    assert!(text(&app).contains("尚未提交"));
}
