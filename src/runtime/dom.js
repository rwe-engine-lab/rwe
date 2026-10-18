// RWE's experimental DOM subset. No networking, storage, modules or browser engine.
(() => {
  let nextId = 1;
  const nodes = new Map();
  class Event {
    constructor(type, init = {}) { Object.assign(this, init); this.type = type; this.bubbles = init.bubbles !== false; this.cancelable = init.cancelable !== false; this.defaultPrevented = false; this.timeStamp = Date.now(); this.button = 0; }
    preventDefault() { if (this.cancelable) this.defaultPrevented = true; }
    stopPropagation() { this._stop = true; }
    stopImmediatePropagation() { this._immediate = this._stop = true; }
    initEvent(type, bubbles, cancelable) { this.type = type; this.bubbles = bubbles; this.cancelable = cancelable; }
  }
  class EventTarget {
    constructor() { this._listeners = Object.create(null); }
    addEventListener(type, fn, options) {
      const capture = typeof options === 'boolean' ? options : !!(options && options.capture);
      const list = this._listeners[type] || (this._listeners[type] = []);
      if (fn && !list.some(e => e.fn === fn && e.capture === capture)) list.push({fn, capture});
    }
    removeEventListener(type, fn, options) {
      const capture = typeof options === 'boolean' ? options : !!(options && options.capture);
      this._listeners[type] = (this._listeners[type] || []).filter(e => e.fn !== fn || e.capture !== capture);
    }
    dispatchEvent(event) {
      event.target = this;
      const path = []; for (let n = this; n; n = n.parentNode) path.push(n);
      const invoke = (node, capture) => {
        event.currentTarget = node;
        event.eventPhase = node === this ? 2 : capture ? 1 : 3;
        for (const entry of (node._listeners[event.type] || []).slice()) {
          if (entry.capture === capture) { typeof entry.fn === 'function' ? entry.fn.call(node, event) : entry.fn.handleEvent(event); }
          if (event._immediate) break;
        }
        if (!capture && typeof node['on' + event.type] === 'function' && !event._immediate) node['on' + event.type](event);
      };
      for (let i = path.length - 1; i >= 0 && !event._stop; i--) invoke(path[i], true);
      for (let i = 0; i < path.length && !event._stop; i++) { invoke(path[i], false); if (!event.bubbles) break; }
      event.currentTarget = null; event.eventPhase = 0;
      return !event.defaultPrevented;
    }
  }
  const escape = s => String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');

  const kebab = s => s.replace(/[A-Z]/g, c => '-' + c.toLowerCase());
  const parseCSS = globalThis.__rwe_parse_css;
  delete globalThis.__rwe_parse_css;
  function style() {
    const data = Object.create(null);
    return new Proxy({
      setProperty(k,v) { data[k] = String(v); },
      removeProperty(k) { const old = data[k] || ''; delete data[k]; return old; },
      getPropertyValue(k) { return data[k] || ''; },
    }, {
      get(target, key) { if (key === 'cssText') return Object.keys(data).map(k => k + ':' + data[k]).join(';'); return key in target ? target[key] : typeof key === 'string' ? data[kebab(key)] || '' : undefined; },
      set(target, key, value) {
        if (key === 'cssText') { const declarations = parseCSS(String(value)); for (const k of Object.keys(data)) delete data[k]; for (const [name,entry] of declarations) data[name] = entry; }
        else if (typeof key === 'string') { const k = kebab(key); if (value === '' || value == null) delete data[k]; else data[k] = String(value); }
        return true;
      }
    });
  }
  class Node extends EventTarget {
    constructor(type, name, data = '') {
      super(); if (nextId > 100000) throw new Error('DOM node allocation limit exceeded');
      this._id = nextId++; nodes.set(this._id, this);
      this.nodeType = type; this.nodeName = name; this.parentNode = null; this.childNodes = []; this.data = data;
    }
    get ownerDocument() { return this.nodeType === 9 ? null : document; }
    get parentElement() { return this.parentNode && this.parentNode.nodeType === 1 ? this.parentNode : null; }
    get firstChild() { return this.childNodes[0] || null; }
    get lastChild() { return this.childNodes[this.childNodes.length-1] || null; }
    get nextSibling() { return this.parentNode ? this.parentNode.childNodes[this.parentNode.childNodes.indexOf(this)+1] || null : null; }
    get previousSibling() { return this.parentNode ? this.parentNode.childNodes[this.parentNode.childNodes.indexOf(this)-1] || null : null; }
    get textContent() { return this.nodeType === 3 || this.nodeType === 8 ? this.data : this.childNodes.filter(n => n.nodeType !== 8).map(n => n.textContent).join(''); }
    set textContent(value) {
      if (this.nodeType === 3 || this.nodeType === 8) { this.data = String(value == null ? '' : value); return; }
      for (const node of this.childNodes) node.parentNode = null;
      this.childNodes = [];
      if (value != null && String(value) !== '') this.appendChild(document.createTextNode(String(value)));
    }
    get nodeValue() { return this.nodeType === 3 || this.nodeType === 8 ? this.data : null; }
    set nodeValue(value) { if (this.nodeType === 3 || this.nodeType === 8) this.data = String(value); }
    appendChild(node) { return this.insertBefore(node, null); }
    insertBefore(node, before) {
      if (node === before) return node;
      if (node === this || node.contains(this)) throw new Error('DOM hierarchy cycle');
      if (before && before.parentNode !== this) throw new Error('reference is not a child');
      if (node.nodeType === 11) { for (const child of node.childNodes.slice()) this.insertBefore(child,before); return node; }
      if (node.parentNode) node.parentNode.removeChild(node);
      const index = before ? this.childNodes.indexOf(before) : this.childNodes.length;
      this.childNodes.splice(index,0,node); node.parentNode = this;
      if (node.nodeType === 1 && node._checked) normalizeRadio(node);
      for (const child of descendants(node).reverse()) if (child.nodeType === 1 && child._checked) normalizeRadio(child);
      return node;
    }
    removeChild(node) { const i = this.childNodes.indexOf(node); if (i < 0) throw new Error('not a child'); this.childNodes.splice(i,1); node.parentNode = null; return node; }
    replaceChild(node, old) { this.insertBefore(node,old); this.removeChild(old); return old; }
    remove() { if (this.parentNode) this.parentNode.removeChild(this); }
    contains(node) { for (let n = node; n; n = n.parentNode) if (n === this) return true; return false; }
    hasChildNodes() { return !!this.childNodes.length; }
    get isConnected() { return document.contains(this); }
    cloneNode(deep) {
      const n = this.nodeType === 1 ? document.createElement(this.localName) : this.nodeType === 3 ? document.createTextNode(this.data) : document.createComment(this.data);
      if (this.nodeType === 1) { for (const k of Object.keys(this._attrs)) n.setAttribute(k,this._attrs[k]); n.style.cssText = this.style.cssText; }
      if (deep) for (const c of this.childNodes) n.appendChild(c.cloneNode(true)); return n;
    }
  }
  class Element extends Node {
    constructor(name, namespace = 'http://www.w3.org/1999/xhtml') { super(1,namespace === 'http://www.w3.org/1999/xhtml' ? name.toUpperCase() : name); this.tagName = this.nodeName; this.localName = namespace === 'http://www.w3.org/1999/xhtml' ? name.toLowerCase() : name; this.namespaceURI = namespace; this._attrs = Object.create(null); this.style = style(); this._inputValue = ''; this._dirtyValue = false; this._selectionStart = this._selectionEnd = 0; this._selectionAnchor = 0; this._checked = false; this._dirtyChecked = false; this.indeterminate = false; if (this.localName === 'template') this.content = new Node(11,'#document-fragment'); }
    get value() { return toggleInput(this) && !this._dirtyValue && !this.hasAttribute('value') ? 'on' : this._inputValue; }
    set value(value) {
      const text = String(value == null ? '' : value).replace(/[\r\n]/g, '');
      if (text.length > 65536) throw new Error('input value exceeds 65536 UTF-16 units');
      this._dirtyValue = true;
      if (text !== this._inputValue) { this._inputValue = text; this.setSelectionRange(text.length, text.length); }
    }
    get checked() { return this._checked; }
    set checked(value) { this._checked = !!value; this._dirtyChecked = true; if (this._checked) normalizeRadio(this); }
    get defaultChecked() { return this.hasAttribute('checked'); }
    set defaultChecked(value) { if (value) this.setAttribute('checked',''); else this.removeAttribute('checked'); }
    get name() { return this.getAttribute('name') || ''; }
    set name(value) { this.setAttribute('name',value); }
    get htmlFor() { return this.getAttribute('for') || ''; }
    set htmlFor(value) { this.setAttribute('for',value); }
    get form() { return formOwner(this); }
    get control() {
      if (this.localName !== 'label') return null;
      const candidate = this.hasAttribute('for') ? document.getElementById(this.htmlFor) : descendants(this).find(labelable);
      return candidate && labelable(candidate) ? candidate : null;
    }
    get elements() { return this.localName === 'form' ? descendants(document).filter(n=>formControl(n) && formOwner(n) === this) : undefined; }
    requestSubmit(submitter = null) {
      if (this.localName !== 'form') throw new TypeError('requestSubmit needs a form');
      if (submitter && (!submitControl(submitter) || formOwner(submitter) !== this)) throw new TypeError('invalid submitter');
      this.dispatchEvent(new Event('submit',{submitter}));
    }
    reset() {
      if (this.localName !== 'form') throw new TypeError('reset needs a form');
      if (!this.dispatchEvent(new Event('reset'))) return;
      for (const n of this.elements) {
        if (n.localName !== 'input') continue;
        n.value = n.defaultValue; n._dirtyValue = false;
        n._checked = n.defaultChecked; n._dirtyChecked = false; n.indeterminate = false; n._userEdited = false;
      }
      for (const n of this.elements) if (n.defaultChecked && toggleInput(n)) { n._checked = true; normalizeRadio(n); }
      cleanFocus();
    }
    get defaultValue() { return this.getAttribute('value') || ''; }
    set defaultValue(value) { this.setAttribute('value', value); }
    get type() { return this.getAttribute('type') || (this.localName === 'input' ? 'text' : this.localName === 'button' ? 'submit' : ''); }
    set type(value) { this.setAttribute('type', value); }
    get readOnly() { return this.hasAttribute('readonly'); }
    set readOnly(value) { if (value) this._attrs.readonly = ''; else delete this._attrs.readonly; }
    get selectionStart() { return this._selectionStart; }
    set selectionStart(value) { this.setSelectionRange(value, Math.max(Number(value), this._selectionEnd)); }
    get selectionEnd() { return this._selectionEnd; }
    set selectionEnd(value) { this.setSelectionRange(Math.min(Number(value), this._selectionStart), value); }
    setSelectionRange(start, end) {
      const clamp = v => Math.max(0, Math.min(this._inputValue.length, Number(v) || 0));
      this._selectionEnd = clamp(end); this._selectionStart = Math.min(clamp(start), this._selectionEnd);
      this._selectionAnchor = this._selectionStart;
    }
    select() { this.setSelectionRange(0, this._inputValue.length); }
    get disabled() { return this.hasAttribute('disabled'); }
    set disabled(value) { if (value) this._attrs.disabled = ''; else delete this._attrs.disabled; }
    get tabIndex() { const value = this.getAttribute('tabindex'); return value !== null ? (/^[+-]?\d+$/.test(value.trim()) ? Number(value) : -1) : (this.localName === 'button' || textInput(this) || toggleInput(this)) ? 0 : -1; }
    set tabIndex(value) { this.setAttribute('tabindex', String(value)); }
    get children() { return this.childNodes.filter(n => n.nodeType === 1); }
    get id() { return this.getAttribute('id') || ''; } set id(v) { this.setAttribute('id',v); }
    get className() { return this.getAttribute('class') || ''; } set className(v) { this.setAttribute('class',v); }
    setAttribute(k,v) { this._attrs[k] = String(v); if (k === 'checked' && !this._dirtyChecked) this._checked = true; if (['checked','name','type','form'].includes(k) && this._checked) normalizeRadio(this); if (k === 'value' && !this._dirtyValue) { const dirty = this._dirtyValue; this.value = v; this._dirtyValue = dirty; } if (k === 'style') this.style.cssText = String(v); if (k === 'disabled') this.disabled = true; }
    getAttribute(k) { return Object.prototype.hasOwnProperty.call(this._attrs,k) ? this._attrs[k] : null; }
    hasAttribute(k) { return this.getAttribute(k) !== null; }
    removeAttribute(k) { delete this._attrs[k]; if (k === 'checked' && !this._dirtyChecked) this._checked = false; if (k === 'style') this.style.cssText = ''; if (k === 'disabled') this.disabled = false; if (k === 'value' && !this._dirtyValue) { this.value = ''; this._dirtyValue = false; } }
    setAttributeNS(ns,k,v) { this.setAttribute(k,v); } removeAttributeNS(ns,k) { this.removeAttribute(k); }
    get attributes() { return Object.keys(this._attrs).map(name => ({name,value:this._attrs[name]})); }
    matches(s) { if (s[0] === '#') return this.id === s.slice(1); if (s[0] === '.') return this.className.split(/\s+/).includes(s.slice(1)); return s === '*' || this.localName === s.toLowerCase(); }
    querySelectorAll(s) { return descendants(this).filter(n => n.nodeType === 1 && n.matches(s)); }
    querySelector(s) { return this.querySelectorAll(s)[0] || null; }
    get innerHTML() { return (this.content || this).childNodes.map(n => serialize(n,false)).join(''); }
    set innerHTML(s) { const fragment = parseFragment(String(s),this); const root = this.content || this; root.textContent = ''; root.appendChild(fragment); }
    get outerHTML() { return serialize(this,false); }
    focus() { if (eligible(this)) setFocus(this); }
    blur() { if (document.activeElement === this) setFocus(null); }
    click() { activate(this); }
  }
  function descendants(root) { const result = [], stack = root.childNodes.slice().reverse(); while (stack.length) { const n = stack.pop(); result.push(n); for (let i=n.childNodes.length-1;i>=0;i--) stack.push(n.childNodes[i]); } return result; }
  const voids = new Set(['area','base','br','col','embed','hr','img','input','link','meta','param','source','track','wbr']);
  const hidden = new Set(['head','script','style','title','meta','link','template']);
  function serialize(n, snapshot, depth = 0) {
    if (depth > 256) throw new Error('DOM depth limit exceeded');
    if (n.nodeType === 3) return escape(n.data);
    if (n.nodeType === 8) return snapshot ? '' : '<!--' + n.data.replace(/-->/g,'') + '-->';
    if (n.nodeType !== 1) return n.childNodes.map(c => serialize(c,snapshot,depth+1)).join('');
    if (snapshot && hidden.has(n.localName)) return '';
    let attrs = '';
    for (const k of Object.keys(n._attrs)) if (k !== 'style' && k !== 'data-rwe-node' && k !== 'data-rwe-focus' && k !== 'data-rwe-focus-visible' && k !== 'data-rwe-selection-start' && k !== 'data-rwe-selection-end' && k !== 'data-rwe-caret' && k !== 'data-rwe-indeterminate' && !(snapshot && toggleInput(n) && k === 'checked') && !(snapshot && textInput(n) && k === 'value') && !['data-rwe-display-value','data-rwe-preedit-start','data-rwe-preedit-end','data-rwe-display-start','data-rwe-display-end','data-rwe-display-caret','data-rwe-hide-caret'].includes(k) && !k.startsWith('on')) attrs += ' ' + k + '="' + escape(n._attrs[k]) + '"';
    if (n.style.cssText) attrs += ' style="' + escape(n.style.cssText) + '"';
    if (snapshot) {
      if (textInput(n)) attrs += ' value="' + escape(n.value) + '" data-rwe-selection-start="' + n.selectionStart + '" data-rwe-selection-end="' + n.selectionEnd + '" data-rwe-caret="' + (n._selectionAnchor === n.selectionStart ? n.selectionEnd : n.selectionStart) + '"';
      if (toggleInput(n)) { if (n.checked) attrs += ' checked=""'; if (n.indeterminate) attrs += ' data-rwe-indeterminate="true"'; }
      attrs += compositionAttributes(n);
      attrs += ' data-rwe-node="' + n._id + '"';
      if (document.activeElement === n) {
        attrs += ' data-rwe-focus="true"';
        if (keyboardFocus) attrs += ' data-rwe-focus-visible="true"';
      }
    }
    return '<' + n.localName + attrs + '>' + (voids.has(n.localName) ? '' : (n.content || n).childNodes.map(c => serialize(c,snapshot,depth+1)).join('') + '</' + n.localName + '>');
  }
  // Capture the shared native parser; applications cannot replace it accidentally.
  const parseHTML = globalThis.__rwe_parse_html;
  delete globalThis.__rwe_parse_html;
  function parseFragment(html, context) {
    const rows = parseHTML(html, context.localName, context.namespaceURI);
    const nodes = [document.createDocumentFragment()];
    for (let i=1;i<rows.length;i++) {
      const row = rows[i];
      const node = row.type === 1 ? document.createElementNS(row.namespace,row.name)
        : row.type === 8 ? document.createComment(row.text) : document.createTextNode(row.text);
      if (row.type === 1) for (const [name,value] of Object.entries(row.attributes)) node.setAttribute(name,value);
      nodes[i] = node;
      const parent = nodes[row.parent];
      (parent.content || parent).appendChild(node);
    }
    return nodes[0];
  }
  class Document extends Node {
    constructor() { super(9,'#document'); this.readyState = 'complete'; this.defaultView = globalThis; this.activeElement = null; this.oninput = null; }
    createElement(name) { return new Element(name); }
    createElementNS(ns,name) { return new Element(name,ns); }
    createTextNode(text) { return new Node(3,'#text',String(text)); }
    createComment(text) { return new Node(8,'#comment',String(text)); }
    createDocumentFragment() { return new Node(11,'#document-fragment'); }
    createEvent() { return new Event(''); }
    get documentElement() { return this.childNodes.find(n => n.nodeType === 1 && n.localName === 'html') || null; }
    get body() { return descendants(this).find(n => n.nodeType === 1 && n.localName === 'body') || null; }
    get head() { return descendants(this).find(n => n.nodeType === 1 && n.localName === 'head') || null; }
    getElementById(id) { return descendants(this).find(n => n.nodeType === 1 && n.id === id) || null; }
    querySelector(s) { return this.querySelectorAll(s)[0] || null; }
    querySelectorAll(s) { return descendants(this).filter(n => n.nodeType === 1 && n.matches(s)); }
  }
  Object.assign(globalThis, {Node, Element, HTMLElement:Element, SVGElement:Element, HTMLIFrameElement:class extends Element {}, Event, MouseEvent:Event, KeyboardEvent:Event, FocusEvent:Event, InputEvent:Event, CompositionEvent:Event, HTMLInputElement:Element, EventTarget, Document});
  globalThis.window = globalThis; globalThis.self = globalThis;
  globalThis.document = new Document();
  globalThis.navigator = {userAgent:'RWE/0.1'};
  globalThis.location = {href:'file:///rwe/', protocol:'file:'};
  const start = Date.now(); globalThis.performance = {now:() => Date.now()-start};
  const globals = new EventTarget(); globalThis.addEventListener = globals.addEventListener.bind(globals); globalThis.removeEventListener = globals.removeEventListener.bind(globals);
  const errors = [];
  globalThis.console = {log(){},warn(){},error(...args){ if (errors.length < 16) errors.push(args.map(String).join(' ')); },info(){},debug(){}};
  globalThis.__rwe_error = () => errors.shift() || '';
  let timerId = 1; const timers = new Map();
  globalThis.setTimeout = (fn, delay = 0, ...args) => { if (typeof fn !== 'function') throw new TypeError('timer callback must be a function'); if (timers.size >= 1024) throw new Error('timer queue exceeds 1024 callbacks'); const id = timerId++; timers.set(id, {fn:() => fn(...args), due:Date.now()+Math.max(0,Number(delay)||0)}); return id; };
  globalThis.clearTimeout = id => timers.delete(id);
  globalThis.requestAnimationFrame = fn => setTimeout(() => fn(performance.now()),16);
  globalThis.cancelAnimationFrame = clearTimeout;
  globalThis.queueMicrotask = fn => Promise.resolve().then(fn);
  globalThis.__rwe_flush_timers = () => {
    const batch = Array.from(timers.entries());
    let count = 0;
    for (const [id,task] of batch) { if (task.due <= Date.now() && timers.delete(id)) { task.fn(); count++; } }
    return count;
  };
  globalThis.__rwe_has_timers = () => timers.size > 0;
  let keyboardFocus = false, spaceTarget = null;
  let composition = null, rejectLateCommit = false;
  let cssHidden = new Set();
  function blocked(n) {
    for (let p = n; p && p.nodeType !== 9; p = p.parentNode) {
      if (p.nodeType === 1 && (p.disabled || cssHidden.has(p._id) || p.hasAttribute('hidden') || p.hasAttribute('inert') || p.style.display === 'none' || hidden.has(p.localName))) return true;
    }
    return false;
  }
  function textInput(n) { return n.localName === 'input' && ['text','search'].includes(n.type.toLowerCase()); }
  function toggleInput(n) { return n.localName === 'input' && ['checkbox','radio'].includes(n.type.toLowerCase()); }
  function labelable(n) { return n.nodeType === 1 && (n.localName === 'button' || n.localName === 'input' && n.type.toLowerCase() !== 'hidden'); }
  function formControl(n) { return n.nodeType === 1 && ['input','button'].includes(n.localName); }
  function formOwner(n) {
    if (n.hasAttribute('form')) { const form = document.getElementById(n.getAttribute('form')); return form && form.localName === 'form' ? form : null; }
    for (let p=n.parentNode;p;p=p.parentNode) if (p.localName === 'form') return p;
    return null;
  }
  function radioPeers(n) {
    if (n.localName !== 'input' || n.type.toLowerCase() !== 'radio' || !n.name || !n.isConnected) return [n];
    const owner = formOwner(n);
    return descendants(document).filter(p=>p.nodeType === 1 && p.localName === 'input' && p.type.toLowerCase() === 'radio' && p.name === n.name && formOwner(p) === owner);
  }
  function normalizeRadio(n) {
    if (n.localName !== 'input' || n.type.toLowerCase() !== 'radio') return;
    for (const other of radioPeers(n)) if (other !== n) other._checked = false;
  }
  function tabCandidate(n) {
    if (!eligible(n) || n.tabIndex < 0) return false;
    if (!toggleInput(n) || n.type.toLowerCase() !== 'radio' || !n.name) return true;
    const peers = radioPeers(n).filter(p=>eligible(p) && p.tabIndex >= 0);
    return (peers.find(p=>p.checked) || peers[0]) === n;
  }
  function disabledControl(n) {
    if (n.disabled) return true;
    for (let p=n.parentNode;p;p=p.parentNode) if (p.localName === 'fieldset' && p.disabled) return true;
    return false;
  }
  function submitControl(n) { return formControl(n) && ['submit','image'].includes(n.type.toLowerCase()); }
  function activate(n) {
    if (blocked(n)) return;
    const toggle = toggleInput(n), old = toggle ? radioPeers(n).map(p=>[p,p._checked,p._dirtyChecked]) : [];
    const wasChecked = n.checked, indeterminate = n.indeterminate;
    if (toggle) { n._checked = n.type.toLowerCase() === 'radio' ? true : !n.checked; n._dirtyChecked = true; if (n._checked) normalizeRadio(n); n.indeterminate = false; }
    if (!n.dispatchEvent(new Event('click'))) {
      for (const [p,checked,dirty] of old) { p._checked = checked; p._dirtyChecked = dirty; }
      if (toggle) n.indeterminate = indeterminate;
      return;
    }
    if (toggle && n.checked !== wasChecked && n.isConnected && !blocked(n)) {
      n.dispatchEvent(new Event('input',{cancelable:false}));
      n.dispatchEvent(new Event('change',{cancelable:false}));
    }
    if (formControl(n) && !blocked(n)) {
      const form = formOwner(n);
      if (form && submitControl(n)) form.requestSubmit(n);
      else if (form && n.type.toLowerCase() === 'reset') form.reset();
    }
    let label = n;
    while (label && label.localName !== 'label') label = label.parentNode;
    const control = label && label.control;
    for (let p=n;p && p!==label;p=p.parentNode) if (labelable(p)) return;
    if (control && control !== n && !control.contains(n) && !blocked(control)) { control.focus(); control.click(); }
  }
  class FormData {
    constructor(form, submitter = null) {
      this._entries = [];
      if (form === undefined) return;
      if (!form || form.localName !== 'form') throw new TypeError('FormData needs a form');
      if (submitter && (!submitControl(submitter) || formOwner(submitter) !== form)) throw new TypeError('invalid submitter');
      for (const n of form.elements) {
        if (!n.name || disabledControl(n) || n.localName === 'button' && n !== submitter) continue;
        if (toggleInput(n) && !n.checked) continue;
        if (['submit','reset','button','image','file'].includes(n.type.toLowerCase()) && n !== submitter) continue;
        if (n.type.toLowerCase() === 'file' || n.type.toLowerCase() === 'image') throw new TypeError('file/image controls are unsupported');
        this.append(n.name,n.value);
      }
    }
    append(name,value) { this._entries.push([String(name),String(value)]); }
    get(name) { const entry=this._entries.find(e=>e[0]===String(name)); return entry ? entry[1] : null; }
    getAll(name) { return this._entries.filter(e=>e[0]===String(name)).map(e=>e[1]); }
    has(name) { return this._entries.some(e=>e[0]===String(name)); }
    delete(name) { this._entries=this._entries.filter(e=>e[0]!==String(name)); }
    set(name,value) { name=String(name);const index=this._entries.findIndex(e=>e[0]===name);this.delete(name);this._entries.splice(index<0?this._entries.length:index,0,[name,String(value)]); }
    *entries() { for (const [key,value] of this._entries) yield [key,value]; }
    *keys() { for (const [key] of this._entries) yield key; }
    *values() { for (const [,value] of this._entries) yield value; }
    [Symbol.iterator]() { return this.entries(); }
    forEach(fn,thisArg) { for (const [key,value] of this._entries) fn.call(thisArg,value,key,this); }
  }
  globalThis.FormData = FormData;
  function eligible(n) { return !!n && n.nodeType === 1 && n.isConnected && !blocked(n) && (n.localName === 'button' || textInput(n) || toggleInput(n) || n.hasAttribute('tabindex')); }
  function setFocus(n) {
    const old = document.activeElement;
    if (old === n) return;
    spaceTarget = null;
    cancelComposition();
    document.activeElement = n;
    if (old && textInput(old) && old._userEdited && old._focusValue !== old.value) {
      old._focusValue = old.value; old._userEdited = false;
      old.dispatchEvent(new Event('change', {cancelable:false}));
    }
    if (n && textInput(n)) { n._focusValue = n.value; n._userEdited = false; }
    if (old && old.isConnected) {
      old.dispatchEvent(new Event('blur', {bubbles:false, relatedTarget:n}));
      old.dispatchEvent(new Event('focusout', {relatedTarget:n}));
    }
    if (n && document.activeElement === n && eligible(n)) {
      n.dispatchEvent(new Event('focus', {bubbles:false, relatedTarget:old}));
      if (document.activeElement === n && eligible(n)) n.dispatchEvent(new Event('focusin', {relatedTarget:old}));
    }
  }
  function cleanFocus() {
    if (composition && (composition.target !== document.activeElement || !eligible(composition.target) || composition.target.readOnly || composition.target.value !== composition.value || composition.target.selectionStart !== composition.start || composition.target.selectionEnd !== composition.end)) cancelComposition();
    if (document.activeElement && !eligible(document.activeElement)) setFocus(null);
    if (spaceTarget && (!eligible(spaceTarget) || spaceTarget !== document.activeElement)) spaceTarget = null;
  }
  function previous(text, index) { if (!index) return 0; const last = text.charCodeAt(index-1); return index > 1 && last >= 0xdc00 && last <= 0xdfff && text.charCodeAt(index-2) >= 0xd800 && text.charCodeAt(index-2) <= 0xdbff ? index-2 : index-1; }
  function next(text, index) { if (index >= text.length) return text.length; return index + (text.codePointAt(index) > 0xffff ? 2 : 1); }
  function replaceInput(n, data, inputType, start = n.selectionStart, end = n.selectionEnd, session = null) {
    const finish = data => { if (session) { endComposition(session, data); session = null; } };
    if (n.readOnly || !eligible(n)) { finish(''); return false; }
    const before = new Event('beforeinput', {data, inputType, isComposing:false});
    if (!n.dispatchEvent(before)) { finish(''); return true; }
    if (n.readOnly || !eligible(n) || document.activeElement !== n) { finish(''); return true; }
    let insert = data || '';
    const maximum = n.getAttribute('maxlength');
    const limit = maximum !== null && /^\d+$/.test(maximum) ? Math.min(65536, Number(maximum)) : 65536;
    const room = Math.max(0, limit - (n.value.length - (end-start)));
    // Truncate at code-point boundaries, preserving surrogate pairs.
    let fit = ''; for (const c of insert) { if (fit.length+c.length > room) break; fit += c; }
    const value = n.value.slice(0,start) + fit + n.value.slice(end);
    if (value === n.value) { finish(fit); return true; }
    // Native editing bypasses the property setter, as React's value tracker expects.
    if (!n._userEdited) n._focusValue = n.value;
    n._inputValue = value; n._dirtyValue = true; n._userEdited = true;
    n.setSelectionRange(start + fit.length, start + fit.length);
    finish(fit);
    n.dispatchEvent(new Event('input', {isComposing:false, data: data === null ? null : fit, inputType, cancelable:false}));
    return true;
  }
  globalThis.__rwe_insert_text = text => {
    if (!text) return false;
    cleanFocus();
    if (composition) return false; const n = document.activeElement;
    if (!n || !textInput(n) || !eligible(n) || n.readOnly) return false;
    return replaceInput(n, String(text).replace(/[\r\n]/g,''), 'insertText');
  };
  function inputKey(n, init) {
    if (!init.pressed) return ['Backspace','Delete','ArrowLeft','ArrowRight','Home','End','Enter',' '].includes(init.key) || Array.from(init.key).length === 1;
    keyboardFocus = true;
    if ((init.ctrlKey || init.metaKey) && !init.altKey && init.key.toLowerCase() === 'a') { n.select(); return true; }
    if (init.ctrlKey || init.metaKey || init.altKey) return false;
    if (['ArrowLeft','ArrowRight','Home','End'].includes(init.key)) {
      const start = n.selectionStart, end = n.selectionEnd;
      let caret = n._selectionAnchor === start ? end : start;
      if (init.key === 'Home') caret = 0;
      else if (init.key === 'End') caret = n.value.length;
      else if (!init.shiftKey && start !== end) caret = init.key === 'ArrowLeft' ? start : end;
      else caret = init.key === 'ArrowLeft' ? previous(n.value, caret) : next(n.value, caret);
      const anchor = init.shiftKey ? n._selectionAnchor : caret;
      n.setSelectionRange(Math.min(anchor,caret),Math.max(anchor,caret)); n._selectionAnchor = anchor;
      return true;
    }
    if (init.key === 'Backspace' || init.key === 'Delete') {
      if (!n.readOnly) replaceInput(n, null, init.key === 'Backspace' ? 'deleteContentBackward' : 'deleteContentForward', n.selectionStart === n.selectionEnd && init.key === 'Backspace' ? previous(n.value,n.selectionStart) : n.selectionStart, n.selectionStart === n.selectionEnd && init.key === 'Delete' ? next(n.value,n.selectionEnd) : n.selectionEnd);
      return true;
    }
    if (init.key === 'Enter') {
      if (n._userEdited && n._focusValue !== n.value) { n._focusValue = n.value; n._userEdited = false; n.dispatchEvent(new Event('change', {cancelable:false})); }
      if (!init.repeat) { const form=formOwner(n); if (form) { const submitter=form.elements.find(p=>submitControl(p) && !disabledControl(p)); submitter ? submitter.click() : form.requestSubmit(); } }
      return true;
    }
    const printable = init.text == null ? (Array.from(init.key).length === 1 ? init.key : '') : init.text;
    if (printable) { if (!n.readOnly) __rwe_insert_text(printable); return true; }
    // Never route letters typed in readonly fields to host R/W shortcuts.
    return Array.from(init.key).length === 1;
  }
  function endComposition(session, data) {
    session.target.dispatchEvent(new Event('compositionend', {data, cancelable:false}));
  }
  function cancelComposition() {
    if (!composition) return false;
    const session = composition; composition = null; rejectLateCommit = true;
    endComposition(session, ''); return true;
  }
  globalThis.__rwe_composing = () => !!composition;
  globalThis.__rwe_ime = event => {
    cleanFocus();
    if (event.kind === 'cancel') return cancelComposition();
    const n = document.activeElement;
    if (!n || !textInput(n) || !eligible(n) || n.readOnly) return false;
    if (event.kind === 'preedit') {
      if (!composition) {
        if (!event.text) return false;
        rejectLateCommit = false;
        composition = {target:n, value:n.value, start:n.selectionStart, end:n.selectionEnd, text:'', cursor:null};
        n.dispatchEvent(new Event('compositionstart', {data:'', cancelable:false}));
        cleanFocus(); if (!composition) return false;
      }
      composition.text = event.text; composition.cursor = event.cursor;
      keyboardFocus = true;
      n.dispatchEvent(new Event('compositionupdate', {data:event.text, cancelable:false}));
      cleanFocus(); return !!composition;
    }
    if (event.kind === 'commit') {
      if (rejectLateCommit) return false;
      const session = composition; composition = null;
      if (!event.text) { if (session) endComposition(session, ''); return !!session; }
      return replaceInput(n, event.text, 'insertText', session ? session.start : n.selectionStart, session ? session.end : n.selectionEnd, session);
    }
    return false;
  };
  function compositionAttributes(n) {
    if (!composition || composition.target !== n) return '';
    const c = composition, begin = c.start, end = begin + c.text.length;
    const display = n.value.slice(0,c.start) + c.text + n.value.slice(c.end);
    const start = c.cursor ? begin + c.cursor[0] : end, caret = c.cursor ? begin + c.cursor[1] : end;
    return ' data-rwe-display-value="'+escape(display)+'" data-rwe-preedit-start="'+begin+'" data-rwe-preedit-end="'+end+'" data-rwe-display-start="'+start+'" data-rwe-display-end="'+caret+'" data-rwe-display-caret="'+caret+'"' + (c.cursor ? '' : ' data-rwe-hide-caret="true"');
  }
  globalThis.__rwe_set_hidden = ids => { cssHidden = new Set(ids); cleanFocus(); };
  globalThis.__rwe_blur = () => { setFocus(null); spaceTarget = null; keyboardFocus = false; };
  globalThis.__rwe_key = init => {
    cleanFocus();
    const target = document.activeElement || document.body || document;
    const event = new Event(init.pressed ? 'keydown' : 'keyup', {...init, isComposing:!!composition});
    const allowed = target.dispatchEvent(event);
    cleanFocus();
    if (!allowed) { if (!init.pressed && init.key === ' ') spaceTarget = null; return true; }
    if (composition) { if (init.pressed && init.key === 'Escape') cancelComposition(); return true; }
    const input = document.activeElement;
    if (input && textInput(input) && eligible(input) && init.key !== 'Tab') return inputKey(input, init);
    if (init.ctrlKey || init.altKey || init.metaKey) return false;
    if (init.pressed && init.key === 'Tab') {
      keyboardFocus = true; spaceTarget = null;
      const candidates = descendants(document).filter(tabCandidate);
      // Positive tabindex comes first, then document order for tabindex=0.
      candidates.sort((a,b) => (a.tabIndex > 0 ? a.tabIndex : Infinity) - (b.tabIndex > 0 ? b.tabIndex : Infinity));
      const index = candidates.indexOf(document.activeElement);
      if (candidates.length) setFocus(candidates[index < 0 ? (init.shiftKey ? candidates.length-1 : 0) : (index + (init.shiftKey ? -1 : 1) + candidates.length) % candidates.length]);
      return true;
    }
    const active = document.activeElement;
    if (active && toggleInput(active) && active.type.toLowerCase() === 'radio' && eligible(active) && init.pressed && !init.ctrlKey && !init.altKey && !init.metaKey && ['ArrowLeft','ArrowRight','ArrowUp','ArrowDown'].includes(init.key)) {
      const peers=radioPeers(active).filter(eligible), index=peers.indexOf(active);
      const next=peers[(index+(['ArrowLeft','ArrowUp'].includes(init.key)?-1:1)+peers.length)%peers.length];
      if (next) { keyboardFocus=true; setFocus(next); next.click(); } return true;
    }
    if (active && (active.localName === 'button' || toggleInput(active)) && eligible(active)) {
      if (active.localName === 'button' && init.key === 'Enter' && init.pressed) { keyboardFocus = true; if (!init.repeat) active.click(); return true; }
      if (init.key === ' ') {
        keyboardFocus = true;
        if (init.pressed) { if (!init.repeat) spaceTarget = active; }
        else { const armed = spaceTarget; spaceTarget = null; if (armed === active) active.click(); }
        return true;
      }
    }
    if (!init.pressed && init.key === ' ') spaceTarget = null;
    return false;
  };
  function pointerClick(n) {
    if (!n || !n.isConnected) throw new Error('click target is detached');
    keyboardFocus = false; spaceTarget = null;
    cancelComposition();
    if (blocked(n)) return;
    let focus = n; while (focus && !eligible(focus)) focus = focus.parentNode;
    setFocus(focus);
    if (focus && textInput(focus)) focus.setSelectionRange(focus.value.length, focus.value.length);
    if (n.isConnected && !blocked(n)) n.click ? n.click() : n.dispatchEvent(new Event('click'));
  }
  globalThis.__rwe_snapshot = () => { cleanFocus(); return serialize(document,true); };
  globalThis.__rwe_click = id => { pointerClick(nodes.get(id)); };
  globalThis.__rwe_click_id = id => { const n = document.getElementById(id); if (!n) throw new Error('click target not found: '+id); pointerClick(n); };
})();
