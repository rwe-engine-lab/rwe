//! Strict author stylesheets with structural selectors and a small cascade.
use crate::{BoxStyle, Document, NodeId, NodeKind, RenderError};

/// A parsed author stylesheet using the same properties as inline styles.
///
/// Selectors support a type or `*`, followed by any number of `.class` and `#id`
/// components, or just class/id components. Comma-separated selector lists are
/// supported, with descendant whitespace and child (`>`) combinators.
/// Sibling combinators, pseudo-classes, escapes, and at-rules are unsupported.
///
/// ```
/// use rwe::{BlockLayout, Document, Stylesheet};
/// let document = Document::parse("<main class='shell'><section class='card'></section></main>")?;
/// let css = Stylesheet::parse(".shell > section.card { height: 24px; padding: 8px }")?;
/// let layout = BlockLayout::with_stylesheet(&document, &css, 120)?;
/// assert_eq!(layout.height(), 40);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Default)]
pub struct Stylesheet {
    rules: Vec<Rule>,
}

#[derive(Debug, Clone)]
struct Rule {
    selectors: Vec<Selector>,
    declarations: Vec<Declaration>,
}
#[derive(Debug, Clone)]
struct Selector {
    compounds: Vec<Compound>,
    relations: Vec<Relation>,
    specificity: (usize, usize, usize),
}

#[derive(Debug, Clone, Copy)]
enum Relation {
    Descendant,
    Child,
}

impl Selector {
    fn parse(source: &str) -> Result<Self, RenderError> {
        let mut compounds = vec![];
        let mut relations = vec![];
        let bytes = source.as_bytes();
        let mut pos = 0;
        while pos < bytes.len() {
            let start = pos;
            while pos < bytes.len() && !bytes[pos].is_ascii_whitespace() && bytes[pos] != b'>' {
                pos += 1;
            }
            compounds.push(Compound::parse(&source[start..pos])?);
            while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
                pos += 1;
            }
            if pos == bytes.len() {
                break;
            }
            if bytes[pos] == b'>' {
                relations.push(Relation::Child);
                pos += 1;
                while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
                    pos += 1;
                }
                if pos == bytes.len() {
                    return Err(RenderError("missing selector after child combinator"));
                }
            } else {
                relations.push(Relation::Descendant);
            }
        }
        if compounds.is_empty() {
            return Err(RenderError("empty selector"));
        }
        let specificity = compounds
            .iter()
            .fold((0, 0, 0), |(ids, classes, types), part| {
                let next = part.specificity();
                (ids + next.0, classes + next.1, types + next.2)
            });
        Ok(Self {
            compounds,
            relations,
            specificity,
        })
    }
    fn specificity(&self) -> (usize, usize, usize) {
        self.specificity
    }
    fn matches(&self, document: &Document, node: NodeId) -> bool {
        if !self
            .compounds
            .last()
            .is_some_and(|part| part.matches(document, node))
        {
            return false;
        }
        if self.compounds.len() == 1 {
            return true;
        }
        let mut ancestors = vec![];
        let mut current = Some(node);
        while let Some(id) = current {
            let dom = &document.nodes()[id];
            if matches!(dom.kind, NodeKind::Element { .. }) {
                ancestors.push(id);
            }
            current = dom.parent;
        }
        if ancestors.len() < self.compounds.len() {
            return false;
        }
        ancestors.reverse();
        // Dynamic matching retains all possible ancestors, including alternatives
        // needed when a nearer descendant match fails an earlier child relation.
        let mut previous = ancestors
            .iter()
            .map(|&id| self.compounds[0].matches(document, id))
            .collect::<Vec<_>>();
        let mut next = vec![false; ancestors.len()];
        for (part, relation) in self.compounds[1..].iter().zip(&self.relations) {
            let mut prefix = false;
            for (index, &id) in ancestors.iter().enumerate() {
                let related = match relation {
                    Relation::Child => index > 0 && previous[index - 1],
                    Relation::Descendant => prefix,
                };
                next[index] = related && part.matches(document, id);
                prefix |= previous[index];
            }
            std::mem::swap(&mut previous, &mut next);
        }
        previous.last() == Some(&true)
    }
}

