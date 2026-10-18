//! Bounded arena TreeSink adapter. Tokenization and recovery belong to html5ever.
use crate::{Document, HtmlError, Node, NodeKind, ParseLimits};
use html5ever::tendril::{StrTendril, TendrilSink};
use html5ever::tree_builder::{ElementFlags, NodeOrText, QuirksMode, TreeSink};
use html5ever::{Attribute, QualName, parse_document, parse_fragment};
use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    rc::Rc,
};
#[derive(Clone)]
struct Handle {
    id: usize,
    name: Option<Rc<QualName>>,
}
struct Entry {
    node: Node,
    template: Option<usize>,
    math_integration: bool,
}
struct Sink {
    entries: RefCell<Vec<Entry>>,
    limits: ParseLimits,
    failed: Cell<Option<&'static str>>,
}
fn failure(offset: usize, message: &'static str) -> HtmlError {
    HtmlError { offset, message }
}
impl Sink {
    fn new(limits: ParseLimits) -> Self {
        Self {
            entries: RefCell::new(vec![Entry {
                node: Node {
                    kind: NodeKind::Document,
                    namespace: None,
                    parent: None,
                    children: vec![],
                },
                template: None,
                math_integration: false,
            }]),
            limits,
            failed: Cell::new(None),
        }
    }
    fn handle(id: usize) -> Handle {
        Handle { id, name: None }
    }
    fn allocate(&self, kind: NodeKind, name: Option<Rc<QualName>>) -> Handle {
        let mut nodes = self.entries.borrow_mut();
        // Fragment setup allocates a context element and a synthetic html root.
        if self.failed.get().is_some() || nodes.len() >= self.limits.max_nodes.saturating_add(4) {
            self.failed
                .set(Some("parser node allocation limit exceeded"));
            return Handle { id: 0, name };
        }
        let id = nodes.len();
        let namespace = name.as_ref().map(|q| q.ns.to_string());
        nodes.push(Entry {
            node: Node {
                kind,
                namespace,
                parent: None,
                children: vec![],
            },
            template: None,
            math_integration: false,
        });
        Handle { id, name }
    }
    fn detach(&self, id: usize) {
        let mut nodes = self.entries.borrow_mut();
        if let Some(parent) = nodes[id].node.parent.take() {
            nodes[parent].node.children.retain(|&n| n != id);
        }
    }
    fn insert(&self, parent: usize, before: Option<usize>, child: NodeOrText<Handle>) {
        if self.failed.get().is_some() {
            return;
        }
        let previous = {
            let nodes = self.entries.borrow();
            let children = &nodes[parent].node.children;
            let index = before
                .and_then(|id| children.iter().position(|&c| c == id))
                .unwrap_or(children.len());
            index.checked_sub(1).map(|i| children[i])
        };
        let child = match child {
            NodeOrText::AppendNode(node) => node,
            NodeOrText::AppendText(text) => {
                if let Some(id) = previous {
                    if let NodeKind::Text(existing) = &mut self.entries.borrow_mut()[id].node.kind {
                        existing.push_str(&text);
                        return;
                    }
                }
                self.allocate(NodeKind::Text(text.to_string()), None)
            }
        };
        if self.failed.get().is_some() {
            return;
        }
        // Bound temporary construction depth, including fragment setup nodes.
        let mut depth = 0;
        let mut ancestor = Some(parent);
        {
            let nodes = self.entries.borrow();
            while let Some(id) = ancestor {
                depth += 1;
                if depth > self.limits.max_depth.saturating_add(4) {
                    self.failed.set(Some("parser depth limit exceeded"));
                    return;
                }
                if id == child.id {
                    self.failed.set(Some("invalid parser tree cycle"));
                    return;
                }
                ancestor = nodes[id].node.parent;
            }
        }
        self.detach(child.id);
        let mut nodes = self.entries.borrow_mut();
        let children = &mut nodes[parent].node.children;
        let index = before
            .and_then(|id| children.iter().position(|&c| c == id))
            .unwrap_or(children.len());
        children.insert(index, child.id);
        nodes[child.id].node.parent = Some(parent);
    }
}
impl TreeSink for Sink {
    type Handle = Handle;
    type Output = Self;
    type ElemName<'a> = &'a QualName;
    fn finish(self) -> Self {
        self
    }
    fn parse_error(&self, _: Cow<'static, str>) {} // HTML5 errors recover; limits are separate.
    fn get_document(&self) -> Handle {
        Self::handle(0)
    }
    fn elem_name<'a>(&'a self, target: &'a Handle) -> &'a QualName {
        target
            .name
            .as_deref()
            .expect("html5ever requested an element name")
    }
    fn same_node(&self, a: &Handle, b: &Handle) -> bool {
        a.id == b.id
    }
    fn set_quirks_mode(&self, _: QuirksMode) {}
    fn create_element(&self, name: QualName, attrs: Vec<Attribute>, flags: ElementFlags) -> Handle {
        let attributes = attrs
            .into_iter()
            .map(|a| (attribute_name(&a), a.value.to_string()))
            .collect();
        let node = self.allocate(
            NodeKind::Element {
                name: name.local.to_string(),
                attributes,
            },
            Some(Rc::new(name)),
        );
        if flags.template && self.failed.get().is_none() {
            let content = self.allocate(NodeKind::Document, None);
            if self.failed.get().is_none() {
                self.entries.borrow_mut()[node.id].template = Some(content.id);
            }
        }
        if self.failed.get().is_none() {
            self.entries.borrow_mut()[node.id].math_integration =
                flags.mathml_annotation_xml_integration_point;
        }
        node
    }
    fn create_comment(&self, text: StrTendril) -> Handle {
        self.allocate(NodeKind::Comment(text.to_string()), None)
    }
    fn create_pi(&self, target: StrTendril, data: StrTendril) -> Handle {
        self.create_comment(format!("{target} {data}").into())
    }
    fn get_template_contents(&self, node: &Handle) -> Handle {
        Self::handle(self.entries.borrow()[node.id].template.unwrap_or(0))
    }
    fn is_mathml_annotation_xml_integration_point(&self, node: &Handle) -> bool {
        self.entries.borrow()[node.id].math_integration
    }
    fn append(&self, parent: &Handle, child: NodeOrText<Handle>) {
        self.insert(parent.id, None, child)
    }
    fn append_before_sibling(&self, sibling: &Handle, child: NodeOrText<Handle>) {
        let parent = self.entries.borrow()[sibling.id].node.parent;
        if let Some(parent) = parent {
            self.insert(parent, Some(sibling.id), child);
        }
    }
    fn append_based_on_parent_node(
        &self,
        element: &Handle,
        previous: &Handle,
        child: NodeOrText<Handle>,
    ) {
        let has_parent = self.entries.borrow()[element.id].node.parent.is_some();
        if has_parent {
            self.append_before_sibling(element, child)
        } else {
            self.append(previous, child)
        }
    }
    fn append_doctype_to_document(&self, _: StrTendril, _: StrTendril, _: StrTendril) {} // Not part of RWE's public arena.
    fn add_attrs_if_missing(&self, target: &Handle, attrs: Vec<Attribute>) {
        if self.failed.get().is_some() {
            return;
        }
        if let NodeKind::Element { attributes, .. } =
            &mut self.entries.borrow_mut()[target.id].node.kind
        {
            for a in attrs {
                attributes
                    .entry(attribute_name(&a))
                    .or_insert_with(|| a.value.to_string());
            }
        }
    }
    fn remove_from_parent(&self, target: &Handle) {
        if self.failed.get().is_none() {
            self.detach(target.id);
        }
    }
    fn reparent_children(&self, node: &Handle, new_parent: &Handle) {
        if self.failed.get().is_some() {
            return;
        }
        let children = self.entries.borrow()[node.id].node.children.clone();
        for id in children {
            self.insert(
                new_parent.id,
                None,
                NodeOrText::AppendNode(Self::handle(id)),
            );
        }
    }
}
fn attribute_name(attribute: &Attribute) -> String {
    match &attribute.name.prefix {
        Some(prefix) => format!("{prefix}:{}", attribute.name.local),
        None => attribute.name.local.to_string(),
    }
}
pub(crate) fn parse(
    source: &str,
    context: Option<(&str, &str)>,
    limits: ParseLimits,
) -> Result<Document, HtmlError> {
    if source.len() > limits.max_bytes {
        return Err(failure(0, "input byte limit exceeded"));
    }
    if limits.max_nodes == 0 {
        return Err(failure(0, "node limit exceeded"));
    }
    let sink = Sink::new(limits);
    let mut parser = match context {
        Some((name, namespace)) => parse_fragment(
            sink,
            Default::default(),
            QualName::new(None, namespace.into(), name.into()),
            vec![],
            false,
        ),
        None => parse_document(sink, Default::default()),
    };
    let mut offset = 0;
    while offset < source.len() {
        let mut end = (offset + 1024).min(source.len());
        while !source.is_char_boundary(end) {
            end -= 1;
        }
        parser.process(StrTendril::from_slice(&source[offset..end]));
        offset = end;
        if let Some(message) = parser.tokenizer.sink.sink.failed.get() {
            return Err(failure(offset, message));
        }
    }
    let sink = parser.finish();
    if let Some(message) = sink.failed.get() {
        return Err(failure(source.len(), message));
    }
    let entries = sink.entries.into_inner();
    let root = if context.is_some() {
        entries[0].node.children.first().copied().unwrap_or(0)
    } else {
        0
    };
    let mut document = Document {
        nodes: vec![Node {
            kind: NodeKind::Document,
            namespace: None,
            parent: None,
            children: vec![],
        }],
    };
    let mut pending: Vec<_> = entries[root]
        .node
        .children
        .iter()
        .rev()
        .map(|&id| (id, 0, 0))
        .collect();
    while let Some((old, parent, depth)) = pending.pop() {
        let entry = &entries[old];
        let next_depth = depth + usize::from(matches!(entry.node.kind, NodeKind::Element { .. }));
        if next_depth > limits.max_depth {
            return Err(failure(source.len(), "depth limit exceeded"));
        }
        if document.nodes.len() >= limits.max_nodes {
            return Err(failure(source.len(), "node limit exceeded"));
        }
        let id = document.nodes.len();
        document.nodes.push(Node {
            kind: entry.node.kind.clone(),
            namespace: entry.node.namespace.clone(),
            parent: Some(parent),
            children: vec![],
        });
        document.nodes[parent].children.push(id);
        let content = entry.template.unwrap_or(old);
        pending.extend(
            entries[content]
                .node
                .children
                .iter()
                .rev()
                .map(|&child| (child, id, next_depth)),
        );
    }
    Ok(document)
}
