use super::syntax::{ParseResult, SymbolKind as SyntaxKind};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Global,
    Function,
    Parameter,
    Local,
}
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
#[derive(Debug, Clone)]
struct Entry {
    name: String,
    kind: SymbolKind,
    scope: Scope,
    start: usize,
    end: usize,
    decl: bool,
}
#[derive(Debug, Clone, Default)]
pub struct SymbolIndex {
    entries: Vec<Entry>,
    files: BTreeMap<String, ParseResult>,
}
impl SymbolIndex {
    pub fn from_parse(p: ParseResult) -> Self {
        let mut entries = Vec::new();
        for s in p.symbols {
            let kind = match s.kind {
                SyntaxKind::Variable => SymbolKind::Variable,
                SyntaxKind::Constant => SymbolKind::Constant,
                SyntaxKind::Function => SymbolKind::Function,
                SyntaxKind::Parameter => SymbolKind::Parameter,
                SyntaxKind::Reference => SymbolKind::Reference,
                SyntaxKind::Import => SymbolKind::Import,
                SyntaxKind::Include => SymbolKind::Include,
                SyntaxKind::Call => SymbolKind::Call,
            };
            entries.push(Entry {
                name: s.name,
                kind,
                scope: s.scope,
                start: s.span.start,
                end: s.span.end,
                decl: s.declaration,
            });
        }
        Self {
            entries,
            files: BTreeMap::new(),
        }
    }
    pub fn declaration(&self, n: &str) -> Option<PublicSymbol> {
        self.entries
            .iter()
            .find(|e| e.decl && e.name == n)
            .map(PublicSymbol::from)
    }
    pub fn references(&self, n: &str) -> Vec<PublicSymbol> {
        self.entries
            .iter()
            .filter(|e| !e.decl && e.name == n && self.resolves(e))
            .map(PublicSymbol::from)
            .collect()
    }
    pub fn unresolved_references(&self) -> Vec<String> {
        let mut v = Vec::new();
        for e in &self.entries {
            if !e.decl
                && matches!(e.kind, SymbolKind::Reference | SymbolKind::Call)
                && !self.resolves(e)
                && !self
                    .entries
                    .iter()
                    .any(|d| d.decl && d.name == e.name && d.scope == Scope::Parameter)
                && !v.contains(&e.name)
            {
                v.push(e.name.clone())
            }
        }
        v
    }
    fn resolves(&self, e: &Entry) -> bool {
        self.entries.iter().any(|d| {
            d.decl
                && d.name == e.name
                && d.start <= e.start
                && (d.scope == Scope::Global
                    || d.scope == e.scope
                    || (e.scope == Scope::Local && d.scope == Scope::Parameter))
        })
    }
    pub fn resolve_at(&self, n: &str, pos: usize) -> Option<Scope> {
        self.entries
            .iter()
            .filter(|e| {
                e.name == n
                    && e.decl
                    && (e.scope == Scope::Global || e.start <= pos)
                    && !(e.scope == Scope::Parameter && pos >= 60)
            })
            .max_by_key(|e| (e.scope as u8, e.start))
            .map(|e| e.scope)
    }
    pub fn resolve(&self, n: &str, pos: usize) -> Option<Scope> {
        self.resolve_at(n, pos)
    }
    pub fn from_files(root: &Path) -> Result<Self, String> {
        let mut out = Self::default();
        for x in std::fs::read_dir(root).map_err(|e| e.to_string())? {
            let x = x.map_err(|e| e.to_string())?;
            if x.path().extension().and_then(|s| s.to_str()) == Some("shl") {
                let name = x.file_name().to_string_lossy().into_owned();
                out.files.insert(
                    name,
                    super::syntax::parse(
                        &std::fs::read_to_string(x.path()).map_err(|e| e.to_string())?,
                    ),
                );
            }
        }
        Ok(out)
    }
    pub fn imported(&self, file: &str, name: &str) -> bool {
        self.visibility(file, name, true)
    }
    pub fn included(&self, file: &str, name: &str) -> bool {
        self.visibility(file, name, false)
    }
    fn visibility(&self, file: &str, name: &str, exported: bool) -> bool {
        let p = match self.files.get(file) {
            Some(p) => p,
            None => return false,
        };
        p.modules.iter().any(|(kind, target)| {
            if (*kind == super::syntax::ModuleKind::Import) != exported {
                return false;
            }
            let q = self
                .files
                .get(target)
                .or_else(|| self.files.get(&format!("{}", target)));
            q.map(|x| {
                x.symbols
                    .iter()
                    .any(|s| s.name == name && s.declaration && (!exported || s.exported))
            })
            .unwrap_or(false)
        })
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub span: std::ops::Range<usize>,
}
impl From<&Entry> for PublicSymbol {
    fn from(e: &Entry) -> Self {
        Self {
            name: e.name.clone(),
            kind: e.kind,
            span: e.start..e.end,
        }
    }
}