#[derive(Debug, Clone)]
struct Compound {
    tag: Option<String>,
    ids: Vec<String>,
    classes: Vec<String>,
}
impl Compound {
    fn parse(source: &str) -> Result<Self, RenderError> {
        let mut selector = Self {
            tag: None,
            ids: vec![],
            classes: vec![],
        };
        let bytes = source.as_bytes();
        let mut pos = 0;
        if bytes.first() == Some(&b'*') {
            pos += 1;
        } else if bytes.first().is_some_and(u8::is_ascii_alphabetic) {
            selector.tag = Some(identifier(source, &mut pos)?.to_ascii_lowercase());
        }
        while pos < bytes.len() {
            let marker = bytes[pos];
            if marker != b'.' && marker != b'#' {
                return Err(RenderError("unsupported selector syntax"));
            }
            pos += 1;
            let name = identifier(source, &mut pos)?.to_owned();
            if marker == b'.' {
                selector.classes.push(name);
            } else {
                selector.ids.push(name);
            }
        }
        if source.is_empty() {
            return Err(RenderError("empty selector"));
        }
        Ok(selector)
    }
    fn specificity(&self) -> (usize, usize, usize) {
        (
            self.ids.len(),
            self.classes.len(),
            usize::from(self.tag.is_some()),
        )
    }
    fn matches(&self, document: &Document, node: NodeId) -> bool {
        let NodeKind::Element { name, attributes } = &document.nodes()[node].kind else {
            return false;
        };
        self.tag.as_deref().is_none_or(|tag| tag == name)
            && self.ids.iter().all(|id| attributes.get("id") == Some(id))
            && self.classes.iter().all(|class| {
                attributes
                    .get("class")
                    .is_some_and(|value| value.split_ascii_whitespace().any(|token| token == class))
            })
    }
}
fn identifier<'a>(source: &'a str, pos: &mut usize) -> Result<&'a str, RenderError> {
    let start = *pos;
    if !source
        .as_bytes()
        .get(*pos)
        .is_some_and(|b| b.is_ascii_alphabetic() || matches!(b, b'_' | b'-'))
    {
        return Err(RenderError("expected ASCII selector identifier"));
    }
    *pos += 1;
    while source
        .as_bytes()
        .get(*pos)
        .is_some_and(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
    {
        *pos += 1;
    }
    Ok(&source[start..*pos])
}

#[derive(Debug, Clone)]
struct Declaration {
    property: Property,
    style: BoxStyle,
}
#[derive(Debug, Clone, Copy)]
enum Property {
    Width,
    Height,
    Margin,
    Padding,
    MarginTop,
    MarginRight,
    MarginBottom,
    MarginLeft,
    PaddingTop,
    PaddingRight,
    PaddingBottom,
    PaddingLeft,
    Border,
    BorderWidth,
    BorderStyle,
    BorderColor,
    Background,
    Display,
    FlexBasis,
    FlexGrow,
    FlexShrink,
    Gap,
    JustifyContent,
    AlignItems,
    Color,
    FontSize,
    TextAlign,
    WhiteSpace,
    Overflow,
}
impl Declaration {
    fn parse_many(source: &str) -> Result<Vec<Self>, RenderError> {
        crate::css::declarations(source)?
            .into_iter()
            .map(Self::from_css)
            .collect()
    }
    fn from_css(declaration: crate::css::Declaration) -> Result<Self, RenderError> {
        let mut style = BoxStyle::default();
        style.apply_property(&declaration.name, &declaration.value)?;
        let key = &declaration.name;
        let property = match key.as_str() {
            "width" => Property::Width,
            "flex-basis" => Property::FlexBasis,
            "flex-grow" => Property::FlexGrow,
            "flex-shrink" => Property::FlexShrink,
            "gap" => Property::Gap,
            "justify-content" => Property::JustifyContent,
            "align-items" => Property::AlignItems,
            "height" => Property::Height,
            "margin" => Property::Margin,
            "padding" => Property::Padding,
            "margin-top" => Property::MarginTop,
            "margin-right" => Property::MarginRight,
            "margin-bottom" => Property::MarginBottom,
            "margin-left" => Property::MarginLeft,
            "padding-top" => Property::PaddingTop,
            "padding-right" => Property::PaddingRight,
            "padding-bottom" => Property::PaddingBottom,
            "padding-left" => Property::PaddingLeft,
            "border" => Property::Border,
            "border-width" => Property::BorderWidth,
            "border-style" => Property::BorderStyle,
            "border-color" => Property::BorderColor,
            "background-color" => Property::Background,
            "display" => Property::Display,
            "color" => Property::Color,
            "font-size" => Property::FontSize,
            "text-align" => Property::TextAlign,
            "white-space" => Property::WhiteSpace,
            "overflow" => Property::Overflow,
            _ => return Err(RenderError("unsupported stylesheet property")),
        };
        Ok(Self { property, style })
    }
    fn apply(&self, target: &mut BoxStyle) {
        match self.property {
            Property::Width => target.width = self.style.width,
            Property::Height => target.height = self.style.height,
            Property::Margin => target.margin = self.style.margin,
            Property::Padding => target.padding = self.style.padding,
            Property::MarginTop => target.margin.top = self.style.margin.top,
            Property::MarginRight => target.margin.right = self.style.margin.right,
            Property::MarginBottom => target.margin.bottom = self.style.margin.bottom,
            Property::MarginLeft => target.margin.left = self.style.margin.left,
            Property::PaddingTop => target.padding.top = self.style.padding.top,
            Property::PaddingRight => target.padding.right = self.style.padding.right,
            Property::PaddingBottom => target.padding.bottom = self.style.padding.bottom,
            Property::PaddingLeft => target.padding.left = self.style.padding.left,
            Property::Border => {
                target.border_width = self.style.border_width;
                target.border_style = self.style.border_style;
                target.border_color = self.style.border_color;
            }
            Property::BorderWidth => target.border_width = self.style.border_width,
            Property::BorderStyle => target.border_style = self.style.border_style,
            Property::BorderColor => target.border_color = self.style.border_color,
            Property::Background => target.background = self.style.background,
            Property::Display => {
                target.visible = self.style.visible;
                target.flex = self.style.flex;
            }
            Property::FlexBasis => target.flex_basis = self.style.flex_basis,
            Property::FlexGrow => target.flex_grow = self.style.flex_grow,
            Property::FlexShrink => target.flex_shrink = self.style.flex_shrink,
            Property::Gap => target.gap = self.style.gap,
            Property::JustifyContent => target.justify_content = self.style.justify_content,
            Property::AlignItems => target.align_items = self.style.align_items,
            Property::Color => target.color = self.style.color,
            Property::FontSize => target.font_size = self.style.font_size,
            Property::TextAlign => target.text_align = self.style.text_align,
            Property::WhiteSpace => target.white_space = self.style.white_space,
            Property::Overflow => target.overflow = self.style.overflow,
        }
    }
}

