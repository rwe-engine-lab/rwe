//! Observable syntax behavior delegated to cssparser, with bounded RWE semantics.
use rwe::{BoxStyle, Color, Document, Stylesheet};
#[test]
fn inline_and_stylesheet_declarations_share_comments_escapes_and_token_boundaries() {
    let source = r"\77 idth: 24px; height: /* } ; : */ 8px; display: bl\6f ck; padding:1px/**/2px; color:#123";
    let inline = BoxStyle::parse(source).unwrap();
    let document = Document::parse("<p id=target></p>").unwrap();
    let sheet = Stylesheet::parse(&format!("#target {{{source}}}")).unwrap();
    assert_eq!(sheet.computed_style(&document, 1).unwrap(), Some(inline));
    assert_eq!(inline.width, Some(24));
    assert_eq!(inline.height, Some(8));
    assert_eq!((inline.padding.top, inline.padding.left), (1, 2));
    assert_eq!(inline.color, Some(Color::rgba(17, 34, 51, 255)));
    assert!(BoxStyle::parse("width:1/**/px").is_err());
}
#[test]
fn comment_removal_does_not_invent_selector_whitespace_or_merge_identifiers() {
    let document =
        Document::parse("<p class='a b'></p><section class='a'><p class='b'></p></section>")
            .unwrap();
    let sheet = Stylesheet::parse(".a/**/.b {height:7px} .a/**/ .b {width:9px}").unwrap();
    assert_eq!(
        sheet.computed_style(&document, 1).unwrap().unwrap().height,
        Some(7)
    );
    assert_eq!(
        sheet.computed_style(&document, 3).unwrap().unwrap().width,
        Some(9)
    );
    assert_eq!(
        sheet.computed_style(&document, 3).unwrap().unwrap().height,
        None
    );
    assert!(Stylesheet::parse("main/**/p {height:1px}").is_err());
}
#[test]
fn library_eof_recovery_is_explicit_but_properties_and_structures_remain_bounded() {
    assert!(Stylesheet::parse("p {height:8px").is_ok());
    assert!(Stylesheet::parse("/* unfinished comment").is_ok());
    assert!(BoxStyle::parse("padding:1px;/* unfinished").is_ok());
    for source in [
        "width:calc(1px + 2px)",
        "color:'red;}'",
        "width:10%",
        "height:1px !important",
        "height:1px; invalid",
    ] {
        assert!(BoxStyle::parse(source).is_err(), "{source}");
    }
    assert!(BoxStyle::parse(&" ".repeat(1_048_577)).is_err());
    assert!(Stylesheet::parse("@media all {p {height:1px}}").is_err());
    assert!(BoxStyle::parse(&format!("x:{}0{}", "f(".repeat(80), ")".repeat(80))).is_err());
}
#[cfg(feature = "javascript")]
#[test]
fn css_text_native_bridge_preserves_strings_blocks_and_previous_state_on_syntax_failure() {
    use rwe::{Application, AssetBundle};
    let html = r#"<p id=target>CSS</p><p id=result></p><script>
    const target=document.getElementById('target');
    target.style.cssText='--sample:"a;b:{/*literal*/}";width:24px;/*x*/height:8px;';
    if(target.style.getPropertyValue('--sample')!=='"a;b:{/*literal*/}"')throw new Error('quoted delimiters');
    if(target.style.width!=='24px'||target.style.height!=='8px')throw new Error('properties');
    const previous=target.style.cssText;let rejected=false;
    try{target.style.cssText='width:24px;bad';}catch(e){rejected=true;}
    if(!rejected||previous!==target.style.cssText)throw new Error('atomic cssText');
    let large='x';for(let i=0;i<21;i++)large+=large;
    for(const source of [large,'--x:'+'f('.repeat(80)+'0'+')'.repeat(80)]) {
      let rejected=false;try{target.style.cssText=source;}catch(e){rejected=true;}
      if(!rejected||target.style.cssText!==previous)throw new Error('bounded cssText');
    }
    target.style.cssText='width:24px;height:8px; background-color:/*};*/#123;';
    document.getElementById('result').textContent='PASSED';
    </script>"#;
    let app =
        Application::from_assets(&AssetBundle::new("index.html", &[("index.html", html)]).unwrap())
            .unwrap();
    assert!(app.document().text_content(0).unwrap().contains("PASSED"));
    let target=app.layout(80).unwrap().boxes().iter().find(|b|matches!(&app.document().nodes()[b.node].kind,rwe::NodeKind::Element{attributes,..} if attributes.get("id").is_some_and(|id|id=="target"))).copied().unwrap();
    assert_eq!(target.content_rect.width, 24);
    assert_eq!(target.background, Color::rgba(17, 34, 51, 255));
}
