//! Immutable UTF-8 assets for applications distributed as a single executable.
use std::{collections::BTreeMap, sync::Arc};

/// An application entry page and its embedded HTML, CSS and classic JavaScript.
/// Asset names are relative virtual paths; no filesystem fallback is performed.
#[derive(Clone, Debug)]
pub struct AssetBundle {
    entry: String,
    sources: Arc<BTreeMap<String, String>>,
}
impl AssetBundle {
    /// Copies and validates up to 256 UTF-8 assets, each at most 1 MiB,
    /// with an 8 MiB total budget. Use `include_str!` to embed build-time files.
    pub fn new(entry: &str, assets: &[(&str, &str)]) -> Result<Self, String> {
        if assets.len() > 256 {
            return Err("bundle exceeds 256 assets".into());
        }
        let entry = normalize(entry)?;
        let mut sources = BTreeMap::new();
        let mut total = 0;
        for &(name, source) in assets {
            let name = normalize(name)?;
            if source.len() > 1_048_576 {
                return Err(format!("{name} exceeds the 1 MiB source limit"));
            }
            total += source.len();
            if total > 8 * 1_048_576 {
                return Err("bundle exceeds 8 MiB resource budget".into());
            }
            if sources.insert(name.clone(), source.to_owned()).is_some() {
                return Err(format!("duplicate embedded asset: {name}"));
            }
        }
        if !sources.contains_key(&entry) {
            return Err(format!("missing embedded entry: {entry}"));
        }
        Ok(Self {
            entry,
            sources: Arc::new(sources),
        })
    }
    /// Returns the normalized virtual entry path.
    pub fn entry(&self) -> &str {
        &self.entry
    }
    pub(crate) fn read(&self, name: &str) -> Result<String, String> {
        self.sources
            .get(name)
            .cloned()
            .ok_or_else(|| format!("missing embedded asset: {name}"))
    }
    pub(crate) fn resolve(&self, reference: &str) -> Result<String, String> {
        validate(reference)?;
        let parent = self.entry.rsplit_once('/').map(|(parent, _)| parent);
        normalize(&parent.map_or_else(|| reference.to_owned(), |p| format!("{p}/{reference}")))
    }
}
fn validate(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name.starts_with('/')
        || name.contains([':', '?', '#', '\\'])
        || name.chars().any(char::is_control)
    {
        return Err(format!("unsupported embedded resource: {name}"));
    }
    Ok(())
}
fn normalize(name: &str) -> Result<String, String> {
    validate(name)?;
    let mut parts = Vec::new();
    for part in name.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                if parts.pop().is_none() {
                    return Err(format!("resource escapes embedded root: {name}"));
                }
            }
            _ => parts.push(part),
        }
    }
    if parts.is_empty() {
        return Err("embedded asset path is empty".into());
    }
    Ok(parts.join("/"))
}
