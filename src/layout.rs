//! Author styles, block flow and bounded flex-row layout.
use crate::flex::{AlignItems, JustifyContent, RowEngine, RowSpan, plan_row};
use crate::text::{TextStyle, layout_text};
use crate::{BorderStyle, Edges, Overflow, TextAlign, TextRun, WhiteSpace};
use crate::{Canvas, Color, Document, NodeId, NodeKind, Rect, Stylesheet};
use std::{error::Error, fmt};

/// Invalid or unsupported style, geometry, or render allocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderError(pub &'static str);
impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}
impl Error for RenderError {}

/// Supported block and text declarations; dimensions use content-box sizing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoxStyle {
    /// Content width; `None` fills available width after margin, padding, and border.
    pub width: Option<u32>,
    /// Single horizontal, nonwrapping flex row instead of vertical blocks.
    pub flex: bool,
    /// Explicit pixel flex basis; falls back to width. Intrinsic auto bases are unsupported.
    pub flex_basis: Option<u32>,
    /// Nonnegative integer free-space growth weight.
    pub flex_grow: u32,
    /// Nonnegative integer shrink factor, scaled by the original content basis.
    pub flex_shrink: u32,
    /// Pixel gap between flex items.
    pub gap: u32,
    /// Horizontal flex distribution.
    pub justify_content: JustifyContent,
    /// Vertical flex alignment.
    pub align_items: AlignItems,
    /// Content height; `None` fits children.
    pub height: Option<u32>,
    /// Nonnegative physical margins, without collapsing.
    pub margin: Edges,
    /// Nonnegative physical padding.
    pub padding: Edges,
    /// Uniform specified border width; used only with a solid style.
    pub border_width: u32,
    /// Uniform border style.
    pub border_style: BorderStyle,
    /// Border color; `None` resolves to this element's inherited text color.
    pub border_color: Option<Color>,
    /// Descendant painting behavior; this property does not inherit.
    pub overflow: Overflow,
    /// Background covering the complete border box, including beneath the border.
    pub background: Color,
    /// Whether this element and its descendants participate in layout.
    pub visible: bool,
    /// Foreground color; `None` inherits from the containing element.
    pub color: Option<Color>,
    /// Bitmap cell height: multiples of 8px from 8px through 64px; `None` inherits.
    pub font_size: Option<u32>,
    /// Text line alignment; `None` inherits from the containing element.
    pub text_align: Option<TextAlign>,
    /// Whitespace processing; `None` inherits from the containing element.
    pub white_space: Option<WhiteSpace>,
}
impl Default for BoxStyle {
    fn default() -> Self {
        Self {
            width: None,
            flex: false,
            flex_basis: None,
            flex_grow: 0,
            flex_shrink: 1,
            gap: 0,
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Stretch,
            height: None,
            margin: Edges::all(0),
            padding: Edges::all(0),
            border_width: 0,
            border_style: BorderStyle::None,
            border_color: None,
            overflow: Overflow::Visible,
            background: Color::TRANSPARENT,
            visible: true,
            color: None,
            font_size: None,
            text_align: None,
            white_space: None,
        }
    }
}
impl BoxStyle {
    /// Parses declarations through cssparser. Unsupported properties/values are errors.
    ///
    /// Supports content dimensions, margin/padding shorthands and side longhands,
    /// uniform solid/none borders, background-color, display, and text properties.
    /// Lengths are integer `px` or zero;
    /// width and height also accept `auto`. Repeated declarations use the last value.
    pub fn parse(source: &str) -> Result<Self, RenderError> {
        let mut result = Self::default();
        for declaration in crate::css::declarations(source)? {
            result.apply_property(&declaration.name, &declaration.value)?;
        }
        Ok(result)
    }
    pub(crate) fn apply_property(&mut self, key: &str, value: &str) -> Result<(), RenderError> {
        let result = self;
        match key {
            "width" => result.width = dimension(value)?,
            "flex-basis" => result.flex_basis = dimension(value)?,
            "flex-grow" => {
                if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
                    return Err(RenderError("flex-grow must be a nonnegative integer"));
                }
                result.flex_grow = value
                    .parse()
                    .map_err(|_| RenderError("flex-grow overflow"))?;
            }
            "flex-shrink" => {
                if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
                    return Err(RenderError("flex-shrink must be a nonnegative integer"));
                }
                result.flex_shrink = value
                    .parse()
                    .map_err(|_| RenderError("flex-shrink overflow"))?;
            }
            "gap" => result.gap = length(value)?,
            "justify-content" => {
                result.justify_content = match value.to_ascii_lowercase().as_str() {
                    "flex-start" => JustifyContent::Start,
                    "center" => JustifyContent::Center,
                    "flex-end" => JustifyContent::End,
                    "space-between" => JustifyContent::SpaceBetween,
                    _ => return Err(RenderError("unsupported justify-content")),
                }
            }
            "align-items" => {
                result.align_items = match value.to_ascii_lowercase().as_str() {
                    "stretch" => AlignItems::Stretch,
                    "flex-start" => AlignItems::Start,
                    "center" => AlignItems::Center,
                    "flex-end" => AlignItems::End,
                    _ => return Err(RenderError("unsupported align-items")),
                }
            }
            "height" => result.height = dimension(value)?,
            "margin" => result.margin = Edges::parse(value)?,
            "padding" => result.padding = Edges::parse(value)?,
            "margin-top" => result.margin.top = length(value)?,
            "margin-right" => result.margin.right = length(value)?,
            "margin-bottom" => result.margin.bottom = length(value)?,
            "margin-left" => result.margin.left = length(value)?,
            "padding-top" => result.padding.top = length(value)?,
            "padding-right" => result.padding.right = length(value)?,
            "padding-bottom" => result.padding.bottom = length(value)?,
            "padding-left" => result.padding.left = length(value)?,
            "border-width" => result.border_width = length(value)?,
            "border-style" => result.border_style = BorderStyle::parse(value)?,
            "border-color" => result.border_color = crate::boxmodel::border_color(value)?,
            "border" => {
                if value.eq_ignore_ascii_case("none") {
                    result.border_width = 0;
                    result.border_style = BorderStyle::None;
                    result.border_color = None;
                } else {
                    let parts = value.split_ascii_whitespace().collect::<Vec<_>>();
                    let [width, style, color] = parts.as_slice() else {
                        return Err(RenderError("border must be 'width style color' or none"));
                    };
                    result.border_width = length(width)?;
                    result.border_style = BorderStyle::parse(style)?;
                    result.border_color = crate::boxmodel::border_color(color)?;
                }
            }
            "background-color" => {
                result.background = if value.eq_ignore_ascii_case("transparent") {
                    Color::TRANSPARENT
                } else {
                    Color::from_css_hex(value)
                        .map_err(|_| RenderError("unsupported background color"))?
                }
            }
            "color" => {
                result.color = Some(if value.eq_ignore_ascii_case("transparent") {
                    Color::TRANSPARENT
                } else {
                    Color::from_css_hex(value).map_err(|_| RenderError("unsupported text color"))?
                })
            }
            "font-size" => {
                let size = length(value)?;
                if !(8..=64).contains(&size) || size % 8 != 0 {
                    return Err(RenderError(
                        "font-size must be a multiple of 8px from 8px to 64px",
                    ));
                }
                result.font_size = Some(size);
            }
            "text-align" => {
                result.text_align = Some(match value.to_ascii_lowercase().as_str() {
                    "left" => TextAlign::Left,
                    "center" => TextAlign::Center,
                    "right" => TextAlign::Right,
                    _ => return Err(RenderError("text-align must be left, center, or right")),
                })
            }
            "white-space" => {
                result.white_space = Some(match value.to_ascii_lowercase().as_str() {
                    "normal" => WhiteSpace::Normal,
                    "pre" => WhiteSpace::Pre,
                    _ => return Err(RenderError("white-space must be normal or pre")),
                })
            }
            "overflow" => {
                result.overflow = match value.to_ascii_lowercase().as_str() {
                    "visible" => Overflow::Visible,
                    "hidden" => Overflow::Hidden,
                    _ => return Err(RenderError("overflow must be visible or hidden")),
                }
            }
            "display" => match value.to_ascii_lowercase().as_str() {
                "block" => {
                    result.visible = true;
                    result.flex = false;
                }
                "flex" => {
                    result.visible = true;
                    result.flex = true;
                }
                "none" => {
                    result.visible = false;
                    result.flex = false;
                }
                _ => return Err(RenderError("display must be block, flex or none")),
            },
            _ => return Err(RenderError("unsupported inline property")),
        }
        Ok(())
    }
}

