use std::ops::Range;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Variable,
    Constant,
    Function,
    Parameter,
    Reference,
    Import,
    Include,
    Call,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleKind {
    Import,
    Include,
}
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub span: Range<usize>,
    pub scope: crate::lsp::symbols::Scope,
    pub declaration: bool,
    pub exported: bool,
}
#[derive(Debug, Clone, Default)]
pub struct ParseResult {
    pub symbols: Vec<Symbol>,
    pub modules: Vec<(ModuleKind, String)>,
}
#[derive(Debug, Clone)]
enum T {
    I(Range<usize>),
    S(String, Range<usize>),
    R,
    P(char),
    N,
}
fn lex(s: &str) -> Vec<T> {
    let b = s.as_bytes();
    let mut v = vec![];
    let mut i = 0;
    let mut in_matches = false;
    while i < b.len() {
        let c = b[i] as char;
        if c == '\n' {
            v.push(T::N);
            in_matches = false;
            i += 1
        } else if c.is_ascii_whitespace() {
            i += 1
        } else if c == '#' {
            while i < b.len() && b[i] != b'\n' {
                i += 1
            }
        } else if c == '"' {
            i += 1;
            let st = i;
            while i < b.len() && b[i] != b'"' {
                i += 1
            }
            v.push(T::S(s[st..i].to_string(), st..i));
            i += 1
        } else if c == '/' && in_matches {
            let st = i;
            i += 1;
            let mut escaped = false;
            let mut class = false;
            let mut complete = false;
            while i < b.len() {
                let ch = b[i] as char;
                if escaped {
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == '[' {
                    class = true;
                } else if ch == ']' {
                    class = false;
                } else if ch == '/' && !class {
                    complete = true;
                    i += 1;
                    break;
                } else if ch == '\n' {
                    break;
                }
                i += 1;
            }
            if complete {
                v.push(T::R);
            } else {
                i = st;
                v.push(T::P('/'));
                i += 1;
                in_matches = false;
            }
        } else if c.is_alphabetic() || c == '_' || b[i] >= 128 {
            let st = i;
            i += c.len_utf8();
            while i < b.len() {
                let ch = s[i..].chars().next().unwrap();
                if ch.is_alphanumeric() || ch == '_' {
                    i += ch.len_utf8()
                } else {
                    break;
                }
            }
            if &s[st..i] == "matches" {
                in_matches = true;
            }
            v.push(T::I(st..i))
        } else {
            v.push(T::P(c));
            i += 1
        }
    }
    v
}
pub fn parse(s: &str) -> ParseResult {
    let t = lex(s);
    let mut r = ParseResult::default();
    let mut i = 0;
    let mut depth = 0;
    let mut fn_depth = None;
    let mut params = std::collections::HashSet::new();
    while i < t.len() {
        if matches!(t[i], T::P('{')) {
            depth += 1;
            fn_depth.get_or_insert(depth);
        }
        if matches!(t[i], T::P('}')) {
            if fn_depth == Some(depth) {
                fn_depth = None;
                params.clear()
            }
            depth -= 1;
        }
        let word = match &t[i] {
            T::I(x) => Some(&s[x.clone()]),
            _ => None,
        };
        let scope = if fn_depth.is_none() {
            crate::lsp::symbols::Scope::Global
        } else if params.contains(word.unwrap_or(&"")) {
            crate::lsp::symbols::Scope::Parameter
        } else if depth == fn_depth.unwrap_or(0) {
            crate::lsp::symbols::Scope::Local
        } else {
            crate::lsp::symbols::Scope::Function
        };
        if matches!(word, Some("use") | Some("include")) {
            if let Some(T::S(x, span)) = t.get(i + 1) {
                let kind = if word == Some("use") {
                    ModuleKind::Import
                } else {
                    ModuleKind::Include
                };
                r.modules.push((kind, x.clone()));
                r.symbols.push(Symbol {
                    name: x.clone(),
                    kind: if kind == ModuleKind::Import {
                        SymbolKind::Import
                    } else {
                        SymbolKind::Include
                    },
                    span: span.clone(),
                    scope: crate::lsp::symbols::Scope::Global,
                    declaration: false,
                    exported: false,
                });
            }
            i += 1
        } else if matches!(word, Some("let") | Some("const")) {
            if let Some(T::I(x)) = t.get(i + 1) {
                r.symbols.push(Symbol {
                    name: s[x.clone()].into(),
                    kind: if word == Some("let") {
                        SymbolKind::Variable
                    } else {
                        SymbolKind::Constant
                    },
                    span: x.clone(),
                    scope,
                    declaration: true,
                    exported: false,
                });
            }
            i += 1
        } else if word == Some("export") {
            if let Some(T::I(x)) = t.get(i + 1) {
                if matches!(t.get(i + 2), Some(T::I(_))) {
                    let k = if &s[x.clone()] == "fn" {
                        SymbolKind::Function
                    } else {
                        SymbolKind::Constant
                    };
                    if let Some(T::I(y)) = t.get(i + 2) {
                        r.symbols.push(Symbol {
                            name: s[y.clone()].into(),
                            kind: k,
                            span: y.clone(),
                            scope: crate::lsp::symbols::Scope::Global,
                            declaration: true,
                            exported: true,
                        });
                    }
                }
            }
            i += 1
        } else if word == Some("fn") {
            if let Some(T::I(x)) = t.get(i + 1) {
                r.symbols.push(Symbol {
                    name: s[x.clone()].into(),
                    kind: SymbolKind::Function,
                    span: x.clone(),
                    scope: crate::lsp::symbols::Scope::Global,
                    declaration: true,
                    exported: false,
                });
            }
            let mut j = i + 2;
            while j < t.len() && !matches!(t[j], T::P('{')) {
                if let T::I(x) = &t[j] {
                    params.insert(s[x.clone()].to_string());
                    r.symbols.push(Symbol {
                        name: s[x.clone()].into(),
                        kind: SymbolKind::Parameter,
                        span: x.clone(),
                        scope: crate::lsp::symbols::Scope::Parameter,
                        declaration: true,
                        exported: false,
                    });
                }
                j += 1
            }
        } else if let T::I(x) = &t[i] {
            if &s[x.clone()] != "export" {
                r.symbols.push(Symbol {
                    name: s[x.clone()].into(),
                    kind: if matches!(t.get(i + 1), Some(T::P('('))) {
                        SymbolKind::Call
                    } else {
                        SymbolKind::Reference
                    },
                    span: x.clone(),
                    scope,
                    declaration: false,
                    exported: false,
                });
            }
        }
        i += 1;
    }
    r
}
