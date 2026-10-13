use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn prints_placeholder_when_no_script_is_provided() {
    let output = Command::new(env!("CARGO_BIN_EXE_shelly"))
        .output()
        .expect("run shelly binary");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout is valid UTF-8");
    assert!(stdout.contains("Pass a .shelly script path"));
    assert!(stdout.contains("Usage: shelly <script.shelly>"));
}

#[test]
fn check_valid_script_succeeds_without_script_output() {
    let output = check_script(
        r#"
$name = "world"
shellout "hello $name"
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "");
}

#[test]
fn check_does_not_execute_command_shellout_ask_or_shellin() {
    let output = check_script(
        r#"
command "printf SHOULD_NOT_PRINT"
shellout "SHOULD_NOT_PRINT"
ask "Question? " -> $answer
shellin $input
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "");
}

#[test]
fn check_reports_missing_end() {
    let output = check_script("if true\n    shellout \"no end\"\n");

    assert!(!output.status.success());
    assert!(stderr(&output).contains("line 2"));
    assert!(stderr(&output).contains("missing end"));
}

#[test]
fn check_reports_break_outside_loop() {
    let output = check_script("break\n");

    assert!(!output.status.success());
    assert!(stderr(&output).contains("line 1"));
    assert!(stderr(&output).contains("break outside loop"));
}

#[test]
fn check_reports_return_outside_function() {
    let output = check_script("return \"nope\"\n");

    assert!(!output.status.success());
    assert!(stderr(&output).contains("line 1"));
    assert!(stderr(&output).contains("return outside function"));
}

#[test]
fn check_reports_call_with_no_name() {
    let output = check_script("call\n");

    assert!(!output.status.success());
    assert!(stderr(&output).contains("line 1"));
    assert!(stderr(&output).contains("call requires a function name"));
}

#[test]
fn check_json_format_emits_diagnostics_array() {
    let script = write_script("break\n");
    let output = Command::new(env!("CARGO_BIN_EXE_shelly"))
        .args(["check", "--format", "json"])
        .arg(script)
        .output()
        .expect("run shelly check");

    assert!(!output.status.success());
    assert_eq!(stderr(&output), "");
    let stdout = stdout(&output);
    assert!(stdout.starts_with('['));
    assert!(stdout.contains("\"file\""));
    assert!(stdout.contains("\"line\":1"));
    assert!(stdout.contains("\"column\":1"));
    assert!(stdout.contains("\"severity\":\"error\""));
    assert!(stdout.contains("break outside loop"));
}

#[test]
fn shellout_literal_and_interpolation() {
    let output = run_script(
        r#"
$name = "world"
shellout "hello $name"
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "hello world\n");
}

#[test]
fn variable_assignment_and_shellout_variable() {
    let output = run_script(
        r#"
$value = "assigned"
shellout $value
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "assigned\n");
}

#[test]
fn command_arrow_captures_stdout_without_printing_it_directly() {
    let output = run_script(
        r#"
command "printf hello" -> $out
shellout "captured=$out"
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "captured=hello\n");
}

#[test]
fn command_arrow_trims_trailing_newlines() {
    let output = run_script(
        r#"
command "printf 'hello\n\n'" -> $out
shellout "<$out>"
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "<hello>\n");
}

#[test]
fn command_arrow_capture_can_be_interpolated_with_shellout() {
    let output = run_script(
        r#"
command "printf blue" -> $color
shellout "favorite=$color"
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "favorite=blue\n");
}