fn dimension(value: &str) -> Result<Option<u32>, RenderError> {
    if value.eq_ignore_ascii_case("auto") {
        Ok(None)
    } else {
        length(value).map(Some)
    }
}
pub(crate) fn length(value: &str) -> Result<u32, RenderError> {
    if value == "0" {
        return Ok(0);
    }
    let normalized = value.to_ascii_lowercase();
    let digits = normalized
        .strip_suffix("px")
        .ok_or(RenderError("expected integer px length"))?;
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return Err(RenderError("expected nonnegative integer px length"));
    }
    let pixels: u32 = digits.parse().map_err(|_| RenderError("length overflow"))?;
    if pixels > i32::MAX as u32 {
        return Err(RenderError("length exceeds coordinate range"));
    }
    Ok(pixels)
}
pub(crate) fn add(a: u32, b: u32) -> Result<u32, RenderError> {
    a.checked_add(b)
        .filter(|&v| v <= i32::MAX as u32)
        .ok_or(RenderError("layout coordinate overflow"))
}

/// A painted border box, in parent-before-child source order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayoutBox {
    /// Element that generated this box.
    pub node: NodeId,
    /// Content plus padding and border, excluding margins.
    pub rect: Rect,
    /// Content box excluding padding, border, and margins.
    pub content_rect: Rect,
    /// Padding-box clip applied only to descendants for overflow: hidden.
    pub clip_rect: Option<Rect>,
    /// Effective border width, zero when the style is none.
    pub border_width: u32,
    /// Resolved border color.
    pub border_color: Color,
    /// Color composited over previously painted boxes.
    pub background: Color,
}

