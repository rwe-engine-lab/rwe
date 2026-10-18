//! Experimental classic JavaScript host with RWE's own DOM subset.
use crate::{Document, LocalPage, NodeKind};
use boa_engine::{
    Context, Source,
    job::{Job, JobExecutor},
};
use std::{cell::RefCell, collections::VecDeque, rc::Rc};

#[derive(Default)]
struct BoundedJobs {
    jobs: RefCell<VecDeque<Job>>,
    error: RefCell<Option<String>>,
}
impl JobExecutor for BoundedJobs {
    fn enqueue_job(self: Rc<Self>, job: Job, _: &mut Context) {
        if self.jobs.borrow().len() >= 1024 {
            *self.error.borrow_mut() = Some("JavaScript job queue exceeds 1024 jobs".into());
        } else {
            self.jobs.borrow_mut().push_back(job);
        }
    }
    fn run_jobs(self: Rc<Self>, context: &mut Context) -> boa_engine::JsResult<()> {
        for _ in 0..1024 {
            let job = self.jobs.borrow_mut().pop_front();
            let Some(job) = job else { return Ok(()) };
            let result = match job {
                Job::PromiseJob(job) => job.call(context),
                Job::GenericJob(job) => job.call(context),
                _ => Err(boa_engine::JsNativeError::typ()
                    .with_message("unsupported host job type")
                    .into()),
            };
            if let Err(e) = result {
                *self.error.borrow_mut() = Some(format!("JavaScript job: {e}"));
                self.jobs.borrow_mut().clear();
                return Ok(());
            }
        }
        if !self.jobs.borrow().is_empty() {
            self.jobs.borrow_mut().clear();
            *self.error.borrow_mut() = Some("JavaScript job budget exceeded".into());
        }
        Ok(())
    }
}

fn quote(text: &str) -> String {
    serde_json::to_string(text)
        .expect("serializing a string cannot fail")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029")
}
fn parse_html_fragment(
    _: &boa_engine::JsValue,
    args: &[boa_engine::JsValue],
    context: &mut Context,
) -> boa_engine::JsResult<boa_engine::JsValue> {
    let arg = |i: usize, context: &mut Context| -> boa_engine::JsResult<String> {
        Ok(args
            .get(i)
            .cloned()
            .unwrap_or_default()
            .to_string(context)?
            .to_std_string_lossy())
    };
    let source = arg(0, context)?;
    let name = arg(1, context)?;
    let namespace = arg(2, context)?;
    let document = Document::parse_fragment(&source, &name, &namespace)
        .map_err(|e| boa_engine::JsNativeError::range().with_message(e.to_string()))?;
    let rows:Vec<_>=document.nodes().iter().map(|n| match &n.kind {
        NodeKind::Document => serde_json::json!({"type":11}),
        NodeKind::Element{name,attributes}=>serde_json::json!({"type":1,"name":name,"namespace":n.namespace,"attributes":attributes,"parent":n.parent}),
        NodeKind::Text(text)=>serde_json::json!({"type":3,"text":text,"parent":n.parent}),
        NodeKind::Comment(text)=>serde_json::json!({"type":8,"text":text,"parent":n.parent}),
    }).collect();
    boa_engine::JsValue::from_json(&serde_json::Value::Array(rows), context)
}

fn parse_css_declarations(
    _: &boa_engine::JsValue,
    args: &[boa_engine::JsValue],
    context: &mut Context,
) -> boa_engine::JsResult<boa_engine::JsValue> {
    let source = args
        .first()
        .cloned()
        .unwrap_or_default()
        .to_string(context)?
        .to_std_string_lossy();
    let declarations = crate::css::declarations(&source)
        .map_err(|e| boa_engine::JsNativeError::syntax().with_message(e.to_string()))?;
    let rows: Vec<_> = declarations
        .into_iter()
        .map(|d| serde_json::json!([d.name, d.value]))
        .collect();
    boa_engine::JsValue::from_json(&serde_json::Value::Array(rows), context)
}

