//! HTML5 recovery, Unicode, arena invariants and resource boundaries.
use rwe::{Document, NodeKind, ParseLimits};

#[test]
fn builds_arena_with_parent_links_and_decodes_text() {
    let doc = Document::parse(
        "<MAIN ID='app' hidden><!-- skip --><p>你好 &amp; &#x1F980;</p><br/>tail</MAIN>",
    )
    .unwrap();
    assert_eq!(doc.text_content(0).unwrap(), "你好 & 🦀tail");
    assert_eq!(doc.nodes()[0].children, [1]);
    assert_eq!(doc.nodes()[1].children, [2, 3, 5, 6]);
    assert_eq!(doc.nodes()[4].parent, Some(3));
    let NodeKind::Element { name, attributes } = &doc.nodes()[1].kind else {
        panic!("expected element")
    };
    assert_eq!(name, "main");
    assert_eq!(attributes["id"], "app");
    assert_eq!(attributes["hidden"], "");
    assert_eq!(doc.node(100), None);
    assert_eq!(doc.text_content(100), None);
}

#[test]
fn recovers_omitted_tags_duplicate_attributes_and_invalid_character_references() {
    let doc =
        Document::parse("<p a=first A=second>one<p>two &copy; &NotEqualTilde; &#0; &unknown;")
            .unwrap();
    let elements: Vec<_> = doc
        .nodes()
        .iter()
        .filter_map(|n| match &n.kind {
            NodeKind::Element { name, attributes } => Some((name, attributes)),
            _ => None,
        })
        .collect();
    assert_eq!(elements.len(), 2);
    assert_eq!(elements[0].1["a"], "first");
    assert_eq!(doc.text_content(0).unwrap(), "onetwo © ≂̸ � &unknown;");
    assert!(
        matches!(&Document::parse("<!--unfinished").unwrap().nodes()[1].kind,NodeKind::Comment(s) if s=="unfinished")
    );
}

#[test]
fn preserves_text_and_handles_entities_and_void_elements() {
    let doc =
        Document::parse("a <img src='a&amp;b' disabled>\n&lt;&gt;&quot;&apos;&nbsp;&#65;").unwrap();
    assert_eq!(doc.text_content(0).unwrap(), "a \n<>\"'\u{a0}A");
    assert_eq!(Document::parse("").unwrap().nodes().len(), 1);
    assert!(Document::parse("<input disabled checked>").is_ok());
}

#[test]
fn enforces_each_limit_at_its_boundary() {
    let limits = ParseLimits {
        max_bytes: 4,
        max_nodes: 2,
        max_depth: 1,
    };
    assert!(Document::parse_with_limits("<a/>", limits).is_ok());
    assert!(Document::parse_with_limits("12345", limits).is_err());
    assert!(
        Document::parse_with_limits(
            "<a/>",
            ParseLimits {
                max_nodes: 1,
                ..limits
            }
        )
        .is_err()
    );
    assert!(
        Document::parse_with_limits(
            "",
            ParseLimits {
                max_nodes: 0,
                ..limits
            }
        )
        .is_err()
    );
    assert!(
        Document::parse_with_limits(
            "<a/>",
            ParseLimits {
                max_depth: 0,
                ..limits
            }
        )
        .is_err()
    );
    assert!(
        Document::parse_with_limits(
            "<a><b/></a>",
            ParseLimits {
                max_bytes: 100,
                ..limits
            }
        )
        .is_err()
    );
}

#[test]
fn deep_inputs_do_not_require_recursive_drop_or_traversal() {
    let source = format!("{}text{}", "<div>".repeat(2000), "</div>".repeat(2000));
    assert!(Document::parse(&source).is_err());
    let doc = Document::parse_with_limits(
        &source,
        ParseLimits {
            max_depth: 2000,
            ..ParseLimits::default()
        },
    )
    .unwrap();
    assert_eq!(doc.text_content(0).unwrap(), "text");
}

#[test]
fn introductory_document_preserves_text_and_parent_child_links() {
    let document = Document::parse(include_str!("fixtures/dom/intro.html")).unwrap();
    assert_eq!(
        document.text_content(0).unwrap(),
        "Hello, rwe!纯 Rust DOM & HTML"
    );
    assert_eq!(document.nodes().len(), 7);
    assert_eq!(document.nodes()[1].children, [2, 4, 6]);
    for &child in &document.nodes()[1].children {
        assert_eq!(document.nodes()[child].parent, Some(1));
    }
}

#[test]
fn fragment_contexts_recover_tables_and_preserve_foreign_namespaces() {
    let table =
        Document::parse_fragment("<tr><td>cell", "table", "http://www.w3.org/1999/xhtml").unwrap();
    let names: Vec<_> = table
        .nodes()
        .iter()
        .filter_map(|n| match &n.kind {
            NodeKind::Element { name, .. } => Some(name.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(names, ["tbody", "tr", "td"]);
    let svg = Document::parse_fragment(
        "<circle/><foreignObject><div>html</div></foreignObject>",
        "svg",
        "http://www.w3.org/2000/svg",
    )
    .unwrap();
    assert_eq!(
        svg.nodes()[1].namespace.as_deref(),
        Some("http://www.w3.org/2000/svg")
    );
    let div = svg
        .nodes()
        .iter()
        .find(|n| matches!(&n.kind,NodeKind::Element{name,..} if name=="div"))
        .unwrap();
    assert_eq!(
        div.namespace.as_deref(),
        Some("http://www.w3.org/1999/xhtml")
    );
}
#[test]
fn document_implies_structure_and_fragments_do_not_add_page_wrappers() {
    let doc = Document::parse_page("<title>title</title><p>body").unwrap();
    let names: Vec<_> = doc
        .nodes()
        .iter()
        .filter_map(|n| match &n.kind {
            NodeKind::Element { name, .. } => Some(name.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(names, ["html", "head", "title", "body", "p"]);
    let fragment = Document::parse("<p>text").unwrap();
    assert_eq!(fragment.nodes().len(), 3);
    assert_eq!(
        fragment.nodes()[1].namespace.as_deref(),
        Some("http://www.w3.org/1999/xhtml")
    );
}

#[test]
fn html5_tree_recovery_fosters_table_text_and_reconstructs_formatting() {
    let doc = Document::parse("<table>before<tr><td>A</table>after").unwrap();
    assert!(matches!(&doc.nodes()[1].kind,NodeKind::Text(s) if s=="before"));
    assert_eq!(doc.text_content(0).unwrap(), "beforeAafter");
    let doc = Document::parse("<b><i>A</b>B</i>").unwrap();
    let names: Vec<_> = doc
        .nodes()
        .iter()
        .filter_map(|n| match &n.kind {
            NodeKind::Element { name, .. } => Some(name.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(names, ["b", "i", "i"]);
    assert_eq!(doc.text_content(0).unwrap(), "AB");
    for (id, node) in doc.nodes().iter().enumerate() {
        for &child in &node.children {
            assert_eq!(doc.nodes()[child].parent, Some(id));
        }
    }
}