/// Blocks and explicit-basis flex rows with wrapped or preformatted text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockLayout {
    boxes: Vec<LayoutBox>,
    text_runs: Vec<TextRun>,
    paint_order: Vec<PaintItem>,
    width: u32,
    height: u32,
}
impl BlockLayout {
    /// Lays out visible elements using inline styles only.
    ///
    /// Children may overflow fixed sizes. Overflow is visible until canvas clipping.
    /// Text properties inherit. Border sizing is content-box; margins do not collapse.
    pub fn new(document: &Document, viewport_width: u32) -> Result<Self, RenderError> {
        Self::with_stylesheet(document, &Stylesheet::default(), viewport_width)
    }
    /// Lays out blocks using author stylesheet rules followed by inline overrides.
    /// Hidden subtrees are skipped, including descendant inline style validation.
    pub fn with_stylesheet(
        document: &Document,
        stylesheet: &Stylesheet,
        viewport_width: u32,
    ) -> Result<Self, RenderError> {
        add(viewport_width, 0)?;
        let mut layout = Self {
            boxes: vec![],
            text_runs: vec![],
            paint_order: vec![],
            width: viewport_width,
            height: 0,
        };
        let mut stack = vec![Task::Enter {
            node: 0,
            x: 0,
            y: 0,
            width: viewport_width,
        }];
        let mut cursors = vec![0; document.nodes().len()];
        let mut text_styles = vec![TextStyle::default(); document.nodes().len()];
        let mut row_parents = vec![false; document.nodes().len()];
        let mut allocated = vec![None; document.nodes().len()];
        let mut row_spans: Vec<Vec<RowSpan>> = vec![vec![]; document.nodes().len()];
        let mut row_engines: Vec<Option<RowEngine>> =
            (0..document.nodes().len()).map(|_| None).collect();
        while let Some(task) = stack.pop() {
            match task {
                Task::Enter { node, x, y, width } => {
                    let dom = &document.nodes()[node];
                    if matches!(dom.kind, NodeKind::Comment(_)) {
                        continue;
                    }
                    let inherited = dom
                        .parent
                        .map(|parent| text_styles[parent])
                        .unwrap_or_default();
                    if let NodeKind::Text(source) = &dom.kind {
                        let runs = layout_text(node, source, x, y, width, inherited)?;
                        if let Some(last) = runs.last() {
                            if let Some(parent) = dom.parent {
                                cursors[parent] = add(last.rect.y as u32, last.rect.height)?;
                            }
                        }
                        for run in runs {
                            layout
                                .paint_order
                                .push(PaintItem::Text(layout.text_runs.len()));
                            layout.text_runs.push(run);
                        }
                        continue;
                    }
                    if matches!(&dom.kind, NodeKind::Element { name, .. } if matches!(name.as_str(), "head" | "script" | "style" | "title" | "meta" | "link" | "template"))
                    {
                        continue;
                    }
                    if matches!(&dom.kind, NodeKind::Element { name, attributes } if name == "input" && attributes.get("type").is_some_and(|t| t.eq_ignore_ascii_case("hidden")))
                    {
                        continue;
                    }
                    let style = stylesheet
                        .computed_style(document, node)?
                        .unwrap_or_default();
                    if !style.visible {
                        continue;
                    }
                    text_styles[node] = TextStyle {
                        color: style.color.unwrap_or(inherited.color),
                        size: style.font_size.unwrap_or(inherited.size),
                        align: style.text_align.unwrap_or(inherited.align),
                        white_space: style.white_space.unwrap_or(inherited.white_space),
                    };
                    let border = if style.border_style == BorderStyle::Solid {
                        style.border_width
                    } else {
                        0
                    };
                    let border_pair = add(border, border)?;
                    let horizontal =
                        add(add(style.padding.left, style.padding.right)?, border_pair)?;
                    let margins = add(style.margin.left, style.margin.right)?;
                    let toggle = matches!(&dom.kind, NodeKind::Element {name,attributes} if crate::form_control::is_toggle(name,attributes));
                    let content_width = allocated[node].or(style.width).unwrap_or(if toggle {
                        text_styles[node].size
                    } else {
                        width.saturating_sub(add(horizontal, margins)?)
                    });
                    let outer_width = add(content_width, horizontal)?;
                    let left = add(x, style.margin.left)?;
                    let top = add(y, style.margin.top)?;
                    add(left, outer_width)?;
                    let child_x = add(add(left, border)?, style.padding.left)?;
                    cursors[node] = add(add(top, border)?, style.padding.top)?;
                    let index = layout.boxes.len();
                    layout.paint_order.push(PaintItem::Box(index));
                    layout.boxes.push(LayoutBox {
                        node,
                        rect: Rect::new(left as i32, top as i32, outer_width, 0),
                        content_rect: Rect::new(
                            child_x as i32,
                            cursors[node] as i32,
                            content_width,
                            0,
                        ),
                        clip_rect: None,
                        border_width: border,
                        border_color: style.border_color.unwrap_or(text_styles[node].color),
                        background: style.background,
                    });
                    if style.overflow == Overflow::Hidden {
                        layout.paint_order.push(PaintItem::PushClip(index));
                    }
                    stack.push(Task::Exit {
                        node,
                        index,
                        top,
                        style,
                    });
                    if let NodeKind::Element { name, attributes } = &dom.kind {
                        if crate::form_control::is_toggle(name, attributes) {
                            let mut view = crate::form_control::ToggleView::from_attributes(
                                attributes,
                                text_styles[node].size,
                            );
                            let mut parent = dom.parent;
                            while let Some(id) = parent {
                                let n = &document.nodes()[id];
                                if matches!(&n.kind,NodeKind::Element{name,attributes} if name=="fieldset" && attributes.contains_key("disabled"))
                                {
                                    view.disabled = true;
                                }
                                parent = n.parent;
                            }
                            layout.paint_order.push(PaintItem::Toggle { index, view });
                            cursors[node] = add(cursors[node], view.size)?;
                            continue;
                        }
                        if crate::form_control::is_text_input(name, attributes) {
                            let text_style = text_styles[node];
                            let view = crate::form_control::input_view(
                                attributes,
                                content_width,
                                text_style.size,
                            );
                            let text = layout.text_runs.len();
                            layout.text_runs.push(TextRun {
                                node,
                                rect: Rect::new(
                                    child_x as i32,
                                    cursors[node] as i32,
                                    crate::text::measure(&view.text, text_style.size)?,
                                    text_style.size,
                                ),
                                text: view.text,
                                font_size: text_style.size,
                                color: if view.placeholder {
                                    Color::rgba(112, 120, 132, 255)
                                } else {
                                    text_style.color
                                },
                            });
                            layout.paint_order.push(PaintItem::Input {
                                index,
                                text,
                                caret: view.caret,
                                selection: view.selection,
                                underline: view.underline,
                            });
                            cursors[node] = add(cursors[node], text_style.size)?;
                            continue;
                        }
                    }
                    if style.flex {
                        row_parents[node] = true;
                        let (items, engine) =
                            plan_row(document, stylesheet, node, content_width, style)?;
                        row_engines[node] = Some(engine);
                        stack.push(Task::RowFinish { node, index, style });
                        for item in items.into_iter().rev() {
                            allocated[item.node] = Some(item.width);
                            stack.push(Task::RowStart {
                                node: item.node,
                                parent: node,
                                x: add(child_x, item.x)?,
                                y: cursors[node],
                                width: item.width,
                                style: item.style,
                            });
                        }
                        continue;
                    }
                    for &child in dom.children.iter().rev() {
                        stack.push(Task::Child {
                            node: child,
                            parent: node,
                            x: child_x,
                            width: content_width,
                        });
                    }
                }
                Task::RowStart {
                    node,
                    parent,
                    x,
                    y,
                    width,
                    style,
                } => {
                    stack.push(Task::RowEnd {
                        parent,
                        style,
                        boxes: layout.boxes.len(),
                        text: layout.text_runs.len(),
                    });
                    stack.push(Task::Enter { node, x, y, width });
                }
                Task::RowEnd {
                    parent,
                    style,
                    boxes,
                    text,
                } => {
                    row_spans[parent].push(RowSpan {
                        style,
                        boxes: boxes..layout.boxes.len(),
                        text: text..layout.text_runs.len(),
                    });
                }
                Task::RowFinish { node, index, style } => {
                    let top = layout.boxes[index].content_rect.y as u32;
                    let height = style.height.unwrap_or(cursors[node] - top);
                    let natural: Vec<_> = row_spans[node]
                        .iter()
                        .map(|span| layout.boxes[span.boxes.start].content_rect.height)
                        .collect();
                    let positions = row_engines[node]
                        .take()
                        .expect("row engine exists")
                        .finish(height, &natural)?;
                    for (span, (target_y, target_height)) in row_spans[node].iter().zip(positions) {
                        let root = span.boxes.start;
                        let shift = target_y
                            .checked_sub(span.style.margin.top)
                            .ok_or(RenderError("negative flex alignment is unsupported"))?;
                        let b = &mut layout.boxes[root];
                        let extra = target_height
                            .checked_sub(b.rect.height)
                            .ok_or(RenderError("Taffy cross sizing requires subtree relayout"))?;
                        b.rect.height = target_height;
                        b.content_rect.height = add(b.content_rect.height, extra)?;
                        if let Some(clip) = &mut b.clip_rect {
                            clip.height = add(clip.height, extra)?;
                        }
                        for b in &mut layout.boxes[span.boxes.clone()] {
                            b.rect.y = add(b.rect.y as u32, shift)? as i32;
                            b.content_rect.y = add(b.content_rect.y as u32, shift)? as i32;
                            if let Some(clip) = &mut b.clip_rect {
                                clip.y = add(clip.y as u32, shift)? as i32;
                            }
                            add(b.rect.y as u32, b.rect.height)?;
                        }
                        for run in &mut layout.text_runs[span.text.clone()] {
                            run.rect.y = add(run.rect.y as u32, shift)? as i32;
                        }
                    }
                    cursors[node] = add(top, height)?;
                }
                Task::Child {
                    node,
                    parent,
                    x,
                    width,
                } => {
                    stack.push(Task::Enter {
                        node,
                        x,
                        y: cursors[parent],
                        width,
                    });
                }
                Task::Exit {
                    node,
                    index,
                    top,
                    style,
                } => {
                    let border = layout.boxes[index].border_width;
                    let content_top = layout.boxes[index].content_rect.y as u32;
                    let content_height = style.height.unwrap_or(cursors[node] - content_top);
                    let vertical = add(
                        add(style.padding.top, style.padding.bottom)?,
                        add(border, border)?,
                    )?;
                    let height = add(content_height, vertical)?;
                    let bottom = add(top, height)?;
                    layout.boxes[index].rect.height = height;
                    layout.boxes[index].content_rect.height = content_height;
                    if style.overflow == Overflow::Hidden {
                        let rect = layout.boxes[index].rect;
                        layout.boxes[index].clip_rect = Some(Rect::new(
                            add(rect.x as u32, border)? as i32,
                            add(rect.y as u32, border)? as i32,
                            rect.width - 2 * border,
                            rect.height - 2 * border,
                        ));
                        layout.paint_order.push(PaintItem::PopClip);
                    }
                    if let Some(parent) = document.nodes()[node].parent {
                        let next = add(bottom, style.margin.bottom)?;
                        cursors[parent] = if row_parents[parent] {
                            cursors[parent].max(next)
                        } else {
                            next
                        };
                    } else {
                        layout.height = bottom;
                    }
                }
            }
        }
        Ok(layout)
    }
    /// Boxes in paint order, including the transparent synthetic document box.
    pub fn boxes(&self) -> &[LayoutBox] {
        &self.boxes
    }
    /// Text lines in source order with resolved foreground styles.
    pub fn text_runs(&self) -> &[TextRun] {
        &self.text_runs
    }
    /// Width supplied at layout time.
    pub fn width(&self) -> u32 {
        self.width
    }
    /// Height of the root flow; visible overflow may extend beyond it.
    pub fn height(&self) -> u32 {
        self.height
    }
    #[cfg(feature = "javascript")]
    pub(crate) fn input_caret_rect(&self, node: NodeId) -> Option<Rect> {
        self.paint_order.iter().find_map(|item| {
            if let PaintItem::Input {
                index,
                text,
                caret,
                underline,
                ..
            } = item
            {
                let b = &self.boxes[*index];
                if b.node == node {
                    let column = caret.or(underline.map(|(_, end)| end)).unwrap_or(0);
                    return Some(Rect::new(
                        (i64::from(b.content_rect.x) + i64::from(column)) as i32,
                        b.content_rect.y,
                        1,
                        self.text_runs[*text].font_size,
                    ));
                }
            }
            None
        })
    }

