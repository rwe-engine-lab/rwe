//! Public flat DOM arena populated by the shared html5ever adapter.
use std::{collections::BTreeMap, error::Error, fmt};

/// An index into a document's node arena.
pub type NodeId = usize;

/// The contents of a DOM node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    /// The synthetic fragment root.
    Document,
    /// An element: HTML names are lowercase; foreign names retain adjusted casing.
    Element {
        /// The element name.
        name: String,
        /// Attributes with HTML-normalized names and optional namespace prefixes.
        attributes: BTreeMap<String, String>,
    },
    /// A comment, excluded from descendant text content and layout.
    Comment(String),
    /// Text with character references decoded.
    Text(String),
}

/// An immutable node in a parsed document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// Node contents.
    pub kind: NodeKind,
    /// Element namespace URI; absent for document, text and comment nodes.
    pub namespace: Option<String>,
    /// Parent index, absent only for the document root.
    pub parent: Option<NodeId>,
    /// Child indices in source order.
    pub children: Vec<NodeId>,
}

/// Input/output limits and bounded temporary parser construction.
#[derive(Debug, Clone, Copy)]
pub struct ParseLimits {
    /// Maximum source length in bytes.
    pub max_bytes: usize,
    /// Maximum output nodes including root; temporary parser nodes allow four setup nodes.
    pub max_nodes: usize,
    /// Maximum open element nesting depth.
    pub max_depth: usize,
}

impl Default for ParseLimits {
    fn default() -> Self {
        Self {
            max_bytes: 1024 * 1024,
            max_nodes: 100_000,
            max_depth: 256,
        }
    }
}

/// A resource-limit failure with the last processed UTF-8 byte position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HtmlError {
    /// Byte offset of the failure.
    pub offset: usize,
    /// Description of the invalid or unsupported input.
    pub message: &'static str,
}
impl fmt::Display for HtmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at byte {}", self.message, self.offset)
    }
}
impl Error for HtmlError {}

/// A flat arena whose root is always node zero.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub(crate) nodes: Vec<Node>,
}
impl Document {
    /// Parses an HTML5 fragment in a div context, with recovery and default limits.
    pub fn parse(source: &str) -> Result<Self, HtmlError> {
        Self::parse_with_limits(source, ParseLimits::default())
    }
    /// Parses a fragment with explicit input, output and parser allocation limits.
    pub fn parse_with_limits(source: &str, limits: ParseLimits) -> Result<Self, HtmlError> {
        crate::html::parse(
            source,
            Some(("div", "http://www.w3.org/1999/xhtml")),
            limits,
        )
    }
    /// Parses an HTML5 document, including implied html/head/body elements.
    pub fn parse_page(source: &str) -> Result<Self, HtmlError> {
        crate::html::parse(source, None, ParseLimits::default())
    }
    /// Parses innerHTML in the element's local-name and namespace context.
    pub fn parse_fragment(source: &str, name: &str, namespace: &str) -> Result<Self, HtmlError> {
        crate::html::parse(source, Some((name, namespace)), ParseLimits::default())
    }
    /// Returns all nodes in insertion order, starting with the document root.
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }
    /// Returns a node, or `None` for an invalid index.
    pub fn node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id)
    }
    /// Concatenates descendant text in source order without whitespace normalization.
    pub fn text_content(&self, id: NodeId) -> Option<String> {
        self.node(id)?;
        let mut result = String::new();
        let mut pending = vec![id];
        while let Some(id) = pending.pop() {
            let node = &self.nodes[id];
            if let NodeKind::Text(text) = &node.kind {
                result.push_str(text);
            }
            pending.extend(node.children.iter().rev().copied());
        }
        Some(result)
    }
}