impl Stylesheet {
    /// Parses up to 1 MiB of CSS. Any unsupported rule or declaration is an error,
    /// even in rules that will not match. cssparser handles syntax, comments and EOF.
    pub fn parse(source: &str) -> Result<Self, RenderError> {
        let rules = crate::css::rules(source)?
            .into_iter()
            .map(|rule| {
                Ok(Rule {
                    selectors: rule
                        .selectors
                        .into_iter()
                        .map(|s| Selector::parse(&s))
                        .collect::<Result<_, _>>()?,
                    declarations: rule
                        .declarations
                        .into_iter()
                        .map(Declaration::from_css)
                        .collect::<Result<_, _>>()?,
                })
            })
            .collect::<Result<_, RenderError>>()?;
        Ok(Self { rules })
    }
    /// Resolves author rules by ID/class/type specificity, then source order,
    /// separately for each property. Inline declarations override all rules.
    /// Returns `None` for text, the document root, or an invalid node index.
    /// Inherited text fields remain `None` here; layout resolves them from parents.
    /// There is no `!important` or browser default stylesheet.
    pub fn computed_style(
        &self,
        document: &Document,
        node: NodeId,
    ) -> Result<Option<BoxStyle>, RenderError> {
        let Some(crate::Node {
            kind: NodeKind::Element { attributes, .. },
            ..
        }) = document.node(node)
        else {
            return Ok(None);
        };
        let mut matched = self
            .rules
            .iter()
            .filter_map(|rule| {
                rule.selectors
                    .iter()
                    .filter(|selector| selector.matches(document, node))
                    .map(Selector::specificity)
                    .max()
                    .map(|specificity| (specificity, rule))
            })
            .collect::<Vec<_>>();
        matched.sort_by_key(|(specificity, _)| *specificity);
        let mut result = BoxStyle::default();
        for (_, rule) in matched {
            for declaration in &rule.declarations {
                declaration.apply(&mut result);
            }
        }
        if let Some(inline) = attributes.get("style") {
            for declaration in Declaration::parse_many(inline)? {
                declaration.apply(&mut result);
            }
        }
        Ok(Some(result))
    }
}
