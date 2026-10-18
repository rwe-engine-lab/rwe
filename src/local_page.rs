//! Bounded local document and resource loading; no networking or browser recovery.
use crate::{Document, NodeKind, Stylesheet};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

/// A classic script loaded in document order.
pub struct PageScript {
    /// Script contents. Module scripts must be bundled as classic/IIFE scripts.
    pub source: String,
    /// Source filename or an inline-script label.
    pub name: String,
    /// Whether this is a module script, which the experimental host rejects.
    pub module: bool,
}

/// A parsed local page with embedded/linked CSS and classic scripts.
pub struct LocalPage {
    /// The document, including metadata and script nodes (not painted).
    pub document: Document,
    /// Author rules, ordered by document position, with explicit CSS last.
    pub stylesheet: Stylesheet,
    /// Scripts in source order; not executed by this loader.
    pub scripts: Vec<PageScript>,
    /// Loaded filesystem paths or embedded virtual paths, in document order.
    pub resources: Vec<PathBuf>,
    /// Original UTF-8 HTML.
    pub html: String,
    /// Combined author CSS.
    pub css: String,
}

/// Reads at most 1 MiB of UTF-8 from a local file.
pub fn read_local_source(path: &Path) -> Result<String, String> {
    let mut bytes = Vec::new();
    File::open(path)
        .and_then(|f| f.take(1_048_577).read_to_end(&mut bytes))
        .map_err(|e| format!("cannot open/read {}: {e}", path.display()))?;
    if bytes.len() > 1_048_576 {
        return Err(format!("{} exceeds the 1 MiB source limit", path.display()));
    }
    String::from_utf8(bytes).map_err(|_| format!("{} is not valid UTF-8", path.display()))
}

impl LocalPage {
    /// Loads local CSS/scripts referenced by a page. References must resolve
    /// inside the HTML directory; URLs, query strings and fragments are rejected.
    /// Individual sources/combined CSS are limited to 1 MiB, all sources to 8 MiB,
    /// and resource references to 32. Explicit author CSS is appended last.
    pub fn load(path: &Path, explicit_css: Option<&Path>) -> Result<Self, String> {
        let html = read_local_source(path)?;
        let base = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."))
            .canonicalize()
            .map_err(|e| e.to_string())?;
        Self::parse_sources(
            html,
            path.to_path_buf(),
            || {
                explicit_css
                    .map(|path| read_local_source(path).map(|text| (path.to_path_buf(), text)))
                    .transpose()
            },
            |reference| {
                if reference.is_empty()
                    || reference.contains([':', '?', '#', '\\'])
                    || Path::new(reference).is_absolute()
                {
                    return Err(format!("unsupported local resource: {reference}"));
                }
                let target = base
                    .join(reference)
                    .canonicalize()
                    .map_err(|e| format!("{reference}: {e}"))?;
                if !target.starts_with(&base) {
                    return Err(format!("resource escapes page directory: {reference}"));
                }
                let text = read_local_source(&target)?;
                Ok((target, text))
            },
        )
    }

    /// Loads an immutable embedded application without accessing the filesystem.
    pub fn from_assets(assets: &crate::AssetBundle) -> Result<Self, String> {
        Self::parse_sources(
            assets.read(assets.entry())?,
            PathBuf::from(assets.entry()),
            || Ok(None),
            |reference| {
                let name = assets.resolve(reference)?;
                Ok((PathBuf::from(&name), assets.read(&name)?))
            },
        )
    }

    fn parse_sources(
        html: String,
        entry: PathBuf,
        explicit_css: impl FnOnce() -> Result<Option<(PathBuf, String)>, String>,
        mut resolve: impl FnMut(&str) -> Result<(PathBuf, String), String>,
    ) -> Result<Self, String> {
        let document = Document::parse_page(&html).map_err(|e| format!("HTML: {e}"))?;
        let mut resources = vec![entry];
        let mut total = html.len();
        let mut css = String::new();
        let mut scripts = Vec::new();
        let mut references = 0;
        let mut read_resource = |reference: &str| -> Result<String, String> {
            references += 1;
            if references > 32 {
                return Err("page exceeds 32 resource references".into());
            }
            let (target, text) = resolve(reference)?;
            total += text.len();
            if total > 8 * 1_048_576 {
                return Err("page exceeds 8 MiB resource budget".into());
            }
            resources.push(target);
            Ok(text)
        };
        for (id, node) in document.nodes().iter().enumerate() {
            let mut parent = node.parent;
            let mut inert = false;
            while let Some(id) = parent {
                let node = document.node(id).unwrap();
                if matches!(&node.kind, NodeKind::Element { name, .. } if name == "template") {
                    inert = true;
                    break;
                }
                parent = node.parent;
            }
            if inert {
                continue;
            }
            let NodeKind::Element { name, attributes } = &node.kind else {
                continue;
            };
            match name.as_str() {
                "style" => {
                    css.push_str(&document.text_content(id).unwrap_or_default());
                    css.push('\n');
                }
                "link"
                    if attributes.get("rel").is_some_and(|s| {
                        s.split_ascii_whitespace()
                            .any(|s| s.eq_ignore_ascii_case("stylesheet"))
                    }) =>
                {
                    if attributes
                        .get("media")
                        .is_some_and(|s| !s.is_empty() && s != "all" && s != "screen")
                    {
                        continue;
                    }
                    css.push_str(&read_resource(
                        attributes.get("href").ok_or("stylesheet link needs href")?,
                    )?);
                    css.push('\n');
                }
                "script" => {
                    let kind = attributes.get("type").map(|s| s.trim()).unwrap_or("");
                    if !matches!(
                        kind,
                        "" | "text/javascript" | "application/javascript" | "module"
                    ) {
                        continue;
                    }
                    let (source, name) = match attributes.get("src") {
                        Some(src) => (read_resource(src)?, src.clone()),
                        None => (
                            document.text_content(id).unwrap_or_default(),
                            format!("inline script {id}"),
                        ),
                    };
                    scripts.push(PageScript {
                        source,
                        name,
                        module: kind == "module",
                    });
                }
                _ => {}
            }
            if css.len() > 1_048_576 {
                return Err("combined CSS exceeds 1 MiB".into());
            }
        }
        if let Some((path, extra)) = explicit_css()? {
            total += extra.len();
            if total > 8 * 1_048_576 {
                return Err("page exceeds 8 MiB resource budget".into());
            }
            css.push_str(&extra);
            resources.push(path);
        }
        if css.len() > 1_048_576 {
            return Err("combined CSS exceeds 1 MiB".into());
        }
        let stylesheet = Stylesheet::parse(&css).map_err(|e| format!("CSS: {e}"))?;
        Ok(Self {
            document,
            stylesheet,
            scripts,
            resources,
            html,
            css,
        })
    }
}
