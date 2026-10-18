//! Input-method transitions independent of native window APIs.
/// A single-line input-method transition. Preedit cursor offsets are UTF-8 byte
/// boundaries, matching winit; the DOM bridge converts them to UTF-16 offsets.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImeEvent {
    /// Updates transient text without changing the committed input value.
    /// Empty text clears the preview but preserves the session until commit/cancel.
    Preedit {
        /// Current composition text, limited to 65536 UTF-16 units and 256 KiB.
        text: String,
        /// Cursor/selection byte range in text; None hides the preedit caret.
        cursor: Option<(usize, usize)>,
    },
    /// Commits text once, replacing the selection saved at composition start.
    Commit(String),
    /// Discards transient text, for example on native IME disable or focus loss.
    Cancel,
}
impl ImeEvent {
    pub(crate) fn validate(&self) -> Result<(), String> {
        match self {
            Self::Preedit { text, cursor } => {
                crate::keyboard::validate_text(text)?;
                if text.encode_utf16().count() > 65536 {
                    return Err("preedit exceeds 65536 UTF-16 units".into());
                }
                if let Some((start, end)) = cursor {
                    if start > end
                        || *end > text.len()
                        || !text.is_char_boundary(*start)
                        || !text.is_char_boundary(*end)
                    {
                        return Err("preedit cursor must be an ordered UTF-8 boundary range".into());
                    }
                }
            }
            Self::Commit(text) => crate::keyboard::validate_text(text)?,
            Self::Cancel => {}
        }
        Ok(())
    }
}
