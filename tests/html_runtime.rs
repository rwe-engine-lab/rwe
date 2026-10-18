//! The same html5ever path backs initial documents and contextual innerHTML.
#![cfg(feature = "javascript")]
use rwe::{Application, AssetBundle};
fn app(source: &str) -> Application {
    Application::from_assets(&AssetBundle::new("index.html", &[("index.html", source)]).unwrap())
        .unwrap()
}
#[test]
fn contextual_innerhtml_recovers_tables_entities_comments_and_templates() {
    let app = app(r#"<div id=host></div><p id=result></p><script>
    const host=document.getElementById('host');
    host.innerHTML='<table><tr><td>cell &copy;</table><!--marker--><template><p>inert';
    const table=host.firstChild;
    if(table.firstChild.localName!=='tbody'||table.firstChild.firstChild.localName!=='tr')throw new Error('table recovery');
    if(table.textContent!=='cell ©')throw new Error('entity');
    if(host.childNodes[1].nodeType!==8||host.childNodes[1].data!=='marker')throw new Error('comment');
    const template=host.lastChild;
    if(template.content.firstChild.localName!=='p'||template.content.textContent!=='inert')throw new Error('template');
    const tbody=table.firstChild;tbody.innerHTML='<tr><td>updated';
    if(tbody.firstChild.localName!=='tr'||table.textContent!=='updated')throw new Error('context');
    document.getElementById('result').textContent='PASSED';
    </script>"#);
    assert!(app.document().text_content(0).unwrap().contains("PASSED"));
}
#[test]
fn initial_and_dynamic_html_have_matching_recovery_and_namespace_behavior() {
    let app = app(
        r#"<div id=initial><p a=first A=second>one<p>two &NotEqualTilde;</div><div id=dynamic></div><p id=result></p><script>
    const a=document.getElementById('initial'),b=document.getElementById('dynamic');
    b.innerHTML='<p a=first A=second>one<p>two &NotEqualTilde;';
    if(a.innerHTML!==b.innerHTML||a.childNodes.length!==2)throw new Error('different parsers');
    const svg=document.createElementNS('http://www.w3.org/2000/svg','svg');
    svg.innerHTML='<circle/><foreignObject><div>html</div></foreignObject>';
    if(svg.firstChild.namespaceURI!=='http://www.w3.org/2000/svg')throw new Error('svg namespace');
    if(svg.lastChild.firstChild.namespaceURI!=='http://www.w3.org/1999/xhtml')throw new Error('html integration');
    const script=document.createElement('script');script.innerHTML='if (a < b) { "&amp;"; }';
    if(script.textContent!=='if (a < b) { "&amp;"; }')throw new Error('raw text');
    const lone=document.createElement('p');lone.innerHTML=String.fromCharCode(0xd800);
    if(lone.textContent!=='�')throw new Error('unpaired surrogate conversion');
    document.getElementById('result').textContent='PASSED';
    </script>"#,
    );
    assert!(app.document().text_content(0).unwrap().contains("PASSED"));
}
#[test]
fn rejected_innerhtml_preserves_previous_children_and_runtime_stays_usable() {
    let app = app(r#"<div id=host>KEEP</div><p id=result></p><script>
    const host=document.getElementById('host');
    let oversized='x';for(let i=0;i<21;i++)oversized+=oversized;
    for(const markup of [oversized,'<div>'.repeat(300),'<i></i>'.repeat(100001)]) {
      let rejected=false;try{host.innerHTML=markup;}catch(error){rejected=true;}
      if(!rejected||host.textContent!=='KEEP')throw new Error('non-atomic bounded update');
    }
    host.innerHTML='<p>RECOVERED';document.getElementById('result').textContent=host.textContent;
    </script>"#);
    assert!(
        app.document()
            .text_content(0)
            .unwrap()
            .contains("RECOVERED")
    );
}

#[test]
fn html5_example_updates_through_public_application_api() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/html5-recovery/index.html");
    let mut app = Application::load(&path, None).unwrap();
    app.click("update").unwrap();
    assert!(
        app.document()
            .text_content(0)
            .unwrap()
            .contains("已更新：2 个段落；注释类型 8")
    );
}