    /// Paints backgrounds and text in source order with ancestor and canvas clips.
    pub fn paint(&self, canvas: &mut Canvas) {
        let mut clips = vec![Rect::new(0, 0, canvas.width(), canvas.height())];
        for item in &self.paint_order {
            let clip = *clips.last().expect("base canvas clip is retained");
            match *item {
                PaintItem::PushClip(index) => {
                    let inner = self.boxes[index].clip_rect.expect("hidden box has a clip");
                    clips.push(clip.intersection(inner).unwrap_or(Rect::new(0, 0, 0, 0)));
                }
                PaintItem::PopClip => {
                    clips.pop();
                }
                PaintItem::Box(index) => {
                    let item = &self.boxes[index];
                    canvas.fill_rect_clipped(item.rect, item.background, clip);
                    let b = item.border_width;
                    if b > 0 {
                        let r = item.rect;
                        let right = (i64::from(r.x) + i64::from(r.width - b)) as i32;
                        let bottom = (i64::from(r.y) + i64::from(r.height - b)) as i32;
                        let middle_y = (i64::from(r.y) + i64::from(b)) as i32;
                        let middle_height = r.height - 2 * b;
                        for edge in [
                            Rect::new(r.x, r.y, r.width, b),
                            Rect::new(r.x, bottom, r.width, b),
                            Rect::new(r.x, middle_y, b, middle_height),
                            Rect::new(right, middle_y, b, middle_height),
                        ] {
                            canvas.fill_rect_clipped(edge, item.border_color, clip);
                        }
                    }
                }
                PaintItem::Input {
                    index,
                    text,
                    caret,
                    selection,
                    underline,
                } => {
                    let content = self.boxes[index].content_rect;
                    if let Some(clip) = clip.intersection(content) {
                        let run = &self.text_runs[text];
                        if let Some((start, end)) = selection {
                            canvas.fill_rect_clipped(
                                Rect::new(
                                    (i64::from(content.x) + i64::from(start)) as i32,
                                    content.y,
                                    end - start,
                                    run.font_size,
                                ),
                                Color::rgba(191, 219, 254, 255),
                                clip,
                            );
                        }
                        run.paint(canvas, clip);
                        if let Some((start, end)) = underline {
                            canvas.fill_rect_clipped(
                                Rect::new(
                                    (i64::from(content.x) + i64::from(start)) as i32,
                                    content.y + (run.font_size - 1) as i32,
                                    end - start,
                                    1,
                                ),
                                run.color,
                                clip,
                            );
                        }
                        if let Some(column) = caret {
                            canvas.fill_rect_clipped(
                                Rect::new(
                                    (i64::from(content.x) + i64::from(column)) as i32,
                                    content.y,
                                    1,
                                    run.font_size,
                                ),
                                run.color,
                                clip,
                            );
                        }
                    }
                }
                PaintItem::Toggle { index, view } => {
                    view.paint(canvas, self.boxes[index].content_rect, clip)
                }
                PaintItem::Text(index) => self.text_runs[index].paint(canvas, clip),
            }
        }
    }
}

enum Task {
    RowStart {
        node: NodeId,
        parent: NodeId,
        x: u32,
        y: u32,
        width: u32,
        style: BoxStyle,
    },
    RowEnd {
        parent: NodeId,
        style: BoxStyle,
        boxes: usize,
        text: usize,
    },
    RowFinish {
        node: NodeId,
        index: usize,
        style: BoxStyle,
    },
    Enter {
        node: NodeId,
        x: u32,
        y: u32,
        width: u32,
    },
    Child {
        node: NodeId,
        parent: NodeId,
        x: u32,
        width: u32,
    },
    Exit {
        node: NodeId,
        index: usize,
        top: u32,
        style: BoxStyle,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PaintItem {
    Toggle {
        index: usize,
        view: crate::form_control::ToggleView,
    },
    Input {
        index: usize,
        text: usize,
        caret: Option<u32>,
        selection: Option<(u32, u32)>,
        underline: Option<(u32, u32)>,
    },
    PushClip(usize),
    PopClip,
    Box(usize),
    Text(usize),
}
