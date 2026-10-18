use shelly::lsp::syntax::{SymbolKind, parse};

fn names(source: &str) -> Vec<(String, SymbolKind, std::ops::Range<usize>)> {
    parse(source)
        .symbols
        .into_iter()
        .map(|symbol| (symbol.name, symbol.kind, symbol.span))
        .collect()
}

#[test]
fn records_declarations_references_imports_and_calls_with_byte_spans() {
    let source = "let total = price\nconst tax = 2\nfn add(amount) { total + amount }\nuse \"math.shl\"\ninclude \"parts.shl\"\nadd(total)\n";
    let symbols = names(source);

    assert!(symbols.iter().any(|(name, kind, span)| name == "total"
        && *kind == SymbolKind::Variable
        && *span == (4..9)));
    assert!(symbols.iter().any(|(name, kind, span)| name == "tax"
        && *kind == SymbolKind::Constant
        && *span == (24..27)));
    assert!(symbols.iter().any(|(name, kind, span)| name == "add"
        && *kind == SymbolKind::Function
        && *span == (35..38)));
    assert!(symbols.iter().any(|(name, kind, span)| name == "amount"
        && *kind == SymbolKind::Parameter
        && *span == (39..45)));
    assert!(symbols.iter().any(|(name, kind, span)| name == "price"
        && *kind == SymbolKind::Reference
        && *span == (12..17)));
    assert!(symbols.iter().any(|(name, kind, span)| *name == "math.shl"
        && *kind == SymbolKind::Import
        && *span == (71..79)));
    assert!(symbols.iter().any(|(name, kind, span)| *name == "parts.shl"
        && *kind == SymbolKind::Include
        && *span == (90..99)));
    assert!(symbols.iter().any(|(name, kind, span)| *name == "add"
        && *kind == SymbolKind::Call
        && *span == (101..104)));
    assert!(symbols.iter().any(|(name, kind, span)| *name == "total"
        && *kind == SymbolKind::Reference
        && *span == (105..110)));
}

#[test]
fn preserves_symbols_after_malformed_code() {
    let source = "fn broken(a { let missing =\n\nlet later = 7\nfn after(value) { later }\n";
    let parsed = parse(source);

    assert!(parsed.symbols.iter().any(|s| s.name == "later"));
    assert!(parsed.symbols.iter().any(|s| s.name == "after"));
    assert!(parsed.symbols.iter().any(|s| s.name == "value"));
}

#[test]
fn ignores_comments_and_multiline_string_false_references() {
    let source = r#"# fake_call(fake_ref)
let text = """
not_a_reference(fake_call)
"""
let real = fake_ref
real()
"#;
    let parsed = parse(source);
    let occurrences = parsed
        .symbols
        .iter()
        .filter(|symbol| symbol.name == "fake_call" || symbol.name == "fake_ref")
        .count();

    assert_eq!(occurrences, 1);
    assert!(parsed.symbols.iter().any(|s| s.name == "real"));
    assert!(
        parsed
            .symbols
            .iter()
            .any(|s| s.name == "real" && s.kind == SymbolKind::Call)
    );
}

#[test]
fn regex_literals_after_matches_hide_interior_identifiers() {
    let source = "matches $value /foo_identifier [a-z]+\\/bar/\nlet visible = value\n";
    let symbols = names(source);

    assert!(!symbols.iter().any(|(name, _, _)| name == "foo_identifier"));
    assert!(!symbols.iter().any(|(name, _, _)| name == "bar"));
    assert!(symbols.iter().any(|(name, _, _)| name == "visible"));
}

#[test]
fn unterminated_regex_is_not_a_complete_literal() {
    let source = "matches $value /unterminated_identifier\nlet visible = value\n";
    let symbols = names(source);

    assert!(
        symbols
            .iter()
            .any(|(name, _, _)| name == "unterminated_identifier")
    );
    assert!(symbols.iter().any(|(name, _, _)| name == "visible"));
}

#[test]
fn spans_are_byte_offsets_for_non_ascii_source() {
    let source = "let café = цена\nцена\n";
    let parsed = parse(source);

    let declaration = parsed.symbols.iter().find(|s| s.name == "café").unwrap();
    assert_eq!(declaration.span, 4..9);
    let reference = parsed.symbols.iter().find(|s| s.name == "цена").unwrap();
    assert_eq!(reference.span, 12..20);
}
