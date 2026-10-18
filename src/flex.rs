//! Taffy adapter for the currently exposed explicit-basis flex-row CSS subset.
use crate::{BorderStyle, BoxStyle, Document, Edges, NodeId, NodeKind, RenderError, Stylesheet};
use taffy::prelude as t;
use taffy::prelude::{FromLength, TaffyAuto};

/// Horizontal distribution of a single flex row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JustifyContent {
    /// Pack against the start edge.
    #[default]
    Start,
    /// Center the row.
    Center,
    /// Pack against the end edge.
    End,
    /// Distribute remaining space between items.
    SpaceBetween,
}
/// Vertical alignment of items in a single flex row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlignItems {
    /// Stretch auto-height item border boxes to the row height.
    #[default]
    Stretch,
    /// Align to the top edge.
    Start,
    /// Center vertically.
    Center,
    /// Align to the bottom edge.
    End,
}

#[derive(Clone)]
pub(crate) struct RowSpan {
    pub(crate) style: BoxStyle,
    pub(crate) boxes: std::ops::Range<usize>,
    pub(crate) text: std::ops::Range<usize>,
}
pub(crate) struct RowItem {
    pub(crate) node: NodeId,
    pub(crate) width: u32,
    pub(crate) x: u32,
    pub(crate) style: BoxStyle,
}
pub(crate) struct RowEngine {
    tree: t::TaffyTree<()>,
    root: t::NodeId,
    nodes: Vec<t::NodeId>,
    width: u32,
}
fn error(_: taffy::TaffyError) -> RenderError {
    RenderError("Taffy layout failed")
}
// Taffy uses f32. Reject geometry outside its exact integer-pixel range rather
// than silently changing a bounded integer coordinate into a different value.
fn scalar(value: u32) -> Result<f32, RenderError> {
    if value > 16_777_216 {
        return Err(RenderError("Taffy geometry exceeds exact pixel range"));
    }
    Ok(value as f32)
}
fn pixel(value: f32) -> Result<u32, RenderError> {
    if !value.is_finite() || !(0.0..=16_777_216.0).contains(&value) {
        return Err(RenderError(
            "Taffy returned unsupported negative or imprecise geometry",
        ));
    }
    Ok(value.round() as u32)
}
fn edges<T>(value: Edges, convert: impl Fn(f32) -> T) -> Result<t::Rect<T>, RenderError> {
    Ok(t::Rect {
        left: convert(scalar(value.left)?),
        right: convert(scalar(value.right)?),
        top: convert(scalar(value.top)?),
        bottom: convert(scalar(value.bottom)?),
    })
}
fn child_style(style: BoxStyle) -> Result<t::Style, RenderError> {
    let basis = style.flex_basis.or(style.width).ok_or(RenderError(
        "flex item requires a pixel width or flex-basis; intrinsic sizing is not yet connected",
    ))?;
    let border = if style.border_style == BorderStyle::Solid {
        style.border_width
    } else {
        0
    };
    Ok(t::Style {
        box_sizing: t::BoxSizing::ContentBox,
        size: t::Size {
            width: style
                .width
                .map(scalar)
                .transpose()?
                .map(t::Dimension::from_length)
                .unwrap_or(t::Dimension::AUTO),
            height: style
                .height
                .map(scalar)
                .transpose()?
                .map(t::Dimension::from_length)
                .unwrap_or(t::Dimension::AUTO),
        },
        min_size: t::Size {
            width: t::LengthPercentageAuto::from_length(0.0),
            height: t::LengthPercentageAuto::from_length(0.0),
        },
        flex_basis: t::Dimension::from_length(scalar(basis)?),
        flex_grow: style.flex_grow as f32,
        flex_shrink: style.flex_shrink as f32,
        margin: edges(style.margin, t::LengthPercentageAuto::from_length)?,
        padding: edges(style.padding, t::LengthPercentage::from_length)?,
        border: edges(Edges::all(border), t::LengthPercentage::from_length)?,
        ..Default::default()
    })
}
pub(crate) fn plan_row(
    document: &Document,
    sheet: &Stylesheet,
    node: NodeId,
    width: u32,
    style: BoxStyle,
) -> Result<(Vec<RowItem>, RowEngine), RenderError> {
    let mut tree = t::TaffyTree::new();
    let mut items = vec![];
    let mut nodes = vec![];
    for &child in &document.nodes()[node].children {
        let dom = &document.nodes()[child];
        match &dom.kind {
            NodeKind::Comment(_) => continue,
            NodeKind::Text(text)
                if text
                    .chars()
                    .all(|c| matches!(c, ' ' | '\t' | '\n' | '\r' | '\u{c}')) =>
            {
                continue;
            }
            NodeKind::Text(_) => {
                return Err(RenderError("flex text must be wrapped in an element"));
            }
            NodeKind::Element { name, attributes }
                if matches!(
                    name.as_str(),
                    "head" | "script" | "style" | "title" | "meta" | "link" | "template"
                ) || (name == "input"
                    && attributes
                        .get("type")
                        .is_some_and(|t| t.eq_ignore_ascii_case("hidden"))) =>
            {
                continue;
            }
            _ => {}
        }
        let cs = sheet.computed_style(document, child)?.unwrap_or_default();
        if !cs.visible {
            continue;
        }
        nodes.push(tree.new_leaf(child_style(cs)?).map_err(error)?);
        items.push(RowItem {
            node: child,
            width: 0,
            x: 0,
            style: cs,
        });
    }
    let root = tree
        .new_with_children(
            t::Style {
                size: t::Size {
                    width: t::Dimension::from_length(scalar(width)?),
                    height: t::Dimension::AUTO,
                },
                gap: t::Size {
                    width: t::LengthPercentage::from_length(scalar(style.gap)?),
                    height: t::LengthPercentage::from_length(0.0),
                },
                justify_content: match style.justify_content {
                    JustifyContent::Start => t::JustifyContent::FLEX_START,
                    JustifyContent::Center => t::JustifyContent::CENTER,
                    JustifyContent::End => t::JustifyContent::FLEX_END,
                    JustifyContent::SpaceBetween => t::JustifyContent::SPACE_BETWEEN,
                },
                align_items: match style.align_items {
                    AlignItems::Start => t::AlignItems::FLEX_START,
                    AlignItems::Center => t::AlignItems::CENTER,
                    AlignItems::End => t::AlignItems::FLEX_END,
                    AlignItems::Stretch => t::AlignItems::STRETCH,
                },
                ..Default::default()
            },
            &nodes,
        )
        .map_err(error)?;
    tree.compute_layout(
        root,
        t::Size {
            width: t::AvailableSpace::Definite(scalar(width)?),
            height: t::AvailableSpace::MaxContent,
        },
    )
    .map_err(error)?;
    for (item, &id) in items.iter_mut().zip(&nodes) {
        let l = tree.layout(id).map_err(error)?;
        item.x = pixel(l.location.x - item.style.margin.left as f32)?;
        item.width = pixel(
            l.size.width - l.padding.left - l.padding.right - l.border.left - l.border.right,
        )?;
    }
    Ok((
        items,
        RowEngine {
            tree,
            root,
            nodes,
            width,
        },
    ))
}
impl RowEngine {
    /// Measured subtree content heights are fed back into Taffy for cross-axis
    /// alignment/stretch. RWE translates paint records, without an alignment solver.
    pub(crate) fn finish(
        &mut self,
        height: u32,
        natural: &[u32],
    ) -> Result<Vec<(u32, u32)>, RenderError> {
        let mut root_style = self.tree.style(self.root).map_err(error)?.clone();
        root_style.size.height = t::Dimension::from_length(scalar(height)?);
        self.tree.set_style(self.root, root_style).map_err(error)?;
        for (&id, &content_height) in self.nodes.iter().zip(natural) {
            let mut style = self.tree.style(id).map_err(error)?.clone();
            if style.size.height == t::Dimension::AUTO {
                style.min_size.height =
                    t::LengthPercentageAuto::from_length(scalar(content_height)?);
            }
            self.tree.set_style(id, style).map_err(error)?;
        }
        self.tree
            .compute_layout(
                self.root,
                t::Size {
                    width: t::AvailableSpace::Definite(scalar(self.width)?),
                    height: t::AvailableSpace::Definite(scalar(height)?),
                },
            )
            .map_err(error)?;
        self.nodes
            .iter()
            .map(|&id| {
                let l = self.tree.layout(id).map_err(error)?;
                Ok((pixel(l.location.y)?, pixel(l.size.height)?))
            })
            .collect()
    }
}
