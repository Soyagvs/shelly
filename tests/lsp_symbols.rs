use shelly::lsp::symbols::{Scope, SymbolIndex, SymbolKind};
use shelly::lsp::syntax::parse;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn index(source: &str) -> SymbolIndex {
    SymbolIndex::from_parse(parse(source))
}

fn temp_fixture(files: &[(&str, &str)]) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("shelly-lsp-symbols-{stamp}"));
    fs::create_dir_all(&root).unwrap();
    for (name, source) in files {
        fs::write(root.join(name), source).unwrap();
    }
    root
}

#[test]
fn resolves_function_parameters_locals_and_globals_by_scope() {
    let symbols =
        index("let global = 1\nfn add(amount) { let local = amount\nlocal + global + amount }\n");

    assert_eq!(symbols.resolve("global", 0), Some(Scope::Global));
    assert_eq!(symbols.resolve("amount", 59), Some(Scope::Parameter));
    assert_eq!(symbols.resolve("local", 59), Some(Scope::Local));
    assert_eq!(symbols.resolve("missing", 59), None);
}

#[test]
fn declaration_and_reference_lookup_distinguish_unresolved_names() {
    let source = "let total = price\nfn show(value) { total + value + absent }\n";
    let symbols = index(source);

    assert_eq!(
        symbols.declaration("total").unwrap().kind,
        SymbolKind::Variable
    );
    assert_eq!(
        symbols.declaration("show").unwrap().kind,
        SymbolKind::Function
    );
    assert_eq!(symbols.references("total").len(), 1);
    assert_eq!(symbols.references("value").len(), 1);
    assert!(symbols.references("absent").is_empty());
    assert_eq!(symbols.unresolved_references(), vec!["price", "absent"]);
}

#[test]
fn same_file_shadowing_prefers_inner_local_declaration() {
    let symbols = index("let value = 1\nfn outer(value) { let result = value\nresult }\nvalue\n");

    assert_eq!(symbols.resolve_at("value", 39), Some(Scope::Parameter));
    assert_eq!(symbols.resolve_at("value", 64), Some(Scope::Global));
}

#[test]
fn imports_expose_only_exported_functions_and_constants() {
    let root = temp_fixture(&[
        (
            "lib.shl",
            "export fn public_fn(value) { value }\nexport const PUBLIC = 1\nfn private_fn() {}\nconst PRIVATE = 2\n",
        ),
        (
            "main.shl",
            "use \"lib.shl\"\npublic_fn(PUBLIC)\nprivate_fn(PRIVATE)\n",
        ),
    ]);
    let symbols = SymbolIndex::from_files(&root).unwrap();

    assert!(symbols.imported("main.shl", "public_fn"));
    assert!(symbols.imported("main.shl", "PUBLIC"));
    assert!(!symbols.imported("main.shl", "private_fn"));
    assert!(!symbols.imported("main.shl", "PRIVATE"));
}

#[test]
fn include_visibility_remains_distinct_from_import_exports() {
    let root = temp_fixture(&[
        ("parts.shl", "fn helper() {}\nconst VALUE = 1\n"),
        ("main.shl", "include \"parts.shl\"\nhelper(VALUE)\n"),
    ]);
    let symbols = SymbolIndex::from_files(&root).unwrap();

    assert!(symbols.included("main.shl", "helper"));
    assert!(symbols.included("main.shl", "VALUE"));
    assert!(!symbols.imported("main.shl", "helper"));
    assert!(!symbols.imported("main.shl", "VALUE"));
}
