//! Single-line text control presentation shared by static and live pages.
use std::collections::BTreeMap;
pub(crate) fn is_text_input(name: &str, attributes: &BTreeMap<String, String>) -> bool {
    name == "input"
        && attributes
            .get("type")
            .is_none_or(|t| t.eq_ignore_ascii_case("text") || t.eq_ignore_ascii_case("search"))
}
pub(crate) struct InputView {
    pub text: String,
    pub placeholder: bool,
    pub caret: Option<u32>,
    pub selection: Option<(u32, u32)>,
    pub underline: Option<(u32, u32)>,
}
pub(crate) fn input_view(
    attributes: &BTreeMap<String, String>,
    width: u32,
    size: u32,
) -> InputView {
    let composing = attributes.contains_key("data-rwe-display-value");
    let value = attributes
        .get(if composing {
            "data-rwe-display-value"
        } else {
            "value"
        })
        .map(String::as_str)
        .unwrap_or("");
    let focused = attributes.contains_key("data-rwe-focus");
    let column = |key: &str| {
        let offset = attributes
            .get(key)
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);
        let mut units = 0;
        value
            .chars()
            .take_while(|c| {
                units += c.len_utf16();
                units <= offset
            })
            .count()
    };
    let caret = column(if composing {
        "data-rwe-display-caret"
    } else {
        "data-rwe-caret"
    });
    let chars: Vec<_> = value.chars().filter(|c| !c.is_control()).collect();
    let mut positions = Vec::with_capacity(chars.len() + 1);
    positions.push(0u32);
    for &ch in &chars {
        positions.push(
            positions
                .last()
                .unwrap()
                .saturating_add(crate::text::advance(ch, size)),
        );
    }
    let caret_index = caret.min(chars.len());
    let start = if focused {
        let minimum = positions[caret_index].saturating_sub(width.saturating_sub(1));
        positions[..=caret_index].partition_point(|&x| x < minimum)
    } else {
        0
    };
    let origin = positions[start];
    let placeholder = value.is_empty();
    let source = if placeholder {
        attributes
            .get("placeholder")
            .map(String::as_str)
            .unwrap_or("")
    } else {
        value
    };
    let mut used = 0;
    let mut text = String::new();
    for ch in source
        .chars()
        .filter(|c| !c.is_control())
        .skip(if placeholder { 0 } else { start })
    {
        let next = crate::text::advance(ch, size);
        if next > width.saturating_sub(used) {
            break;
        }
        text.push(ch);
        used += next;
    }
    let position = |key| {
        positions[column(key).min(chars.len())]
            .saturating_sub(origin)
            .min(width)
    };
    let selected_start = position(if composing {
        "data-rwe-display-start"
    } else {
        "data-rwe-selection-start"
    });
    let selected_end = position(if composing {
        "data-rwe-display-end"
    } else {
        "data-rwe-selection-end"
    });
    let preedit_start = position("data-rwe-preedit-start");
    let preedit_end = position("data-rwe-preedit-end");
    InputView {
        text,
        placeholder,
        caret: (focused
            && !attributes.contains_key("readonly")
            && !attributes.contains_key("data-rwe-hide-caret")
            && width > 0)
            .then_some(
                positions[caret_index]
                    .saturating_sub(origin)
                    .min(width.saturating_sub(1)),
            ),
        underline: (focused && composing && preedit_end > preedit_start)
            .then_some((preedit_start, preedit_end)),
        selection: (focused && selected_end > selected_start)
            .then_some((selected_start, selected_end)),
    }
}

pub(crate) fn is_toggle(name: &str, attributes: &BTreeMap<String, String>) -> bool {
    name == "input"
        && attributes
            .get("type")
            .is_some_and(|t| t.eq_ignore_ascii_case("checkbox") || t.eq_ignore_ascii_case("radio"))
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ToggleView {
    pub radio: bool,
    pub checked: bool,
    pub indeterminate: bool,
    pub disabled: bool,
    pub size: u32,
}
impl ToggleView {
    pub fn from_attributes(attributes: &BTreeMap<String, String>, size: u32) -> Self {
        Self {
            radio: attributes
                .get("type")
                .is_some_and(|t| t.eq_ignore_ascii_case("radio")),
            checked: attributes.contains_key("checked"),
            indeterminate: attributes.contains_key("data-rwe-indeterminate"),
            disabled: attributes.contains_key("disabled"),
            size,
        }
    }
    pub fn paint(self, canvas: &mut crate::Canvas, content: crate::Rect, clip: crate::Rect) {
        use crate::{Color, Rect};
        let Some(clip) = content.intersection(clip) else {
            return;
        };
        let border = if self.disabled {
            Color::rgba(157, 168, 181, 255)
        } else {
            Color::rgba(97, 117, 139, 255)
        };
        let paper = if self.disabled {
            Color::rgba(229, 234, 240, 255)
        } else {
            Color::WHITE
        };
        let accent = if self.disabled {
            border
        } else {
            Color::rgba(37, 99, 235, 255)
        };
        let stroke = (self.size / 8).max(1);
        let size = self.size;
        if self.radio {
            let radius = i64::from(size - 1);
            for y in 0..size {
                for x in 0..size {
                    let dx = 2 * i64::from(x) - radius;
                    let dy = 2 * i64::from(y) - radius;
                    let d = dx * dx + dy * dy;
                    if d > radius * radius {
                        continue;
                    }
                    let inner = (radius - 2 * i64::from(stroke)).max(0);
                    let color = if d > inner * inner {
                        border
                    } else if self.checked && d <= i64::from(size / 2).pow(2) {
                        accent
                    } else {
                        paper
                    };
                    canvas.fill_rect_clipped(
                        Rect::new(content.x + x as i32, content.y + y as i32, 1, 1),
                        color,
                        clip,
                    );
                }
            }
        } else {
            let active = self.checked || self.indeterminate;
            canvas.fill_rect_clipped(
                Rect::new(content.x, content.y, size, size),
                if active { accent } else { border },
                clip,
            );
            if !active {
                canvas.fill_rect_clipped(
                    Rect::new(
                        content.x + stroke as i32,
                        content.y + stroke as i32,
                        size.saturating_sub(2 * stroke),
                        size.saturating_sub(2 * stroke),
                    ),
                    paper,
                    clip,
                );
            } else if self.indeterminate {
                canvas.fill_rect_clipped(
                    Rect::new(
                        content.x + (size / 4) as i32,
                        content.y + (size / 2 - stroke / 2) as i32,
                        size / 2,
                        stroke,
                    ),
                    Color::WHITE,
                    clip,
                );
            } else {
                let point = |x, y| {
                    (
                        content.x + (x * size / 16) as i32,
                        content.y + (y * size / 16) as i32,
                    )
                };
                for (from, to) in [(point(3, 8), point(6, 11)), (point(6, 11), point(12, 4))] {
                    let (mut x, mut y) = from;
                    let dx = (to.0 - x).abs();
                    let sx = if x < to.0 { 1 } else { -1 };
                    let dy = -(to.1 - y).abs();
                    let sy = if y < to.1 { 1 } else { -1 };
                    let mut error = dx + dy;
                    loop {
                        canvas.fill_rect_clipped(
                            Rect::new(x, y, stroke, stroke),
                            Color::WHITE,
                            clip,
                        );
                        if (x, y) == to {
                            break;
                        }
                        let e = 2 * error;
                        if e >= dy {
                            error += dy;
                            x += sx;
                        }
                        if e <= dx {
                            error += dx;
                            y += sy;
                        }
                    }
                }
            }
        }
    }
}
