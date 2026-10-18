//! Host-independent keyboard input for live applications.
/// A key transition using DOM key names, for example `Tab`, `Enter`, or ` `.
/// Optional committed native key text is supported. Composition/preedit uses
/// `ImeEvent`; physical key codes are not provided.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyEvent {
    /// Logical DOM key name or character, limited to 64 UTF-8 bytes.
    pub key: String,
    /// Optional native text; `None` falls back to a one-character logical key.
    /// An empty string suppresses insertion for dead keys or composition.
    pub text: Option<String>,
    /// True for keydown, false for keyup.
    pub pressed: bool,
    /// Whether this is an operating-system key repeat.
    pub repeat: bool,
    /// Shift modifier state.
    pub shift_key: bool,
    /// Control modifier state.
    pub ctrl_key: bool,
    /// Alt/Option modifier state.
    pub alt_key: bool,
    /// Meta/Command modifier state.
    pub meta_key: bool,
}
impl KeyEvent {
    /// Creates an unmodified keydown transition.
    pub fn down(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            text: None,
            pressed: true,
            repeat: false,
            shift_key: false,
            ctrl_key: false,
            alt_key: false,
            meta_key: false,
        }
    }
    /// Creates an unmodified keyup transition.
    pub fn up(key: impl Into<String>) -> Self {
        Self {
            pressed: false,
            ..Self::down(key)
        }
    }
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.key.is_empty() || self.key.len() > 64 || self.key.chars().any(char::is_control) {
            return Err("key must contain 1..64 UTF-8 bytes without control characters".into());
        }
        if let Some(text) = &self.text {
            validate_text(text)?;
        }
        Ok(())
    }
}

pub(crate) fn validate_text(text: &str) -> Result<(), String> {
    if text.len() > 262144 || text.chars().any(char::is_control) {
        return Err("text must be at most 256 KiB without control characters".into());
    }
    Ok(())
}