#[test]
fn command_streams_system_command_output() {
    let output = run_script(r#"command "printf hello""#);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "hello");
}

#[test]
fn old_capture_syntax_is_rejected() {
    let output = run_script(r#"capture $out, "printf hello""#);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("line 1"));
    assert!(stderr(&output).contains("command \"...\" -> $var"));
}

#[test]
fn command_arrow_rejects_constant_destination() {
    let output = run_script(r#"command "pwd" -> $$CWD"#);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("line 1"));
    assert!(stderr(&output).contains("destination must be a variable"));
}

#[test]
fn command_arrow_rejects_malformed_destination() {
    for source in [
        r#"command "printf hello" -> "#,
        r#"command "printf hello" -> out"#,
        r#"command "printf hello" -> $9out"#,
        r#"command "printf hello" -> $out extra"#,
        r#"command "printf hello" - $out"#,
    ] {
        let output = run_script(source);

        assert!(!output.status.success(), "source succeeded: {source}");
        assert!(stderr(&output).contains("line 1"));
        assert!(
            stderr(&output).contains("command capture syntax")
                || stderr(&output).contains("destination must be a variable"),
            "stderr did not describe malformed destination: {}",
            stderr(&output)
        );
    }
}

#[test]
fn run_keyword_is_rejected() {
    let output = run_script(r#"run "printf old""#);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("line 1"));
    assert!(stderr(&output).contains("unknown command or invalid syntax"));
}

#[test]
fn ask_prints_prompt_reads_stdin_line_and_stores_variable() {
    let script = write_script(
        r#"
ask "Name? " -> $name
if $name == "Ada"
    shellout "hello $name"
else
    shellout "unknown"
end
"#,
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_shelly"))
        .arg(&script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn shelly");

    child
        .stdin
        .as_mut()
        .expect("stdin is piped")
        .write_all(b"Ada\r\n")
        .expect("write stdin");
    let output = child.wait_with_output().expect("wait for shelly");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "Name? hello Ada\n");
}

#[test]
fn ask_rejects_invalid_argument_forms() {
    for (source, expected) in [
        (
            "ask \"Question?\"\n",
            "ask syntax is ask \"Question?\" -> $answer",
        ),
        ("ask $answer -> $dest\n", "expected string literal"),
        (
            "ask \"Question?\", $answer\n",
            "use ask \"Question?\" -> $answer",
        ),
        (
            "ask \"Question?\" -> \"answer\"\n",
            "ask destination must be a variable",
        ),
        (
            "ask \"Question?\" -> $$ANSWER\n",
            "ask destination must be a variable",
        ),
        (
            "ask \"Question?\" -> $answer extra\n",
            "ask destination must be a variable",
        ),
    ] {
        let output = run_script(source);

        assert!(!output.status.success(), "source succeeded: {source}");
        assert!(stderr(&output).contains("line 1"));
        assert!(
            stderr(&output).contains(expected),
            "stderr did not contain {expected:?}: {}",
            stderr(&output)
        );
    }
}

#[test]
fn docs_and_examples_use_ask_arrow_syntax() {
    for path in [
        "README.md",
        "docs/language.md",
        "example-01.shelly",
        "examples/ask-if.shelly",
    ] {
        let content = fs::read_to_string(path).expect("read documented example");
        assert!(
            !content.contains("ask \"Continue? \", $answer")
                && !content.contains("ask \"Name? \", $name")
                && !content.contains("ask \"Question?\", $name"),
            "{path} still uses old comma ask syntax"
        );
    }
}

#[test]
fn shellin_reads_stdin_line() {
    let script = write_script(
        r#"
shellin $answer
shellout "got $answer"
"#,
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_shelly"))
        .arg(&script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn shelly");

    child
        .stdin
        .as_mut()
        .expect("stdin is piped")
        .write_all(b"input line\r\n")
        .expect("write stdin");
    let output = child.wait_with_output().expect("wait for shelly");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "got input line\n");
}

#[test]
fn explicit_line_comment_is_ignored() {
    let output = run_script(
        r#"
# this is a comment #
shellout "after"
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "after\n");
}

#[test]
fn explicit_block_comment_is_ignored() {
    let output = run_script(
        r#"
#
whole block
#
shellout "after block"
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "after block\n");
}

#[test]
fn old_style_comment_without_closing_marker_is_rejected_when_never_closed() {
    let output = run_script("# old comment\n");

    assert!(!output.status.success());
    assert!(stderr(&output).contains("line 1"));
    assert!(stderr(&output).contains("unterminated comment"));
}

#[test]
fn unterminated_block_comment_reports_start_line() {
    let output = run_script("shellout \"before\"\n#\nblock\n");

    assert!(!output.status.success());
    assert!(stderr(&output).contains("line 2"));
    assert!(stderr(&output).contains("unterminated comment"));
}

#[test]
fn comment_markers_inside_strings_are_literal_text() {
    let output = run_script(r##"shellout "# not a comment #""##);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "# not a comment #\n");
}

#[test]
fn unknown_command_returns_non_zero_with_line_number() {
    let output = run_script("bogus\n");

    assert!(!output.status.success());
    assert!(stderr(&output).contains("line 1"));
    assert!(stderr(&output).contains("unknown command"));
}

#[test]
fn invalid_syntax_returns_non_zero_with_line_number() {
    let output = run_script("$9bad = \"nope\"\n");

    assert!(!output.status.success());
    assert!(stderr(&output).contains("line 1"));
    assert!(stderr(&output).contains("variable name"));
}

#[test]
fn variables_are_case_insensitive_and_may_be_empty() {
    let output = run_script(
        r#"
$Name = "Ada"
shellout $name
$empty
shellout "empty:$EMPTY"
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "Ada\nempty:\n");
}

#[test]
fn identifiers_must_start_with_ascii_letter() {
    for source in [
        "$9bad = \"nope\"\n",
        "$_bad = \"nope\"\n",
        "$$9BAD = \"nope\"\n",
        "$$_BAD = \"nope\"\n",
        "$bad name = \"nope\"\n",
    ] {
        let output = run_script(source);

        assert!(!output.status.success(), "source succeeded: {source}");
        assert!(stderr(&output).contains("line 1"));
        assert!(stderr(&output).contains("identifier"));
    }
}

#[test]
fn values_support_numbers_booleans_and_constant_interpolation() {
    let output = run_script(
        r#"
$count = 42
$enabled = true
$$NAME = "Shelly"
$$LIMIT = 10
shellout "$$NAME count=$count enabled=$enabled limit=$$LIMIT"
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "Shelly count=42 enabled=true limit=10\n");
}

#[test]
fn constants_require_uppercase_initial_values_and_cannot_be_reassigned() {
    for source in [
        "$$name = \"nope\"\n",
        "$$NAME\n",
        "$$NAME =\n",
        "$$NAME = \"first\"\n$$NAME = \"second\"\n",
    ] {
        let output = run_script(source);

        assert!(!output.status.success(), "source succeeded: {source}");
        assert!(stderr(&output).contains("line"));
    }
}

#[test]
fn functions_call_with_parameters_defaults_local_scope_and_global_reads() {
    let output = run_script(
        r#"
$outer = "outside"
$$APP = "Shelly"
fn person $name, $age, $prefix = "Hola"
    global $outer, $$APP
    $outer = "inside"
    shellout "$prefix $name age=$age outer=$outer app=$$APP"
end
call person "Agustin", 27
shellout $outer
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "Hola Agustin age=27 outer=inside app=Shelly\noutside\n"
    );
}

#[test]
fn function_call_arguments_accept_variable_and_constant_references() {
    let output = run_script(
        r#"
$name = "Ada"
$$APP = "Shelly"
fn greet $name, $app, $age, $enabled
    shellout "$name from $app age=$age enabled=$enabled"
end
call greet $name, $$APP, 27, true
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "Ada from Shelly age=27 enabled=true\n");
}

#[test]
fn function_call_reference_arguments_fail_when_undefined() {
    for (source, expected) in [
        (
            "fn greet $name\nend\ncall greet $missing\n",
            "undefined variable $missing",
        ),
        (
            "fn greet $app\nend\ncall greet $$MISSING\n",
            "undefined constant $$MISSING",
        ),
    ] {
        let output = run_script(source);

        assert!(!output.status.success(), "source succeeded: {source}");
        assert!(
            stderr(&output).contains(expected),
            "stderr did not contain {expected:?}: {}",
            stderr(&output)
        );
    }
}

#[test]
fn function_errors_cover_invalid_definitions_and_calls() {
    for (source, expected) in [
        ("fn missing\nshellout \"no end\"\n", "missing end"),
        ("fn same\nend\nfn same\nend\n", "duplicate function"),
        ("does_not_exist \"x\"\n", "unknown command"),
        (
            "fn greet $name\nend\ngreet \"Ada\"\n",
            "function calls must use call",
        ),
        ("fn one $name\nend\ncall one\n", "wrong argument count"),
        (
            "fn bad $first = \"x\", $second\nend\n",
            "required parameter cannot follow default",
        ),
        (
            "fn bad\nglobal name\nend\ncall bad\n",
            "expected identifier",
        ),
        (
            "$outer = \"outside\"\nfn bad\nshellout $outer\nend\ncall bad\n",
            "undefined variable",
        ),
    ] {
        let output = run_script(source);

        assert!(!output.status.success(), "source succeeded: {source}");
        assert!(
            stderr(&output).contains(expected),
            "stderr did not contain {expected:?}: {}",
            stderr(&output)
        );
    }
}

#[test]
fn conditionals_cover_branches_comparisons_truthiness_and_scope() {
    let output = run_script(
        r#"
$enabled = true
$result = "start"
if $enabled
    $result = "if"
end
shellout $result
$count = 3
if $count > 5
    shellout "too high"
elif $count == 3
    shellout "three"
else
    shellout "fallback"
end
if ""
    shellout "empty true"
else
    shellout "empty false"
end
if 0
    shellout "zero true"
else
    shellout "zero false"
end
if false
    shellout "false true"
else
    shellout "false false"
end
if "hello $result" != ""
    shellout "interpolated true"
end
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "if\nthree\nempty false\nzero false\nfalse false\ninterpolated true\n"
    );
}

#[test]
fn conditional_errors_cover_structure_and_unsupported_expressions() {
    for (source, expected) in [
        ("if true\nshellout \"no end\"\n", "missing end"),
        ("if false\nelse\nelif true\nend\n", "elif after else"),
        ("if false\nelse\nelse\nend\n", "duplicate else"),
        ("if\nend\n", "condition required"),
        ("if (true)\nend\n", "unsupported complex condition"),
    ] {
        let output = run_script(source);

        assert!(!output.status.success(), "source succeeded: {source}");
        assert!(stderr(&output).contains("line"));
        assert!(
            stderr(&output).contains(expected),
            "stderr did not contain {expected:?}: {}",
            stderr(&output)
        );
    }
}

#[test]
fn import_loads_exported_constant() {
    let dir = test_dir();
    fs::write(dir.join("module.shelly"), "export $$GREETING = \"Hola\"\n").expect("write module");
    fs::write(
        dir.join("main.shelly"),
        "import \"module.shelly\"\nshellout $$GREETING\n",
    )
    .expect("write main");

    let output = run_script_path(dir.join("main.shelly"));

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "Hola\n");
}

#[test]
fn import_loads_exported_function() {
    let dir = test_dir();
    fs::write(
        dir.join("module.shelly"),
        "export fn greet $name\n    shellout \"Hola $name\"\nend\n",
    )
    .expect("write module");
    fs::write(
        dir.join("main.shelly"),
        "import \"module.shelly\"\ncall greet \"Ada\"\n",
    )
    .expect("write main");

    let output = run_script_path(dir.join("main.shelly"));

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "Hola Ada\n");
}

#[test]
fn import_keeps_private_symbols_unavailable() {
    for (use_private, expected) in [
        ("shellout $secret\n", "undefined variable $secret"),
        ("shellout $$SECRET\n", "undefined constant $$SECRET"),
        ("hidden\n", "unknown command"),
    ] {
        let dir = test_dir();
        fs::write(
            dir.join("module.shelly"),
            "$secret = \"private var\"\n$$SECRET = \"private const\"\nfn hidden\n    shellout \"private fn\"\nend\n",
        )
        .expect("write module");
        fs::write(
            dir.join("main.shelly"),
            format!("import \"module.shelly\"\n{use_private}"),
        )
        .expect("write main");

        let output = run_script_path(dir.join("main.shelly"));

        assert!(
            !output.status.success(),
            "private use succeeded: {use_private}"
        );
        assert!(
            stderr(&output).contains(expected),
            "stderr did not contain {expected:?}: {}",
            stderr(&output)
        );
    }
}

#[test]
fn import_does_not_run_non_exported_top_level_side_effects() {
    let dir = test_dir();
    fs::write(
        dir.join("module.shelly"),
        "shellout \"SHOULD NOT RUN\"\nexport $$VALUE = \"ok\"\n",
    )
    .expect("write module");
    fs::write(
        dir.join("main.shelly"),
        "import \"module.shelly\"\nshellout $$VALUE\n",
    )
    .expect("write main");

    let output = run_script_path(dir.join("main.shelly"));

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "ok\n");
}

#[test]
fn include_runs_full_file_in_current_scope() {
    let dir = test_dir();
    fs::write(
        dir.join("setup.shelly"),
        "$color = \"blue\"\nshellout \"setup\"\n",
    )
    .expect("write setup");
    fs::write(
        dir.join("main.shelly"),
        "include \"setup.shelly\"\nshellout $color\n",
    )
    .expect("write main");

    let output = run_script_path(dir.join("main.shelly"));

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "setup\nblue\n");
}

#[test]
fn import_include_errors_cover_export_variables_missing_files_cycles_and_duplicate_imports() {
    let output = run_script("export $name = \"x\"\n");
    assert!(!output.status.success());
    assert!(stderr(&output).contains("line 1"));
    assert!(stderr(&output).contains("only constants and functions can be exported"));

    let output = run_script("import \"missing.shelly\"\n");
    assert!(!output.status.success());
    assert!(stderr(&output).contains("line 1"));
    assert!(stderr(&output).contains("could not read file"));

    let dir = test_dir();
    fs::write(dir.join("a.shelly"), "import \"b.shelly\"\n").expect("write a");
    fs::write(dir.join("b.shelly"), "import \"a.shelly\"\n").expect("write b");
    let output = run_script_path(dir.join("a.shelly"));
    assert!(!output.status.success());
    assert!(stderr(&output).contains("cycle"));
    assert!(stderr(&output).contains("line 1"));

    let dir = test_dir();
    fs::write(dir.join("module.shelly"), "export $$VALUE = \"once\"\n").expect("write module");
    fs::write(
        dir.join("main.shelly"),
        "import \"module.shelly\"\nimport \"module.shelly\"\nshellout $$VALUE\n",
    )
    .expect("write main");
    let output = run_script_path(dir.join("main.shelly"));
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "once\n");
}

#[test]
fn numeric_expressions_support_precedence_parentheses_floats_negatives_and_references() {
    let output = run_script(
        r#"
$a = 1 + 2 * 3
$b = (1 + 2) * 3
$c = 10.5 + 2.25
$d = -3 * 2
$$BASE = 10
$total = $$BASE * $c
$clean = 6 / 2
shellout $a
shellout $b
shellout $c
shellout $d
shellout $total
shellout $clean
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "7\n9\n12.75\n-6\n127.5\n3\n");
}

#[test]
fn numeric_expression_division_by_zero_is_clear_error() {
    let output = run_script("$bad = 1 / 0\n");

    assert!(!output.status.success());
    assert!(stderr(&output).contains("line 1"));
    assert!(stderr(&output).contains("division by zero"));
}

#[test]
fn numeric_expression_rejects_non_numeric_operands_undefined_references_and_malformed_syntax() {
    for (source, expected) in [
        ("$name = \"Ada\"\n$bad = $name + 1\n", "numeric operand"),
        ("$enabled = true\n$bad = $enabled + 1\n", "numeric operand"),
        ("$bad = $missing + 1\n", "undefined variable $missing"),
        ("$bad = $$MISSING + 1\n", "undefined constant $$MISSING"),
        ("$bad = 1 +\n", "expected numeric operand"),
        ("$bad = (1 + 2\n", "missing ')'"),
    ] {
        let output = run_script(source);

        assert!(!output.status.success(), "source succeeded: {source}");
        assert!(stderr(&output).contains("line"));
        assert!(
            stderr(&output).contains(expected),
            "stderr did not contain {expected:?}: {}",
            stderr(&output)
        );
    }
}

#[test]
fn repeat_runs_block_fixed_number_of_times() {
    let output = run_script(
        r#"
repeat 3
    shellout "Hola"
end
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "Hola
Hola
Hola
"
    );
}

#[test]
fn repeat_zero_runs_no_iterations() {
    let output = run_script(
        r#"
repeat 0
    shellout "never"
end
shellout "done"
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "done
"
    );
}

#[test]
fn repeat_count_accepts_numeric_expressions_variables_and_constants() {
    let output = run_script(
        r#"
$extra = 1
$$BASE = 1
repeat $$BASE + $extra + 1 -> $i
    shellout "Index $i"
end
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "Index 1
Index 2
Index 3
"
    );
}

#[test]
fn repeat_assignments_persist_after_block() {
    let output = run_script(
        r#"
$count = 0
repeat 3
    $count = $count + 1
end
shellout "count=$count"
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "count=3
"
    );
}

#[test]
fn repeat_index_binding_exposes_one_based_iteration_numbers() {
    let output = run_script(
        r#"
repeat 3 -> $turn
    shellout "Vuelta $turn"
end
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "Vuelta 1
Vuelta 2
Vuelta 3
"
    );
}

#[test]
fn repeat_index_binding_does_not_overwrite_outer_variable() {
    let output = run_script(
        r#"
$i = "outer"
repeat 2 -> $i
    shellout "inner=$i"
end
shellout "outer=$i"
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "inner=1
inner=2
outer=outer
"
    );
}

#[test]
fn nested_repeat_supports_distinct_index_variables() {
    let output = run_script(
        r#"
repeat 2 -> $row
    repeat 2 -> $col
        shellout "$row,$col"
    end
end
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "1,1
1,2
2,1
2,2
"
    );
}

#[test]
fn while_numeric_comparison_increments_and_prints_final_count() {
    let output = run_script(
        r#"
$count = 0
while $count < 3
    shellout "Count $count"
    $count = $count + 1
end
shellout "Done at $count"
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "Count 0\nCount 1\nCount 2\nDone at 3\n");
}

#[test]
fn while_initially_false_skips_body() {
    let output = run_script(
        r#"
$count = 3
while $count < 3
    shellout "never"
    $count = $count + 1
end
shellout "done $count"
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "done 3\n");
}

#[test]
fn while_truthiness_runs_until_variable_becomes_false() {
    let output = run_script(
        r#"
$enabled = true
while $enabled
    shellout "once"
    $enabled = false
end
shellout "enabled=$enabled"
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "once\nenabled=false\n");
}

#[test]
fn while_assignments_persist_after_block() {
    let output = run_script(
        r#"
$value = 1
while $value < 2
    $value = $value + 4
end
shellout "value=$value"
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "value=5\n");
}

#[test]
fn while_condition_uses_variables_constants_and_numeric_expressions() {
    let output = run_script(
        r#"
$count = 0
$$LIMIT = 5
while $count + 1 <= $$LIMIT - 1
    $count = $count + 2
    shellout "count=$count"
end
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "count=2\ncount=4\n");
}

#[test]
fn while_allows_nested_repeat_if_and_while_blocks() {
    let output = run_script(
        r#"
$outer = 0
while $outer < 2
    if $outer == 0
        shellout "first"
    end
    repeat 2 -> $inner
        shellout "$outer:$inner"
    end
    $nested = true
    while $nested
        shellout "nested $outer"
        $nested = false
    end
    $outer = $outer + 1
end
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "first\n0:1\n0:2\nnested 0\n1:1\n1:2\nnested 1\n"
    );
}

#[test]
fn for_list_iterates_values_in_order() {
    let output = run_script(
        r#"
$names = ["Ana", "Luis", "Mora"]
for $name in $names
    shellout "Hello $name"
end
shellout "last=$name"
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "Hello Ana\nHello Luis\nHello Mora\nlast=Mora\n"
    );
}

#[test]
fn for_list_skips_empty_lists_and_allows_nested_blocks() {
    let output = run_script(
        r#"
$empty = []
for $item in $empty
    shellout "never"
end
$groups = ["A", "B"]
for $group in $groups
    repeat 2 -> $turn
        shellout "$group:$turn"
    end
end
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "A:1\nA:2\nB:1\nB:2\n");
}

#[test]
fn for_list_iterates_constant_lists_selectors_ranges_and_filters() {
    let output = run_script(
        r#"
$$NAMES = ["Ana", "Luis", "Mora"]
for $name in $$NAMES
    shellout "const=$name"
end
$numbers = [1, 3, 5, 8]
for $number in $numbers[2..3]
    shellout "range=$number"
end
for $number in $numbers[>3]
    shellout "filter=$number"
end
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "const=Ana\nconst=Luis\nconst=Mora\nrange=3\nrange=5\nfilter=5\nfilter=8\n"
    );
}

#[test]
fn list_filters_are_truthy_when_non_empty_and_falsey_when_empty_in_blocks() {
    let output = run_script(
        r#"
$numbers = [1, 3, 5]
if $numbers[>3]
    shellout "if non-empty"
end
if $numbers[>10]
    shellout "bad empty if"
else
    shellout "if empty skipped"
end
$limit = 2
while $numbers[>$limit]
    shellout "while non-empty $limit"
    $limit = $limit + 10
end
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "if non-empty\nif empty skipped\nwhile non-empty 2\n"
    );
}

#[test]
fn nested_control_blocks_work_in_combination() {
    let output = run_script(
        r#"
$groups = ["A", "B"]
$numbers = [1, 2]
for $group in $groups
    for $number in $numbers
        if $number == 1
            shellout "$group:$number:first"
        else
            $again = true
            while $again
                shellout "$group:$number:again"
                $again = false
            end
        end
    end
end
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "A:1:first\nA:2:again\nB:1:first\nB:2:again\n"
    );
}

#[test]
fn for_list_errors_cover_malformed_headers_non_lists_and_missing_end() {
    for (source, expected) in [
        (
            "for\n    shellout \"bad\"\nend\n",
            "for syntax is for $item in $list",
        ),
        (
            "for item in $names\n    shellout \"bad\"\nend\n",
            "for item destination must be a variable",
        ),
        (
            "for $item $names\n    shellout \"bad\"\nend\n",
            "for syntax is for $item in $list",
        ),
        (
            "$name = \"Ana\"\nfor $item in $name\n    shellout $item\nend\n",
            "for iterable must be a list",
        ),
        (
            "for $item in $missing\n    shellout $item\nend\n",
            "undefined variable $missing",
        ),
        (
            "$names = [\"Ana\"]\nfor $item in $names\n    shellout $item\n",
            "missing end for for",
        ),
    ] {
        let output = run_script(source);

        assert!(!output.status.success(), "source succeeded: {source}");
        assert!(stderr(&output).contains("line"));
        assert!(
            stderr(&output).contains(expected),
            "stderr did not contain {expected:?}: {}",
            stderr(&output)
        );
    }
}

#[test]
fn while_errors_cover_missing_end_malformed_condition_and_safety_limit() {
    for (source, expected) in [
        (
            "while true\n    shellout \"no end\"\n",
            "missing end for while",
        ),
        (
            "$count = 0\nwhile $count <\n    shellout \"bad\"\nend\n",
            "expected string, number, boolean, or numeric expression",
        ),
        (
            "while true\n    shellout \"infinite\"\nend\n",
            "while loop exceeded safety limit of 10000 iterations",
        ),
    ] {
        let output = run_script(source);

        assert!(!output.status.success(), "source succeeded: {source}");
        assert!(stderr(&output).contains("line"));
        assert!(
            stderr(&output).contains(expected),
            "stderr did not contain {expected:?}: {}",
            stderr(&output)
        );
    }
}

#[test]
fn repeat_errors_cover_invalid_counts_destinations_and_structure() {
    for (source, expected) in [
        (
            "repeat -1\n    shellout \"no\"\nend\n",
            "repeat count cannot be negative",
        ),
        (
            "repeat 2.5\n    shellout \"no\"\nend\n",
            "repeat count must be an integer",
        ),
        (
            "repeat 3 -> bad\n    shellout \"no\"\nend\n",
            "repeat index destination must be a variable",
        ),
        (
            "repeat 3 -> $$I\n    shellout \"no\"\nend\n",
            "repeat index destination must be a variable",
        ),
        (
            "repeat 1\n    shellout \"no end\"\n",
            "missing end for repeat",
        ),
        (
            "repeat 1 +\n    shellout \"no\"\nend\n",
            "expected numeric operand",
        ),
    ] {
        let output = run_script(source);

        assert!(!output.status.success(), "source succeeded: {source}");
        assert!(stderr(&output).contains("line"));
        assert!(
            stderr(&output).contains(expected),
            "stderr did not contain {expected:?}: {}",
            stderr(&output)
        );
    }
}

#[test]
fn exit_without_code_exits_zero_and_stops_later_statements() {
    let output = run_script(
        r#"
shellout "before"
exit
shellout "after"
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "before\n");
}

#[test]
fn exit_zero_exits_zero() {
    let output = run_script("exit 0\n");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "");
}

#[test]
fn exit_one_exits_one() {
    let output = run_script("exit 1\n");

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(stdout(&output), "");
}

#[test]
fn exit_code_accepts_numeric_expressions_variables_and_constants() {
    let output = run_script(
        r#"
$extra = 1
$$BASE = 2
exit $$BASE + $extra
"#,
    );

    assert_eq!(output.status.code(), Some(3));
    assert_eq!(stdout(&output), "");
}

#[test]
fn exit_inside_if_stops_whole_script() {
    let output = run_script(
        r#"
if true
    shellout "inside"
    exit 0
end
shellout "after"
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "inside\n");
}

#[test]
fn exit_inside_repeat_stops_whole_script() {
    let output = run_script(
        r#"
repeat 3
    shellout "loop"
    exit 0
end
shellout "after"
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "loop\n");
}

#[test]
fn exit_inside_while_stops_whole_script() {
    let output = run_script(
        r#"
while true
    shellout "loop"
    exit 0
end
shellout "after"
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "loop\n");
}

#[test]
fn exit_inside_function_stops_whole_script() {
    let output = run_script(
        r#"
fn stop
    shellout "function"
    exit 0
end
call stop
shellout "after"
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "function\n");
}

#[test]
fn exit_rejects_invalid_codes() {
    for (source, expected) in [
        (
            "exit \"no\"\n",
            "exit code must be an integer from 0 to 255",
        ),
        ("exit true\n", "exit code must be an integer from 0 to 255"),
        ("exit [1]\n", "exit code must be an integer from 0 to 255"),
        ("exit 1.5\n", "exit code must be an integer"),
        ("exit -1\n", "exit code must be between 0 and 255"),
        ("exit 256\n", "exit code must be between 0 and 255"),
        ("exit 1 extra\n", "unexpected text in numeric expression"),
    ] {
        let output = run_script(source);

        assert!(!output.status.success(), "source succeeded: {source}");
        assert!(stderr(&output).contains("line 1"));
        assert!(
            stderr(&output).contains(expected),
            "stderr did not contain {expected:?}: {}",
            stderr(&output)
        );
    }
}

#[test]
fn lists_accept_empty_assignment_and_index_values() {
    let output = run_script(
        r#"
$empty = []
$names = ["Ana", "Luis", "Mora"]
$numbers = [1, 2 + 3, 6]
$flags = [true, false]
shellout $names[1]
shellout $numbers[2]
shellout $flags[1]
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "Ana\n5\ntrue\n");
}

#[test]
fn lists_accept_variable_and_constant_elements_and_interpolation_indexing() {
    let output = run_script(
        r#"
$name = "Ada"
$$AGE = 37
$refs = [$name, $$AGE, false]
$names = ["Ana", "Luis", "Mora"]
$idx = 1 + 1
shellout "Second: $names[2]"
shellout $refs[1]
shellout $refs[2]
shellout $names[$idx]
shellout $names[1 + 2]
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "Second: Luis\nAda\n37\nLuis\nMora\n");
}

#[test]
fn list_indexing_errors_are_clear() {
    for (source, expected) in [
        (
            "$names = [\"Ana\"]\nshellout $names[0]\n",
            "list index must be 1 or greater",
        ),
        (
            "$names = [\"Ana\"]\nshellout $names[-1]\n",
            "list index must be 1 or greater",
        ),
        (
            "$names = [\"Ana\"]\nshellout $names[1.5]\n",
            "list index must be an integer",
        ),
        (
            "$names = [\"Ana\"]\nshellout $names[\"x\"]\n",
            "list index must be numeric",
        ),
        (
            "$names = [\"Ana\"]\nshellout $names[2]\n",
            "list index 2 out of range",
        ),
        ("shellout $missing[1]\n", "undefined variable $missing"),
        (
            "$name = \"Ana\"\nshellout $name[1]\n",
            "cannot index non-list value $name",
        ),
        (
            "$names = [\"Ana\"]\nshellout $names[1\n",
            "malformed list index syntax",
        ),
    ] {
        let output = run_script(source);

        assert!(!output.status.success(), "source succeeded: {source}");
        assert!(stderr(&output).contains("line"));
        assert!(
            stderr(&output).contains(expected),
            "stderr did not contain {expected:?}: {}",
            stderr(&output)
        );
    }
}

#[test]
fn advanced_list_selectors_return_scalars_and_lists() {
    let output = run_script(
        r#"
$names = ["Ana", "Luis", "Mora", "Zoe"]
shellout $names[first]
shellout $names[last]
shellout $names[..]
shellout $names[1,3]
shellout $names[1..3]
shellout $names[..3]
shellout $names[3..]
shellout $names[first..last]
shellout "pick=$names[1,3] first=$names[first]"
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "Ana\nZoe\n[Ana, Luis, Mora, Zoe]\n[Ana, Mora]\n[Ana, Luis, Mora]\n[Ana, Luis, Mora]\n[Mora, Zoe]\n[Ana, Luis, Mora, Zoe]\npick=[Ana, Mora] first=Ana\n"
    );
}

#[test]
fn advanced_list_filters_return_matching_items() {
    let output = run_script(
        r#"
$numbers = [1, 3, 5, 8, 10]
$names = ["Ana", "Luis", "Ana"]
$flags = [true, false, true]
$mixed = ["Ana", 3, true]
shellout $numbers[==5]
shellout $numbers[!=5]
shellout $numbers[>5]
shellout $numbers[>=5]
shellout $numbers[<5]
shellout $numbers[<=5]
shellout $names[=="Ana"]
shellout $names[!="Ana"]
shellout $flags[==true]
shellout $flags[!=true]
shellout $mixed[==3]
shellout $mixed[!=3]
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "[5]\n[1, 3, 8, 10]\n[8, 10]\n[5, 8, 10]\n[1, 3]\n[1, 3, 5]\n[Ana, Ana]\n[Luis]\n[true, true]\n[false]\n[3]\n[Ana, true]\n"
    );
}

#[test]
fn advanced_list_selector_and_filter_errors_are_clear() {
    for (source, expected) in [
        (
            "$names = [\"Ana\", \"Luis\"]\nshellout $names[3..1]\n",
            "reverse list ranges are not supported",
        ),
        (
            "$names = [\"Ana\", \"Luis\"]\nshellout $names[0,1]\n",
            "list index must be 1 or greater",
        ),
        (
            "$names = [\"Ana\", \"Luis\"]\nshellout $names[first..3]\n",
            "list index 3 out of range",
        ),
        (
            "$names = [\"Ana\", \"Luis\"]\nshellout $names[1,,2]\n",
            "expected value after comma",
        ),
        (
            "$names = [\"Ana\", \"Luis\"]\nshellout $names[>\"Ana\"]\n",
            "ordered string filters are not supported",
        ),
        (
            "$mixed = [1, \"Ana\"]\nshellout $mixed[>1]\n",
            "ordered list filters require numeric items",
        ),
    ] {
        let output = run_script(source);
        assert!(!output.status.success(), "source succeeded: {source}");
        assert!(stderr(&output).contains("line"));
        assert!(
            stderr(&output).contains(expected),
            "stderr did not contain {expected:?}: {}",
            stderr(&output)
        );
    }
}

#[test]
fn list_mutations_append_prepend_remove_replace_and_persist() {
    let output = run_script(
        r#"
$names = ["Ana", "Luis"]
append $names, "Mora"
prepend $names, "Iris"
$names[2] = "Ada"
remove $names[last]
remove $names[first]
shellout $names[..]
$empty = []
append $empty, "tail"
prepend $empty, "head"
shellout "$empty[1]|$empty[last]"
$remove = ["first", "middle", "last"]
remove $remove[first]
remove $remove[last]
shellout $remove[..]
$nums = [1, 2, 3]
remove $nums[2]
shellout $nums[..]
$replace = ["a", "b", "c"]
$replace[first] = "first"
$replace[last] = "last"
$replace[2] = "middle"
shellout $replace[..]
shellout "persist=$names[1]"
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "[Ada, Luis]\nhead|tail\n[middle]\n[1, 3]\n[first, middle, last]\npersist=Ada\n"
    );
}

#[test]
fn list_mutations_accept_scalar_values() {
    let output = run_script(
        r#"
$value = "Zoe"
$$AGE = 37
$items = []
append $items, "Ana"
append $items, 2 + 3
append $items, true
append $items, $value
prepend $items, $$AGE
$items[last] = false
$items[2] = $value
$items[first] = 1 + 1
shellout $items[..]
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "[2, Zoe, 5, true, false]\n");
}

#[test]
fn list_mutation_errors_are_clear() {
    for (source, expected) in [
        (
            "$$NAMES = [\"Ana\"]\nappend $$NAMES, \"Luis\"\n",
            "mutation target must be a variable list",
        ),
        ("append $missing, \"Luis\"\n", "undefined variable $missing"),
        (
            "$name = \"Ana\"\nappend $name, \"Luis\"\n",
            "mutation target must be a list",
        ),
        (
            "$names = [\"Ana\"]\nremove $names[2]\n",
            "list index 2 out of range",
        ),
        ("$names = []\nremove $names[first]\n", "list is empty"),
        ("$names = []\n$names[last] = \"Ana\"\n", "list is empty"),
        (
            "$names = [\"Ana\"]\nremove $names[1,2]\n",
            "mutation index must select exactly one item",
        ),
        (
            "$names = [\"Ana\"]\nremove $names[..]\n",
            "mutation index must select exactly one item",
        ),
        (
            "$names = [\"Ana\"]\nremove $names[==\"Ana\"]\n",
            "mutation index must select exactly one item",
        ),
        (
            "$names = [\"Ana\", \"Luis\"]\n$names[1..2] = \"bad\"\n",
            "mutation index must select exactly one item",
        ),
        (
            "$names = [\"Ana\"]\n$names[==\"Ana\"] = \"bad\"\n",
            "mutation index must select exactly one item",
        ),
        (
            "$names = [\"Ana\"]\nappend $names \"Luis\"\n",
            "append requires exactly two arguments",
        ),
        (
            "$names = [\"Ana\"]\nappend $names, [\"nested\"]\n",
            "nested lists are not supported yet",
        ),
    ] {
        let output = run_script(source);
        assert!(!output.status.success(), "source succeeded: {source}");
        assert!(stderr(&output).contains("line"));
        assert!(
            stderr(&output).contains(expected),
            "stderr did not contain {expected:?}: {}",
            stderr(&output)
        );
    }
}

#[test]
fn break_and_continue_control_loop_iteration() {
    let output = run_script(
        r#"
repeat 4 -> $i
    if $i == 2
        continue
    end
    if $i == 4
        break
    end
    shellout "r$i"
end
$items = ["a", "skip", "b", "stop", "c"]
for $item in $items
    if $item == "skip"
        continue
    end
    if $item == "stop"
        break
    end
    shellout "f$item"
end
$count = 0
while $count < 5
    $count = $count + 1
    if $count == 2
        continue
    end
    if $count == 4
        break
    end
    shellout "w$count"
end
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "r1\nr3\nfa\nfb\nw1\nw3\n");
}

#[test]
fn break_continue_and_return_outside_their_scopes_are_errors() {
    for (source, expected) in [
        ("break\n", "break outside loop"),
        ("continue\n", "continue outside loop"),
        ("return \"nope\"\n", "return outside function"),
    ] {
        let output = run_script(source);
        assert!(!output.status.success(), "source succeeded: {source}");
        assert!(
            stderr(&output).contains(expected),
            "stderr did not contain {expected:?}: {}",
            stderr(&output)
        );
    }
}

#[test]
fn function_return_can_be_captured_or_discarded() {
    let output = run_script(
        r#"
fn label $name
    return "hello $name"
    shellout "unreachable"
end
call label "Ada" -> $message
shellout $message
call label "Bea"
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "hello Ada\n");
}

#[test]
fn logical_condition_operators_use_expected_precedence() {
    let output = run_script(
        r#"
$empty = []
$items = ["x"]
if not false and $items or $empty
    shellout "first"
end
if false or false and true
    shellout "bad"
else
    shellout "second"
end
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "first\nsecond\n");
}

fn run_script(source: &str) -> std::process::Output {
    let script = write_script(source);
    Command::new(env!("CARGO_BIN_EXE_shelly"))
        .arg(script)
        .output()
        .expect("run shelly binary")
}

fn check_script(source: &str) -> std::process::Output {
    let script = write_script(source);
    Command::new(env!("CARGO_BIN_EXE_shelly"))
        .args(["check"])
        .arg(script)
        .output()
        .expect("run shelly check")
}

fn write_script(source: &str) -> std::path::PathBuf {
    let path = test_dir().join("script.shelly");
    fs::write(&path, source).expect("write script");
    path
}

fn run_script_path(path: std::path::PathBuf) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_shelly"))
        .arg(path)
        .output()
        .expect("run shelly binary")
}

fn test_dir() -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "shelly-test-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    ));
    fs::create_dir_all(&path).expect("create test dir");
    path
}

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout is valid UTF-8")
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr is valid UTF-8")
}