/// Trusted-local-code JavaScript runtime. This is an experimental DOM host,
/// not an HTML5 browser or a security sandbox.
pub struct ScriptRuntime {
    context: Context,
    jobs: Rc<BoundedJobs>,
    poisoned: bool,
    stylesheet: crate::Stylesheet,
}
impl ScriptRuntime {
    /// Builds the DOM, then evaluates classic scripts in document order.
    /// Module scripts require an external build into a classic/IIFE bundle.
    pub fn load(page: &LocalPage) -> Result<Self, String> {
        if page.scripts.iter().any(|s| s.module) {
            return Err("ES modules are unsupported; build a classic/IIFE bundle first".into());
        }
        let jobs = Rc::new(BoundedJobs::default());
        let mut context = Context::builder()
            .job_executor(jobs.clone())
            .build()
            .map_err(|e| e.to_string())?;
        context
            .runtime_limits_mut()
            .set_loop_iteration_limit(1_000_000);
        context.runtime_limits_mut().set_recursion_limit(512);
        context
            .register_global_builtin_callable(
                boa_engine::js_string!("__rwe_parse_html"),
                3,
                boa_engine::NativeFunction::from_fn_ptr(parse_html_fragment),
            )
            .map_err(|e| e.to_string())?;
        context
            .register_global_builtin_callable(
                boa_engine::js_string!("__rwe_parse_css"),
                1,
                boa_engine::NativeFunction::from_fn_ptr(parse_css_declarations),
            )
            .map_err(|e| e.to_string())?;
        let mut runtime = Self {
            context,
            jobs,
            poisoned: false,
            stylesheet: page.stylesheet.clone(),
        };
        let trace = std::env::var_os("RWE_JS_TRACE").is_some();
        if trace {
            eprintln!("rwe-js: DOM bootstrap");
        }
        runtime.eval(include_str!("dom.js"))?;
        let mut init = String::from("(() => {const n=[document];");
        for (id, node) in page.document.nodes().iter().enumerate().skip(1) {
            match &node.kind {
                NodeKind::Element { name, attributes } => {
                    init.push_str(&format!(
                        "n[{id}]=document.createElementNS({},{});",
                        quote(
                            node.namespace
                                .as_deref()
                                .unwrap_or("http://www.w3.org/1999/xhtml")
                        ),
                        quote(name)
                    ));
                    for (key, value) in attributes {
                        init.push_str(&format!(
                            "n[{id}].setAttribute({},{});",
                            quote(key),
                            quote(value)
                        ));
                    }
                }
                NodeKind::Text(text) => init.push_str(&format!(
                    "n[{id}]=document.createTextNode({});",
                    quote(text)
                )),
                NodeKind::Comment(text) => {
                    init.push_str(&format!("n[{id}]=document.createComment({});", quote(text)))
                }
                NodeKind::Document => continue,
            }
            init.push_str(&format!(
                "(n[{0}].content||n[{0}]).appendChild(n[{id}]);",
                node.parent.unwrap_or(0)
            ));
        }
        init.push_str("document.activeElement=document.body;})();");
        if trace {
            eprintln!("rwe-js: initial document");
        }
        runtime.eval(&init)?;
        for script in &page.scripts {
            if trace {
                eprintln!("rwe-js: evaluating {}", script.name);
            }
            runtime
                .eval(&script.source)
                .map_err(|e| format!("{}: {e}", script.name))?;
        }
        if trace {
            eprintln!("rwe-js: initial jobs");
        }
        runtime.pump()?;
        Ok(runtime)
    }
    fn eval(&mut self, source: &str) -> Result<boa_engine::JsValue, String> {
        if self.poisoned {
            return Err("JavaScript runtime failed; reload the page".into());
        }
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.context.eval(Source::from_bytes(source))
        })) {
            Ok(result) => result.map_err(|e| format!("JavaScript: {e}")),
            Err(_) => {
                self.poisoned = true;
                Err("JavaScript engine failure; reload or use a precompiled render function".into())
            }
        }
    }
    /// Runs bounded promise jobs and currently due timer callbacks.
    pub fn pump(&mut self) -> Result<(), String> {
        for step in 0..32 {
            if std::env::var_os("RWE_JS_TRACE").is_some() {
                eprintln!(
                    "rwe-js: pump {step}, jobs {}",
                    self.jobs.jobs.borrow().len()
                );
            }
            if self.poisoned {
                return Err("JavaScript runtime failed; reload the page".into());
            }
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.context.run_jobs()))
            {
                Ok(result) => result.map_err(|e| format!("JavaScript jobs: {e}"))?,
                Err(_) => {
                    self.poisoned = true;
                    return Err("JavaScript engine job failure; reload the page".into());
                }
            }
            if let Some(error) = self.jobs.error.borrow_mut().take() {
                return Err(error);
            }
            let count = self
                .eval("__rwe_flush_timers()")?
                .as_number()
                .unwrap_or(0.0);
            if count == 0.0 {
                let error = self.eval("__rwe_error()")?;
                let message = error
                    .as_string()
                    .map(|s| s.to_std_string_lossy())
                    .unwrap_or_default();
                if !message.is_empty() {
                    return Err(format!("JavaScript console: {message}"));
                }
                return Ok(());
            }
        }
        Err("JavaScript timer budget exceeded".into())
    }
    /// Whether the window should wake to process timers.
    pub fn has_timers(&mut self) -> Result<bool, String> {
        Ok(self
            .eval("__rwe_has_timers()")?
            .as_boolean()
            .unwrap_or(false))
    }
    /// Serializes the live DOM into RWE's bounded rendering subset.
    pub fn snapshot(&mut self) -> Result<Document, String> {
        self.sync_visibility()?;
        self.pump()?;
        self.sync_visibility()?;
        self.raw_snapshot()
    }
    fn sync_visibility(&mut self) -> Result<(), String> {
        let document = self.raw_snapshot()?;
        let mut hidden = Vec::new();
        for (id, node) in document.nodes().iter().enumerate() {
            if let NodeKind::Element { attributes, .. } = &node.kind {
                if self
                    .stylesheet
                    .computed_style(&document, id)
                    .map_err(|e| e.to_string())?
                    .is_some_and(|s| !s.visible)
                {
                    if let Some(id) = attributes
                        .get("data-rwe-node")
                        .and_then(|v| v.parse::<u64>().ok())
                    {
                        hidden.push(id.to_string());
                    }
                }
            }
        }
        self.eval(&format!("__rwe_set_hidden([{}])", hidden.join(",")))?;
        Ok(())
    }
    fn raw_snapshot(&mut self) -> Result<Document, String> {
        if std::env::var_os("RWE_JS_TRACE").is_some() {
            eprintln!("rwe-js: snapshot");
        }
        let value = self.eval("__rwe_snapshot()")?;
        let html = value
            .as_string()
            .ok_or("invalid DOM snapshot")?
            .to_std_string_lossy();
        Document::parse_page(&html).map_err(|e| format!("live DOM: {e}"))
    }
    /// Sends keyboard input to the active element and runs default button actions.
    /// Returns true when default handling was prevented or the host key was consumed.
    pub fn keyboard(&mut self, event: &crate::KeyEvent) -> Result<bool, String> {
        event.validate()?;
        self.sync_visibility()?;
        let consumed = self.eval(&format!(
            "__rwe_key({{key:{},text:{},pressed:{},repeat:{},shiftKey:{},ctrlKey:{},altKey:{},metaKey:{}}})",
            quote(&event.key), event.text.as_deref().map(quote).unwrap_or_else(|| "null".into()), event.pressed, event.repeat, event.shift_key,
            event.ctrl_key, event.alt_key, event.meta_key,
        ))?.as_boolean().unwrap_or(false);
        self.pump()?;
        Ok(consumed)
    }
    /// Inserts committed single-line text into the focused editable text input.
    /// Dispatches beforeinput/input, preserving framework-controlled value tracking.
    pub fn insert_text(&mut self, text: &str) -> Result<bool, String> {
        crate::keyboard::validate_text(text)?;
        self.sync_visibility()?;
        let consumed = self
            .eval(&format!("__rwe_insert_text({})", quote(text)))?
            .as_boolean()
            .unwrap_or(false);
        self.pump()?;
        Ok(consumed)
    }
    /// Applies an input-method transition to the current editable input.
    pub fn ime(&mut self, event: &crate::ImeEvent) -> Result<bool, String> {
        event.validate()?;
        self.sync_visibility()?;
        let source = match event {
            crate::ImeEvent::Preedit { text, cursor } => {
                let cursor = cursor
                    .map(|(start, end)| {
                        format!(
                            "[{},{}]",
                            text[..start].encode_utf16().count(),
                            text[..end].encode_utf16().count()
                        )
                    })
                    .unwrap_or_else(|| "null".into());
                format!(
                    "__rwe_ime({{kind:'preedit',text:{},cursor:{cursor}}})",
                    quote(text)
                )
            }
            crate::ImeEvent::Commit(text) => {
                format!("__rwe_ime({{kind:'commit',text:{}}})", quote(text))
            }
            crate::ImeEvent::Cancel => "__rwe_ime({kind:'cancel'})".into(),
        };
        let consumed = self.eval(&source)?.as_boolean().unwrap_or(false);
        self.pump()?;
        Ok(consumed)
    }
    /// Whether a transient input-method session is active.
    pub fn is_composing(&mut self) -> Result<bool, String> {
        Ok(self
            .eval("__rwe_composing()")?
            .as_boolean()
            .unwrap_or(false))
    }
    /// Clears focus and any pending Space activation after native focus loss.
    pub fn blur(&mut self) -> Result<(), String> {
        self.eval("__rwe_blur()")?;
        self.pump()
    }
    /// Dispatches a bubbling click by HTML id, flushing framework updates.
    pub fn click(&mut self, id: &str) -> Result<(), String> {
        if std::env::var_os("RWE_JS_TRACE").is_some() {
            eprintln!("rwe-js: click {id}");
        }
        self.sync_visibility()?;
        self.eval(&format!("__rwe_click_id({})", quote(id)))?;
        self.pump()
    }
    /// Dispatches a click to a live `data-rwe-node` identifier from a snapshot.
    pub fn click_node(&mut self, id: u64) -> Result<(), String> {
        self.sync_visibility()?;
        self.eval(&format!("__rwe_click({id})"))?;
        self.pump()
    }
}
