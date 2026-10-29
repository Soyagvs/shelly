use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn functions_accept_nested_end_blocks_and_keep_loop_control_local() {
    let source = r#"
fn nested() {
    if true
        repeat 2
            if false
                continue
            end
            shellout "ok"
        end
    elif false
        shellout "wrong"
    else
        shellout "wrong"
    end
    return "done"
}
call nested()
"#;
    let output = run_script(source);
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "ok\nok\n");
}

#[test]
fn function_nesting_matrix_executes_local_loops_and_returns() {
    let output = run_script(
        r#"
fn nested() {
    $values = ["for"]
    for $item in $values
        shellout $item
    end
    $i = 0
    while $i < 2
        $i = $i + 1
        if $i == 1
            continue
        end
        repeat 2
            break
        end
        shellout "after"
    end
    return "done"
}
call nested()
"#,
    );
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "for\nafter\n");
}

#[test]
fn functions_support_nested_collections_try_timeout_defer_and_watch() {
    let output = run_script(
        r#"
fn nested() {
    defer
        shellout "deferred"
    end
    try
        timeout 1
            watch printf "watched\\n" -> $line
                shellout $line
            end
        end
    catch
        shellout "caught"
    end
    $value = {"items": ["ok"]}
    shellout $value.items[1]
}
call nested()
"#,
    );
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "watched\nok\ndeferred\n");
}

#[test]
fn exported_and_nested_function_braces_have_strict_closures() {
    let valid = check_script(
        r#"
export fn outer() {
    fn inner() {
        return "inner"
    }
    return "outer"
}
"#,
    );
    assert!(valid.status.success(), "stderr: {}", stderr(&valid));

    for source in [
        "fn broken() {\n    if true\n        shellout \"x\"\n}\n",
        "fn broken() {\n    if true\n        shellout \"x\"\nend\n",
        "fn broken() {\n    shellout \"x\"\nend\n}\n",
    ] {
        let output = check_script(source);
        assert!(
            !output.status.success(),
            "accepted mismatched closure: {source}"
        );
    }
}

#[test]
fn checker_rejects_loop_control_that_crosses_function_scope() {
    for statement in ["break", "continue"] {
        let output = check_script(&format!(
            "while true\n    fn inner() {{\n        {statement}\n    }}\n    break\nend\n"
        ));
        assert!(
            !output.status.success(),
            "accepted {statement} in nested function"
        );
        assert!(
            stderr(&output).contains(&format!("{statement} outside loop")),
            "stderr: {}",
            stderr(&output)
        );
    }
}

#[test]
fn watch_streams_stdout_lines_to_sequential_handler() {
    let output = run_script("watch printf \"one\\ntwo\\n\" -> $line\n    shellout $line\nend\n");
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "one\ntwo\n");
}

#[test]
fn wait_requires_no_arguments() {
    let output = check_script("wait extra\n");
    assert!(!output.status.success());
}

#[test]
fn case_when_alternatives_use_or_and_reject_commas() {
    let valid = run_script("case \"b\"\nwhen \"a\" or \"b\"\n    shellout \"matched\"\nend\n");
    assert!(valid.status.success(), "stderr: {}", stderr(&valid));
    assert_eq!(stdout(&valid), "matched\n");

    for source in [
        "case \"b\"\nwhen \"a\", \"b\"\nend\n",
        "case \"b\"\nwhen \"a\", \"b\"\nend\n",
    ] {
        for output in [run_script(source), check_script(source)] {
            assert!(!output.status.success(), "accepted comma case alternatives");
            assert!(
                stderr(&output).contains("or"),
                "stderr: {}",
                stderr(&output)
            );
        }
    }
}

#[test]
fn nested_collection_mutation_v1_supports_set_append_delete() {
    let output = run_script(
        r#"
$items = [{"name": "a", "tags": ["x"]}, {"name": "b", "tags": ["y"]}]
set $items[1].tags[1] "z"
append $items[2].tags "q"
set $items[1].extra {"ok": true}
delete $items[1].tags[1]
shellout $items
"#,
    );
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output).trim(),
        "[{extra: {ok: true}, name: a, tags: []}, {name: b, tags: [y, q]}]"
    );
}

#[test]
fn collection_mutation_rejects_comma_syntax() {
    for source in [
        "set $items[1] , \"x\"\n",
        "append $items , \"x\"\n",
        "delete $items[1], \"x\"\n",
    ] {
        let output = run_script(&format!("$items = [\"a\"]\n{source}"));
        assert!(
            !output.status.success(),
            "accepted comma mutation: {source}"
        );
        assert!(
            stderr(&output).contains("whitespace-separated"),
            "stderr: {}",
            stderr(&output)
        );
    }
}

#[test]
fn prepend_rejects_comma_syntax() {
    let output = run_script("$items = [\"a\"]\nprepend $items, \"x\"\n");
    assert!(!output.status.success(), "accepted comma prepend");
    assert!(
        stderr(&output).contains("whitespace-separated"),
        "stderr: {}",
        stderr(&output)
    );
}

#[test]
fn root_collection_mutation_uses_whitespace_arguments() {
    let output = run_script("$items = []\nappend $items \"x\"\nshellout $items\n");
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output).trim(), "[x]");
}

#[test]
fn nested_collection_mutation_rejects_missing_or_invalid_paths() {
    for source in [
        "set $items[0], \"x\"\n",
        "append $items.missing \"x\"\n",
        "delete $items[1].missing\n",
    ] {
        let output = run_script(&format!("$items = [[\"a\"]]\n{source}"));
        assert!(
            !output.status.success(),
            "accepted invalid mutation: {source}"
        );
    }
}

#[test]
fn nested_collection_assignment_copies_values() {
    let output = run_script(
        r#"
$source = {"nested": ["a"]}
$copy = $source
append $copy.nested "b"
shellout $source
shellout $copy
"#,
    );
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output).lines().collect::<Vec<_>>(),
        ["{nested: [a]}", "{nested: [a, b]}"]
    );
}

#[test]
fn init_in_current_directory_creates_the_approved_scaffold() {
    let dir = test_dir();
    let output = Command::new(env!("CARGO_BIN_EXE_shelly"))
        .arg("init")
        .current_dir(&dir)
        .output()
        .expect("run shelly init");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    for file in ["Shelly.toml", "main.shelly", "README.md", ".gitignore"] {
        assert!(dir.join(file).is_file(), "missing {file}");
    }
    for directory in [
        "tests",
        "consts",
        "functions",
        "utils",
        "setup",
        "scripts",
        "modules",
        "domains",
    ] {
        assert!(
            dir.join(directory).join(".gitkeep").is_file(),
            "missing {directory}/.gitkeep"
        );
    }
    assert!(!dir.join("logs").exists());
    assert!(
        fs::read_to_string(dir.join(".gitignore"))
            .expect("read .gitignore")
            .contains("logs/")
    );
    assert!(!dir.join(dir.file_name().expect("directory name")).exists());
}

#[test]
fn init_preflights_existing_targets_without_partial_scaffold() {
    let dir = test_dir();
    let readme = "keep this README";
    fs::write(dir.join("README.md"), readme).expect("write README");
    fs::create_dir(dir.join("tests")).expect("create tests");
    fs::write(dir.join("tests/existing.shelly"), "keep this test").expect("write test");

    let output = Command::new(env!("CARGO_BIN_EXE_shelly"))
        .arg("init")
        .current_dir(&dir)
        .output()
        .expect("run shelly init");

    assert!(!output.status.success());
    assert_eq!(fs::read_to_string(dir.join("README.md")).unwrap(), readme);
    assert_eq!(
        fs::read_to_string(dir.join("tests/existing.shelly")).unwrap(),
        "keep this test"
    );
    for target in [
        "Shelly.toml",
        "main.shelly",
        ".gitignore",
        "consts",
        "functions",
        "utils",
        "setup",
        "scripts",
        "modules",
        "domains",
        "logs",
    ] {
        assert!(
            !dir.join(target).exists(),
            "unexpected partial target: {target}"
        );
    }
}

#[test]
fn init_second_run_fails_without_changing_generated_contents() {
    let dir = test_dir();
    let init = || {
        Command::new(env!("CARGO_BIN_EXE_shelly"))
            .arg("init")
            .current_dir(&dir)
            .output()
            .expect("run shelly init")
    };
    let first = init();
    assert!(first.status.success(), "stderr: {}", stderr(&first));
    let targets = ["Shelly.toml", "main.shelly", "README.md", ".gitignore"];
    let before: Vec<_> = targets
        .iter()
        .map(|name| fs::read(dir.join(name)).expect("read generated file"))
        .collect();

    let second = init();
    assert!(!second.status.success());
    for (name, contents) in targets.iter().zip(before) {
        assert_eq!(
            fs::read(dir.join(name)).unwrap(),
            contents,
            "changed {name}"
        );
    }
}

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
fn single_dollar_constants_and_lowercase_variables_define_the_new_syntax() {
    let valid = check_script(
        r#"
$APP = "Shelly"
$app = "world"
shellout "$APP $app"
"#,
    );
    assert!(valid.status.success(), "stderr: {}", stderr(&valid));

    for source in [
        "$App = \"mixed\"\n",
        "$appName = \"mixed\"\n",
        "$$NAME = \"legacy\"\n",
        "$$\n",
        "$$1\n",
    ] {
        let output = check_script(source);
        assert!(
            !output.status.success(),
            "accepted invalid syntax: {source:?}"
        );
    }
}

#[test]
fn check_does_not_execute_command_shellout_ask_or_shellin() {
    let output = check_script(
        r#"
command printf SHOULD_NOT_PRINT
shellout "SHOULD_NOT_PRINT"
ask "Question? " -> $answer
shellin $input
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "");
}

#[test]
fn runtime_assertions_report_arity_errors_instead_of_panicking() {
    for source in [
        "assertEqual \"only one\"\n",
        "assertTrue\n",
        "assertFalse true extra\n",
        "assertContains \"only one\"\n",
    ] {
        let output = run_script(source);
        assert!(
            !output.status.success(),
            "accepted malformed assertion: {source:?}"
        );
        assert!(
            stderr(&output).contains("expects"),
            "stderr: {}",
            stderr(&output)
        );
        assert!(
            !stderr(&output).contains("panicked"),
            "stderr: {}",
            stderr(&output)
        );
    }
}

#[test]
fn defer_runs_cleanup_in_lifo_order_and_on_errors() {
    let output = run_script(
        r#"
$log = ""
def defer
    $log = "$log outer"
    shellout "$log"
end
def defer
    $log = "$log inner"
end
assertTrue false
"#,
    );
    assert!(!output.status.success());
    assert!(!stderr(&output).is_empty());
    assert!(output.status.code().is_some());
}

#[test]
fn check_rejects_malformed_defer_and_nested_blocks_without_end() {
    for source in ["defer extra\nend\n", "defer\nif true\nend\n", "defer\n"] {
        let output = check_script(source);
        assert!(
            !output.status.success(),
            "accepted malformed defer: {source:?}"
        );
    }
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
fn shellout_multiline_string_prints_one_block() {
    let output = run_script(
        r#"
shellout ```
line one
  line two
```
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "line one\n  line two\n");
}

#[test]
fn multiline_assignment_shellout_file_write_append_and_call_args() {
    let dir = test_dir();
    fs::write(
        dir.join("script.shelly"),
        r#"
$value = ```
assigned
block
```
shellout $value
write "out.txt" ```
written
block
```
append "out.txt" ```
appended
block
```
read "out.txt" -> $file_text
shellout $file_text
fn echo($text) {
    shellout $text
}
call echo(```
called
block
```
)
"#,
    )
    .expect("write script");

    let output = run_script_path(dir.join("script.shelly"));

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "assigned\nblock\nwritten\nblockappended\nblock\n\ncalled\nblock\n"
    );
}

#[test]
fn unterminated_multiline_string_is_reported_by_check_and_runtime() {
    let source = "shellout ```\nmissing close\n";

    let checked = check_script(source);
    assert!(!checked.status.success());
    assert!(stderr(&checked).contains("line 1"));
    assert!(stderr(&checked).contains("unterminated multiline string"));

    let output = run_script(source);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("unterminated multiline string"));
}

#[test]
fn multiline_content_does_not_affect_block_parsing_or_comments() {
    let output = run_script(
        r#"
if true
    shellout ```
end
}
fn nope() {
if false
# not a comment
```
end
shellout "after"
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "end\n}\nfn nope() {\nif false\n# not a comment\nafter\n"
    );
}

#[test]
fn dictionaries_support_access_mutation_interpolation_and_truthiness() {
    let output = run_script(
        r#"
$user = {
    name: "Ada"
    "first-name": "Ada"
    admin: true
}
shellout $user.name
$user.name = "Grace"
$key = "first-name"
shellout $user[$key]
$user[$key] = "Amazing"
shellout "hello $user.name $user[$key]"
if $user
    shellout "truthy"
end
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "Ada\nAda\nhello Grace Amazing\ntruthy\n");
}

#[test]
fn dictionaries_support_chained_read_mutation_interpolation_and_remove() {
    let output = run_script(
        r#"
$user = {
    pais: {
        argentina: {
            city: "Buenos Aires"
            code: "AR"
        }
    }
}
$country = "argentina"
shellout $user.pais.argentina.city
shellout $user["pais"]["argentina"].city
shellout $user.pais[$country].city
$user.pais.argentina.city = "Córdoba"
shellout "city=$user.pais.argentina.city"
$user["pais"]["argentina"].city = "Rosario"
shellout $user.pais.argentina.city
remove $user["pais"]["argentina"].code
shellout $user.pais.argentina
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "Buenos Aires\nBuenos Aires\nBuenos Aires\ncity=Córdoba\nRosario\n{city: Rosario}\n"
    );
}

#[test]
fn dictionaries_report_chained_missing_keys_non_dictionary_and_constant_mutation() {
    let missing = run_script("$user = { pais: {} }\nshellout $user.pais.argentina.city\n");
    assert!(!missing.status.success());
    assert!(stderr(&missing).contains("missing key 'argentina'"));

    let wrong_target = run_script("$user = { pais: \"AR\" }\nshellout $user.pais.city\n");
    assert!(!wrong_target.status.success());
    assert!(stderr(&wrong_target).contains("non-dictionary"));

    let constant = run_script(
        "$USER = { pais: { argentina: { city: \"BA\" } } }\n$USER.pais.argentina.city = \"Cordoba\"\n",
    );
    assert!(!constant.status.success());
    assert!(stderr(&constant).contains("variable dictionary"));
}

#[test]
fn remove_preserves_list_and_file_behavior_while_supporting_dictionary_paths() {
    let dir = test_dir();
    fs::write(
        dir.join("script.shelly"),
        r#"
$items = ["a", "b", "c"]
remove $items[2]
shellout $items
write "gone.txt" "bye"
remove "gone.txt"
exists "gone.txt" -> $exists
shellout $exists
$user = { profile: { city: "BA", other: "kept" } }
remove $user.profile.city
shellout $user.profile
"#,
    )
    .expect("write script");

    let output = run_script_path(dir.join("script.shelly"));

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "[a, c]\nfalse\n{other: kept}\n");
}

#[test]
fn dictionaries_report_missing_keys_and_non_dictionary_mutation() {
    let missing = run_script("$user = { name: \"Ada\" }\nshellout $user.age\n");
    assert!(!missing.status.success());
    assert!(stderr(&missing).contains("missing key 'age'"));

    let wrong_target = run_script("$name = \"Ada\"\n$name.first = \"Grace\"\n");
    assert!(!wrong_target.status.success());
    assert!(stderr(&wrong_target).contains("non-dictionary"));
}

#[test]
fn check_accepts_dictionary_syntax_and_rejects_malformed_literal() {
    let valid = check_script("$user = { name: \"Ada\", age: 32 }\nshellout $user.name\n");
    assert!(valid.status.success(), "stderr: {}", stderr(&valid));

    let malformed = check_script("$user = { name \"Ada\" }\n");
    assert!(!malformed.status.success());
    assert!(stderr(&malformed).contains("malformed dictionary entry"));
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
fn command_timeout_succeeds_and_times_out_with_timeout_error_kind() {
    let success = run_script(r#"command printf hello timeout 1"#);
    assert!(success.status.success(), "stderr: {}", stderr(&success));
    assert_eq!(stdout(&success), "hello");

    let failure = run_script(
        r#"
try -> $error
    command sleep 1 timeout 0.01
catch
    shellout "kind=$error.kind"
end
"#,
    );
    assert!(failure.status.success(), "stderr: {}", stderr(&failure));
    assert_eq!(stdout(&failure), "kind=timeout\n");
}

#[test]
fn timeout_blocks_expire_and_inherit_into_functions_and_loops() {
    let output = run_script(
        r#"
fn wait() {
    command sleep 1
}
timeout 0.01
    call wait
end
"#,
    );
    assert!(!output.status.success());
    assert!(stderr(&output).contains("timeout"));

    let loop_output = run_script(
        r#"
timeout 0.01
    while true
        command sleep 1
    end
end
"#,
    );
    assert!(!loop_output.status.success());
    assert!(stderr(&loop_output).contains("timeout"));
}

#[test]
fn nested_timeout_cannot_extend_parent_deadline() {
    let output = run_script(
        r#"
timeout 0.01
    timeout 10
        command sleep 1
    end
end
"#,
    );
    assert!(!output.status.success());
    assert!(stderr(&output).contains("timeout"));
}

#[test]
fn timeout_checkers_accept_fractional_values_and_reject_invalid_values() {
    for source in [
        "command printf ok timeout 1.5\n",
        "timeout 0.5\n    command printf ok\nend\n",
    ] {
        let output = check_script(source);
        assert!(output.status.success(), "stderr: {}", stderr(&output));
    }

    for source in [
        "command printf ok timeout 0\n",
        "command printf ok timeout -1\n",
        "command printf ok timeout nope\n",
        "timeout 0\n    command printf ok\nend\n",
        "timeout -1\n    command printf ok\nend\n",
        "timeout nope\n    command printf ok\nend\n",
    ] {
        let output = check_script(source);
        assert!(
            !output.status.success(),
            "accepted invalid timeout: {source:?}"
        );
    }
}

#[test]
fn command_executes_program_and_arguments_directly() {
    let output = run_script(r#"command printf hello"#);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "hello");
}

#[test]
fn command_preserves_interpolated_argument_as_one_literal_argument() {
    let output = run_script(
        r#"
$value = "x; printf BAD"
command printf $value
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "x; printf BAD");
}

#[test]
fn command_rejects_quoted_legacy_syntax() {
    let output = run_script(r#"command \"printf hello\""#);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("line 1"));
}

#[test]
fn shell_executes_intentional_shell_syntax() {
    let output = run_script(r#"shell "printf x | tr x y""#);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "y");
}

#[test]
fn command_direct_capture_supports_one_destination() {
    let output = run_script(
        r#"
command printf hello -> $out
shellout "captured=$out"
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "captured=hello\n");
}

#[test]
fn command_direct_capture_supports_three_destinations() {
    let output = run_script(
        r#"
command printf hello -> $out, $err, $code
shellout "out=<$out> err=<$err> code=$code"
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "out=<hello> err=<> code=0\n");
}

#[test]
fn command_nonzero_error_exposes_program_and_argv_in_catch() {
    let output = run_script(
        r#"
try -> $error
    command sh -c "printf err >&2; exit 7"
catch
    shellout "kind=$error.kind program=$error.program args=$error.args code=$error.code"
end
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "kind=process program=sh args=[-c, printf err >&2; exit 7] code=7\n"
    );
}

#[test]
fn command_arrow_captures_stdout_without_printing_it_directly() {
    let output = run_script(
        r#"
command printf hello -> $out
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
command printf "hello\n\n" -> $out
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
command printf blue -> $color
shellout "favorite=$color"
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "favorite=blue\n");
}

#[test]
fn command_streams_system_command_output() {
    let output = run_script(r#"command printf hello"#);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "hello");
}

#[test]
fn command_arrow_triple_captures_stdout_stderr_and_zero_code() {
    let output = run_script(
        r#"
shell "printf 'out\n'; printf 'err\n' >&2" -> $out, $err, $code
shellout "out=<$out> err=<$err> code=$code"
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "out=<out> err=<err> code=0\n");
}

#[test]
fn command_arrow_triple_captures_nonzero_status_without_failing() {
    let output = run_script(
        r#"
shell "printf 'bad\n' >&2; exit 7" -> $out, $err, $code
shellout "continued code=$code err=<$err> out=<$out>"
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "continued code=7 err=<bad> out=<>\n");
}

#[test]
fn command_arrow_one_destination_still_fails_on_nonzero_status() {
    let output = run_script(r#"shell "printf 'bad\n' >&2; exit 7" -> $out"#);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("line 1"));
    assert!(stderr(&output).contains("shell exited with status"));
}

#[test]
fn check_accepts_command_triple_capture() {
    let output = check_script(r#"command printf hello -> $out, $err, $code"#);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
}

#[test]
fn command_arrow_rejects_two_destinations() {
    for output in [
        run_script(r#"command printf hello -> $out, $err"#),
        check_script(r#"command printf hello -> $out, $err"#),
    ] {
        assert!(!output.status.success());
        assert!(stderr(&output).contains("line 1"));
        assert!(stderr(&output).contains("2 destinations are not supported"));
    }
}

#[test]
fn command_arrow_rejects_more_than_three_destinations() {
    let output = check_script(r#"command printf hello -> $out, $err, $code, $extra"#);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("line 1"));
    assert!(stderr(&output).contains("at most 3 destinations"));
}

#[test]
fn old_capture_syntax_is_rejected() {
    let output = run_script(r#"capture $out, "printf hello""#);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("line 1"));
    assert!(stderr(&output).contains("capture syntax was removed"));
}

#[test]
fn command_arrow_rejects_constant_destination() {
    for output in [
        run_script(r#"command pwd -> $CWD"#),
        check_script(r#"command pwd -> $out, $ERR, $code"#),
    ] {
        assert!(!output.status.success());
        assert!(stderr(&output).contains("line 1"));
        assert!(stderr(&output).contains("destination must be a variable"));
    }
}

#[test]
fn command_arrow_rejects_malformed_destination() {
    for source in [
        r#"command printf hello -> "#,
        r#"command printf hello -> out"#,
        r#"command printf hello -> $9out"#,
        r#"command printf hello -> $out extra"#,
        r#"command printf hello -> $out, err, $code"#,
        r#"command printf hello -> $out, $err, $9code"#,
    ] {
        let output = check_script(source);

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
fn choose_by_number_stores_selected_value_and_prints_menu() {
    let output = run_script_with_stdin(
        r#"
choose "Pick one" ["A", "B", "C"] -> $choice
shellout "choice=$choice"
"#,
        b"2\n",
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "Pick one\n1) A\n2) B\n3) C\nChoose: choice=B\n"
    );
}

#[test]
fn choose_by_exact_text_stores_selected_value() {
    let output = run_script_with_stdin(
        r#"
choose "Pick one" ["A", "B"] -> $choice
shellout $choice
"#,
        b"B\n",
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "Pick one\n1) A\n2) B\nChoose: B\n");
}

#[test]
fn choose_invalid_input_reprompts_then_accepts_valid_input() {
    let output = run_script_with_stdin(
        r#"
choose "Pick one" ["A", "B"] -> $choice
shellout $choice
"#,
        b"nope\n1\n",
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "Pick one\n1) A\n2) B\nChoose: Choose: A\n");
}

#[test]
fn choose_empty_option_list_fails() {
    let output = run_script(r#"choose "Pick one" [] -> $choice"#);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("choose options must not be empty"));
}

#[test]
fn check_valid_choose_syntax_succeeds() {
    let output = check_script(r#"choose "Pick one" ["A", "B"] -> $choice"#);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
}

#[test]
fn check_validates_choose_invalid_syntax() {
    for (source, expected) in [
        (r#"choose "Pick one" ["A"]"#, "choose syntax is choose"),
        (
            r#"choose "Pick one" [] -> $choice"#,
            "choose options must not be empty",
        ),
        (
            r#"choose "Pick one" ["A"] -> $CHOICE"#,
            "choose destination must be a variable",
        ),
    ] {
        let output = check_script(source);

        assert!(!output.status.success(), "source succeeded: {source}");
        assert!(
            stderr(&output).contains(expected),
            "stderr did not contain {expected:?}: {}",
            stderr(&output)
        );
    }
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
            "ask \"Question?\" -> $ANSWER\n",
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
    for path in ["README.md", "docs/language.md", "examples/ask-if.shelly"] {
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
fn variables_are_lowercase_only_and_may_be_empty() {
    let output = run_script(
        r#"
$name = "Ada"
shellout $name
$empty
shellout "empty:$empty"
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "Ada\nempty:\n");

    for source in ["$Name = \"mixed\"\n", "$EMPTY\n"] {
        let rejected = check_script(source);
        assert!(
            !rejected.status.success(),
            "accepted invalid variable: {source:?}"
        );
    }
}

#[test]
fn identifiers_must_start_with_ascii_letter() {
    for source in [
        "$9bad = \"nope\"\n",
        "$_bad = \"nope\"\n",
        "$9BAD = \"nope\"\n",
        "$_BAD = \"nope\"\n",
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
$NAME = "Shelly"
$LIMIT = 10
shellout "$NAME count=$count enabled=$enabled limit=$LIMIT"
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "Shelly count=42 enabled=true limit=10\n");

    let legacy = run_script("shellout \"legacy $$NAME\"\n");
    assert!(!legacy.status.success(), "source succeeded");
    assert!(stderr(&legacy).contains("legacy '$$' identifier prefix is not supported; use '$'"));
}

#[test]
fn constants_require_uppercase_initial_values_and_cannot_be_reassigned() {
    let variable = run_script("$name = \"value\"\n");
    assert!(variable.status.success(), "stderr: {}", stderr(&variable));

    let mixed_case = run_script("$Name = \"value\"\n");
    assert!(!mixed_case.status.success(), "source succeeded");
    assert!(stderr(&mixed_case).contains("all uppercase or all lowercase"));

    for source in ["$NAME =\n", "$NAME = \"first\"\n$NAME = \"second\"\n"] {
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
$APP = "Shelly"

fn person($name, $age, $prefix = "Hola") {
    global $outer, $APP
    $outer = "inside"
    shellout "$prefix $name age=$age outer=$outer app=$APP"
}
call person("Agustin", 27)
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
$APP = "Shelly"

fn greet($name, $app, $age, $enabled) {
    shellout "$name from $app age=$age enabled=$enabled"
}
call greet($name, $APP, 27, true)
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "Ada from Shelly age=27 enabled=true\n");
}

#[test]
fn function_call_reference_arguments_fail_when_undefined() {
    for (source, expected) in [
        (
            "fn greet($name) {\n}\ncall greet($missing)\n",
            "undefined variable $missing",
        ),
        (
            "fn greet($app) {\n}\ncall greet($MISSING)\n",
            "undefined constant $MISSING",
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
        ("fn missing() {\nshellout \"no end\"\n", "missing }"),
        ("fn same() {\n}\nfn same() {\n}\n", "duplicate function"),
        ("does_not_exist \"x\"\n", "unknown command"),
        ("fn old $name\nend\n", "old function syntax was removed"),
        (
            "fn greet($name) {\n}\ngreet \"Ada\"\n",
            "function calls must use call",
        ),
        (
            "fn greet($name) {\n}\ncall greet \"Ada\"\n",
            "old call syntax was removed",
        ),
        (
            "fn greet($name) {\n}\ncall greet(\"Ada\"\n",
            "call arguments must be enclosed in parentheses",
        ),
        ("fn one($name) {\n}\ncall one()\n", "wrong argument count"),
        (
            "fn bad($first = \"x\", $second) {\n}\n",
            "required parameter cannot follow default",
        ),
        (
            "fn bad() {\nglobal name\n}\ncall bad()\n",
            "expected identifier",
        ),
        (
            "$outer = \"outside\"\nfn bad() {\nshellout $outer\n}\ncall bad()\n",
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
        ("if true)\nend\n", "unexpected ')'"),
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
fn case_blocks_cover_matching_else_no_match_multiple_and_first_match() {
    let output = run_script(
        r#"
$color = "blue"
case $color
    when "red"
        shellout "red"
    when "blue"
        shellout "blue"
    else
        shellout "fallback"
end
case "missing"
    when "other"
        shellout "bad"
    else
        shellout "else"
end
case "none"
    when "other"
        shellout "bad"
end
case "b"
    when "a" or "b" or "c"
        shellout "multi"
end
case "same"
    when "same"
        shellout "first"
    when "same"
        shellout "second"
end
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "blue\nelse\nmulti\nfirst\n");
}

#[test]
fn case_blocks_match_numbers_booleans_constants_variables_and_nested_blocks() {
    let output = run_script(
        r#"
$count = 2
$enabled = true
$MODE = "prod"
case $count
    when 1
        shellout "one"
    when 1 + 1
        shellout "two"
end
case $enabled
    when false
        shellout "disabled"
    when true
        shellout "enabled"
end
case $MODE
    when "prod"
        if $enabled
            shellout "prod-if"
        end
        case $count
            when 2
                shellout "nested-case"
        end
end
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "two\nenabled\nprod-if\nnested-case\n");
}

#[test]
fn check_accepts_valid_case_syntax() {
    let output = check_script(
        r#"
case "a"
    when "a" or "b"
        shellout "ok"
    else
        shellout "fallback"
end
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
}

#[test]
fn case_errors_cover_malformed_forms() {
    for (source, expected) in [
        ("case\n    when \"a\"\nend\n", "case value required"),
        ("when \"a\"\n", "when outside case"),
        (
            "case \"a\"\n    when \"a\"\n    else\n    else\nend\n",
            "duplicate else",
        ),
        (
            "case \"a\"\n    else\n    when \"a\"\nend\n",
            "when after else",
        ),
        ("case \"a\"\n    when\nend\n", "when value required"),
        ("case \"a\"\n    when \"a\"\n", "missing end for case"),
        ("case \"a\"\nend\n", "case requires at least one when"),
    ] {
        for output in [run_script(source), check_script(source)] {
            assert!(!output.status.success(), "source succeeded: {source}");
            assert!(
                stderr(&output).contains(expected),
                "stderr did not contain {expected:?}: {}",
                stderr(&output)
            );
        }
    }
}

#[test]
fn import_loads_exported_constant() {
    let dir = test_dir();
    fs::write(dir.join("module.shelly"), "export $GREETING = \"Hola\"\n").expect("write module");
    fs::write(
        dir.join("main.shelly"),
        "import \"module.shelly\"\nshellout $GREETING\n",
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
        "export fn greet($name) {\n    shellout \"Hola $name\"\n}\n",
    )
    .expect("write module");
    fs::write(
        dir.join("main.shelly"),
        "import \"module.shelly\"\ncall greet(\"Ada\")\n",
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
        ("shellout $SECRET\n", "undefined constant $SECRET"),
        ("hidden\n", "unknown command"),
    ] {
        let dir = test_dir();
        fs::write(
            dir.join("module.shelly"),
            "$secret = \"private var\"\n$SECRET = \"private const\"\nfn hidden() {\n    shellout \"private fn\"\n}\n",
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
        "shellout \"SHOULD NOT RUN\"\nexport $VALUE = \"ok\"\n",
    )
    .expect("write module");
    fs::write(
        dir.join("main.shelly"),
        "import \"module.shelly\"\nshellout $VALUE\n",
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
    fs::write(dir.join("module.shelly"), "export $VALUE = \"once\"\n").expect("write module");
    fs::write(
        dir.join("main.shelly"),
        "import \"module.shelly\"\nimport \"module.shelly\"\nshellout $VALUE\n",
    )
    .expect("write main");
    let output = run_script_path(dir.join("main.shelly"));
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "once\n");
}

#[test]
fn check_traverses_transitive_imports_and_includes_without_execution() {
    let dir = test_dir();
    fs::write(
        dir.join("leaf.shelly"),
        "shellout \"MUST_NOT_EXECUTE\"\nexport $VALUE = \"ok\"\n",
    )
    .expect("write leaf");
    fs::write(dir.join("middle.shelly"), "include \"leaf.shelly\"\n").expect("write middle");
    fs::write(
        dir.join("main.shelly"),
        "import \"middle.shelly\"\nshellout $VALUE\n",
    )
    .expect("write main");

    let output = run_shelly_check(&dir, &["main.shelly"]);
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "");
}

#[test]
fn check_reports_missing_and_malformed_transitive_dependencies() {
    let dir = test_dir();
    fs::write(dir.join("main.shelly"), "import \"middle.shelly\"\n").expect("write main");
    fs::write(dir.join("middle.shelly"), "include \"missing.shelly\"\n").expect("write middle");
    let output = run_shelly_check(&dir, &["main.shelly"]);
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("missing.shelly"),
        "stderr: {}",
        stderr(&output)
    );

    fs::write(dir.join("middle.shelly"), "if true\n").expect("write malformed middle");
    let output = run_shelly_check(&dir, &["main.shelly"]);
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("middle.shelly"),
        "stderr: {}",
        stderr(&output)
    );
    assert!(
        stderr(&output).contains("missing end"),
        "stderr: {}",
        stderr(&output)
    );
}

#[test]
fn check_detects_direct_two_node_and_mixed_import_include_cycles() {
    for (name, files) in [
        (
            "direct",
            vec![("main.shelly", "import \\\"main.shelly\\\"\\n")],
        ),
        (
            "two-node",
            vec![
                ("main.shelly", "import \\\"other.shelly\\\"\\n"),
                ("other.shelly", "import \\\"main.shelly\\\"\\n"),
            ],
        ),
        (
            "mixed",
            vec![
                ("main.shelly", "import \\\"other.shelly\\\"\\n"),
                ("other.shelly", "include \\\"main.shelly\\\"\\n"),
            ],
        ),
    ] {
        let dir = test_dir();
        for (file, source) in files {
            fs::write(dir.join(file), source).expect("write cycle file");
        }
        let output = run_shelly_check(&dir, &["main.shelly"]);
        assert!(!output.status.success(), "accepted {name} cycle");
        assert!(
            stderr(&output).contains("cycle"),
            "stderr: {}",
            stderr(&output)
        );
    }
}

#[test]
fn check_json_reports_dependency_diagnostics_and_preserves_duplicate_imports() {
    let dir = test_dir();
    fs::write(dir.join("dep.shelly"), "if true\n").expect("write dep");
    fs::write(
        dir.join("main.shelly"),
        "import \"dep.shelly\"\nimport \"dep.shelly\"\n",
    )
    .expect("write main");

    let output = run_shelly_check(&dir, &["--format", "json", "main.shelly"]);
    assert!(!output.status.success());
    assert_eq!(stderr(&output), "");
    let diagnostics = stdout(&output);
    assert!(diagnostics.starts_with('['));
    assert!(diagnostics.contains("dep.shelly"));
    assert!(
        diagnostics.matches("dep.shelly").count() >= 2,
        "diagnostics: {diagnostics}"
    );
    assert!(
        diagnostics.contains("missing end"),
        "diagnostics: {diagnostics}"
    );
}

#[test]
fn numeric_expressions_support_precedence_parentheses_floats_negatives_and_references() {
    let output = run_script(
        r#"
$a = 1 + 2 * 3
$b = (1 + 2) * 3
$c = 10.5 + 2.25
$d = -3 * 2
$BASE = 10
$total = $BASE * $c
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
        ("$bad = $MISSING + 1\n", "undefined constant $MISSING"),
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
$BASE = 1
repeat $BASE + $extra + 1 -> $i
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
$LIMIT = 5
while $count + 1 <= $LIMIT - 1
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
$NAMES = ["Ana", "Luis", "Mora"]
for $name in $NAMES
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
fn files_collects_txt_paths_and_for_iterates_results() {
    let dir = test_dir();
    fs::write(dir.join("b.txt"), "b").expect("write b");
    fs::write(dir.join("a.txt"), "a").expect("write a");
    fs::write(dir.join("ignore.md"), "ignore").expect("write ignore");
    fs::write(
        dir.join("script.shelly"),
        r#"
files "*.txt" -> $items
for $file in $items
    shellout $file
end
"#,
    )
    .expect("write script");

    let output = run_script_path(dir.join("script.shelly"));

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "a.txt\nb.txt\n");
}

#[test]
fn files_shellout_displays_list_without_iterating() {
    let dir = test_dir();
    fs::write(dir.join("one.txt"), "one").expect("write one");
    fs::write(dir.join("two.txt"), "two").expect("write two");
    fs::write(
        dir.join("script.shelly"),
        "files \"*.txt\" -> $items\nshellout $items\n",
    )
    .expect("write script");

    let output = run_script_path(dir.join("script.shelly"));

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "[one.txt, two.txt]\n");
}

#[test]
fn files_empty_match_is_empty_list_and_falsey() {
    let output = run_script(
        r#"
files "missing*.txt" -> $items
shellout $items
if $items
    shellout "bad"
else
    shellout "empty"
end
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "[]\nempty\n");
}

#[test]
fn files_supports_recursive_glob_without_symlink_traversal() {
    let dir = test_dir();
    fs::create_dir_all(dir.join("src/test/unit")).expect("create nested dirs");
    fs::write(dir.join("root.shelly"), "root").expect("write root");
    fs::write(dir.join("src/test_one.shelly"), "one").expect("write one");
    fs::write(dir.join("src/test/unit/test_two.shelly"), "two").expect("write two");
    fs::write(dir.join("src/test/unit/nope.txt"), "nope").expect("write txt");
    #[cfg(unix)]
    std::os::unix::fs::symlink(dir.join("src/test"), dir.join("linked")).expect("symlink");
    fs::write(
        dir.join("script.shelly"),
        "files \"**/*.shelly\" -> $items\nshellout $items\nfiles \"src/**/test_*.shelly\" -> $nested\nshellout $nested\n",
    )
    .expect("write script");

    let output = run_script_path(dir.join("script.shelly"));

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "[root.shelly, script.shelly, src/test/unit/test_two.shelly, src/test_one.shelly]\n[src/test/unit/test_two.shelly, src/test_one.shelly]\n"
    );
}

#[test]
fn files_supports_subdirectory_segment_glob() {
    let dir = test_dir();
    fs::create_dir_all(dir.join("dir")).expect("create dir");
    fs::write(dir.join("dir/b.txt"), "b").expect("write b");
    fs::write(dir.join("dir/a.txt"), "a").expect("write a");
    fs::write(dir.join("dir/c.md"), "c").expect("write c");
    fs::write(
        dir.join("script.shelly"),
        "files \"dir/*.txt\" -> $items\nshellout $items\n",
    )
    .expect("write script");

    let output = run_script_path(dir.join("script.shelly"));

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "[dir/a.txt, dir/b.txt]\n");
}

#[test]
fn check_accepts_files_and_rejects_invalid_files_syntax() {
    let valid = check_script("files \"**/*.shelly\" -> $items\n");
    assert!(valid.status.success(), "stderr: {}", stderr(&valid));

    for (source, expected) in [
        ("files\n", "files syntax is files"),
        ("files *.txt -> $items\n", "files requires a string pattern"),
        ("files \"*.txt\"\n", "files syntax is files"),
        (
            "files \"*.txt\" -> $ITEMS\n",
            "files destination must be a variable",
        ),
        (
            "files \"*.txt\" -> items\n",
            "files destination must be a variable",
        ),
        (
            "files \"*.txt\" -> $items extra\n",
            "files destination must be a variable",
        ),
    ] {
        let output = check_script(source);
        assert!(!output.status.success(), "source should fail: {source}");
        assert!(
            stderr(&output).contains(expected),
            "stderr did not contain {expected:?}: {}",
            stderr(&output)
        );
    }
}

#[test]
fn file_commands_create_paths_relative_to_script_dir_and_preserve_existing_files() {
    let dir = test_dir();
    fs::write(dir.join("keep.txt"), "keep").expect("write existing");
    fs::write(
        dir.join("script.shelly"),
        r#"
create "./nested/file.txt"
create "./nested/dir/"
create "keep.txt"
write "nested/dir/value.txt" "value"
read "./nested/dir/value.txt" -> $value
shellout $value
"#,
    )
    .expect("write script");

    let output = run_script_path(dir.join("script.shelly"));

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "value\n");
    assert!(dir.join("nested/file.txt").is_file());
    assert!(dir.join("nested/dir").is_dir());
    assert_eq!(fs::read_to_string(dir.join("keep.txt")).unwrap(), "keep");
}

#[test]
fn file_commands_copy_files_and_directories_and_fail_when_destination_exists() {
    let dir = test_dir();
    fs::create_dir_all(dir.join("srcdir/sub")).expect("create src dir");
    fs::write(dir.join("source.txt"), "file").expect("write source");
    fs::write(dir.join("srcdir/sub/item.txt"), "dir").expect("write dir item");
    fs::write(dir.join("exists.txt"), "exists").expect("write exists");
    fs::write(
        dir.join("script.shelly"),
        r#"
copy "source.txt" "out/file.txt"
copy "srcdir" "out/dircopy"
read "out/file.txt" -> $file
read "out/dircopy/sub/item.txt" -> $dir
shellout "$file/$dir"
"#,
    )
    .expect("write script");

    let output = run_script_path(dir.join("script.shelly"));
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "file/dir\n");

    fs::write(
        dir.join("fail.shelly"),
        "copy \"source.txt\" \"exists.txt\"\n",
    )
    .expect("write fail script");
    let output = run_script_path(dir.join("fail.shelly"));
    assert!(!output.status.success());
    assert!(stderr(&output).contains("destination already exists"));
}

#[test]
fn file_commands_move_rename_and_remove_behaviors() {
    let dir = test_dir();
    fs::write(dir.join("move.txt"), "moved").expect("write move");
    fs::create_dir_all(dir.join("tree/child")).expect("create tree");
    fs::write(dir.join("tree/child/item.txt"), "item").expect("write item");
    fs::write(dir.join("rename.txt"), "renamed").expect("write rename");
    fs::write(dir.join("remove.txt"), "remove").expect("write remove");
    fs::create_dir_all(dir.join("remove-dir")).expect("create remove dir");
    fs::write(dir.join("remove-dir/item.txt"), "x").expect("write remove dir item");
    fs::write(
        dir.join("script.shelly"),
        r#"
move "move.txt" "dest/move.txt"
move "tree" "dest/tree"
rename "rename.txt" "renamed.txt"
remove "remove.txt"
remove "missing.txt"
remove "remove-dir/"
read "dest/move.txt" -> $moved
read "dest/tree/child/item.txt" -> $item
read "renamed.txt" -> $renamed
shellout "$moved/$item/$renamed"
"#,
    )
    .expect("write script");

    let output = run_script_path(dir.join("script.shelly"));
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "moved/item/renamed\n");
    assert!(!dir.join("remove.txt").exists());
    assert!(!dir.join("remove-dir").exists());

    fs::create_dir_all(dir.join("must-trail")).expect("create must trail");
    fs::write(dir.join("bad-remove.shelly"), "remove \"must-trail\"\n").expect("write bad remove");
    let output = run_script_path(dir.join("bad-remove.shelly"));
    assert!(!output.status.success());
    assert!(stderr(&output).contains("must end with '/'"));

    fs::write(
        dir.join("bad-rename.shelly"),
        "rename \"renamed.txt\" \"nested/name.txt\"\n",
    )
    .expect("write bad rename");
    let output = run_script_path(dir.join("bad-rename.shelly"));
    assert!(!output.status.success());
    assert!(stderr(&output).contains("plain name"));
}

#[test]
fn file_commands_read_write_errors_and_check_validation() {
    let dir = test_dir();
    fs::create_dir_all(dir.join("adir")).expect("create dir");
    fs::write(
        dir.join("script.shelly"),
        r#"
$text = "hello"
write "./out/text.txt" $text
read "out/text.txt" -> $read
shellout $read
"#,
    )
    .expect("write script");
    let output = run_script_path(dir.join("script.shelly"));
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "hello\n");

    for (source, expected) in [
        ("read \"missing.txt\" -> $text\n", "could not read file"),
        ("read \"adir\" -> $text\n", "is a directory"),
        ("write \"adir\" \"x\"\n", "is a directory"),
    ] {
        fs::write(dir.join("fail.shelly"), source).expect("write fail script");
        let output = run_script_path(dir.join("fail.shelly"));
        assert!(!output.status.success(), "source succeeded: {source}");
        assert!(
            stderr(&output).contains(expected),
            "stderr: {}",
            stderr(&output)
        );
    }

    let valid = check_script(
        r#"
create "a.txt"
copy "a.txt" "b.txt"
move "b.txt" "c.txt"
rename "c.txt" "d.txt"
remove "d.txt"
read "d.txt" -> $text
write "e.txt" $text
"#,
    );
    assert!(valid.status.success(), "stderr: {}", stderr(&valid));

    for (source, expected) in [
        ("create a.txt\n", "create requires a string path"),
        ("copy \"a\"\n", "copy syntax"),
        ("move \"a\" $to\n", "move syntax"),
        ("rename \"a\" \"dir/name\"\n", "plain name"),
        (
            "read \"a\" -> $TEXT\n",
            "read destination must be a variable",
        ),
        ("write \"a\"\n", "write syntax"),
    ] {
        let output = check_script(source);
        assert!(!output.status.success(), "source should fail: {source}");
        assert!(
            stderr(&output).contains(expected),
            "stderr: {}",
            stderr(&output)
        );
    }
}

#[test]
fn file_helper_commands_check_file_dir_temp_size_and_append() {
    let dir = test_dir();
    fs::create_dir_all(dir.join("existing-dir")).expect("create dir");
    fs::write(dir.join("existing.txt"), "hello").expect("write file");
    fs::write(
        dir.join("script.shelly"),
        r#"
isFile "existing.txt" -> $file_true
isFile "missing.txt" -> $file_false
isDir "existing-dir" -> $dir_true
isDir "existing.txt" -> $dir_false
size "existing.txt" -> $bytes
tempFile -> $temp_file
tempDir -> $temp_dir
write $temp_file "alpha"
append $temp_file "beta"
read $temp_file -> $temp_text
size $temp_file -> $temp_bytes
isFile $temp_file -> $temp_file_ok
isDir $temp_dir -> $temp_dir_ok
remove $temp_file
isFile $temp_file -> $temp_file_removed
append "logs/out.txt" "first"
append "logs/out.txt" "second"
read "logs/out.txt" -> $log
shellout "$file_true/$file_false/$dir_true/$dir_false/$bytes/$temp_file_ok/$temp_dir_ok/$temp_bytes/$temp_file_removed"
shellout $temp_text
shellout $log
"#,
    )
    .expect("write script");

    let output = run_script_path(dir.join("script.shelly"));

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "true/false/true/false/5/true/true/10/false\nalphabeta\n\nfirst\nsecond\n\n"
    );
    assert_eq!(
        fs::read_to_string(dir.join("logs/out.txt")).expect("read appended file"),
        "first\nsecond\n"
    );
}

#[test]
fn size_fails_for_directory_and_append_preserves_list_append() {
    let dir = test_dir();
    fs::create_dir_all(dir.join("adir")).expect("create dir");
    fs::write(dir.join("size-dir.shelly"), "size \"adir\" -> $bytes\n").expect("write script");

    let output = run_script_path(dir.join("size-dir.shelly"));
    assert!(!output.status.success());
    assert!(stderr(&output).contains("size path is not a file"));

    let output = run_script(
        r#"
$items = ["one"]
append $items "two"
shellout $items
"#,
    );
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "[one, two]\n");
}

#[test]
fn check_accepts_file_helpers_and_rejects_invalid_file_helper_syntax() {
    let valid = check_script(
        r#"
isFile "a.txt" -> $ok
isDir "dir" -> $ok
tempFile -> $path
tempDir -> $path
size "a.txt" -> $bytes
append "logs/out.txt" "entry"
write $path "entry"
append $path "entry"
read $path -> $text
size $path -> $bytes
isFile $path -> $ok
remove $path
"#,
    );
    assert!(valid.status.success(), "stderr: {}", stderr(&valid));

    for (source, expected) in [
        ("isFile \"a\"\n", "isFile syntax"),
        ("isDir a -> $ok\n", "isDir requires a string path"),
        (
            "isFile \"a\" -> $OK\n",
            "isFile destination must be a variable",
        ),
        ("tempFile\n", "tempFile syntax"),
        ("tempDir -> path\n", "temp destination must be a variable"),
        ("size \"a\"\n", "size syntax"),
        ("append \"a\"\n", "append syntax"),
    ] {
        let output = check_script(source);
        assert!(!output.status.success(), "source should fail: {source}");
        assert!(
            stderr(&output).contains(expected),
            "stderr did not contain {expected:?}: {}",
            stderr(&output)
        );
    }
}

#[test]
fn import_and_include_support_explicit_current_directory_paths() {
    let dir = test_dir();
    fs::write(dir.join("module.shelly"), "export $VALUE = \"imported\"\n").expect("write module");
    fs::write(dir.join("shared.shelly"), "$shared = \"included\"\n").expect("write shared");
    fs::write(
        dir.join("script.shelly"),
        "import \"./module.shelly\"\ninclude \"./shared.shelly\"\nshellout \"$VALUE/$shared\"\n",
    )
    .expect("write script");

    let output = run_script_path(dir.join("script.shelly"));

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "imported/included\n");
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
            "repeat 3 -> $I\n    shellout \"no\"\nend\n",
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
$BASE = 2
exit $BASE + $extra
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
fn stop() {
    shellout "function"
    exit 0
}
call stop()
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
$AGE = 37
$refs = [$name, $AGE, false]
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
append $names "Mora"
prepend $names "Iris"
$names[2] = "Ada"
remove $names[last]
remove $names[first]
shellout $names[..]
$empty = []
append $empty "tail"
prepend $empty "head"
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
$AGE = 37
$items = []
append $items "Ana"
append $items 2 + 3
append $items true
append $items $value
prepend $items $AGE
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
            "$NAMES = [\"Ana\"]\nappend $NAMES \"Luis\"\n",
            "collection path requires a nested segment",
        ),
        ("append $missing \"Luis\"\n", "undefined variable $missing"),
        (
            "$name = \"Ana\"\nappend $name \"Luis\"\n",
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
            "$names = [\"Ana\"]\nappend $names [\"nested\"]\n",
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
fn label($name) {
    return "hello $name"
    shellout "unreachable"
}
call label("Ada") -> $message
shellout $message
call label("Bea")
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "hello Ada\n");
}

#[test]
fn parenthesized_conditions_override_precedence_and_nest() {
    let output = run_script(
        r#"
$items = ["x"]
$force = false
if (true or false) and false
    shellout "bad-precedence"
else
    shellout "group-false"
end
if true and (false or true)
    shellout "group-true"
end
if ((false or true) and (not false))
    shellout "nested"
end
if not (true and false)
    shellout "not-group"
end
if (exists "script.shelly" and contains $items "x") or $force
    shellout "predicates"
end
if false
    shellout "bad-if"
elif (true and (false or true))
    shellout "elif-group"
end
$count = 0
while ($count < 2)
    shellout "while $count"
    $count = $count + 1
end
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "group-false
group-true
nested
not-group
predicates
elif-group
while 0
while 1
"
    );
}

#[test]
fn check_accepts_grouped_conditions_and_rejects_malformed_groups() {
    let valid = check_script(
        r#"
$items = ["x"]
$force = false
if (exists "README.md" and contains $items "x") or $force
    shellout "ok"
elif not (false)
    shellout "elif"
end
while (false or $force)
    break
end
"#,
    );
    assert!(valid.status.success(), "stderr: {}", stderr(&valid));

    for (source, expected) in [
        (
            r#"if (true
    shellout "bad"
end
"#,
            "missing ')'",
        ),
        (
            r#"if true)
    shellout "bad"
end
"#,
            "unexpected ')'",
        ),
        (
            r#"if ()
    shellout "bad"
end
"#,
            "empty parenthesized condition",
        ),
        (
            "(true)
",
            "unknown command or invalid syntax",
        ),
        (
            r#"shellout ("hi")
"#,
            "expected string",
        ),
    ] {
        let output = check_script(source);
        assert!(!output.status.success(), "source should fail: {source}");
        assert!(
            stderr(&output).contains(expected),
            "stderr did not contain {expected:?}: {}",
            stderr(&output)
        );
    }
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

#[test]
fn exists_predicate_supports_conditions_capture_and_missing_paths() {
    let dir = test_dir();
    fs::write(dir.join("present.txt"), "ok").expect("write present file");
    fs::write(dir.join("has->arrow.txt"), "ok").expect("write arrow file");
    let script = dir.join("script.shelly");
    fs::write(
        &script,
        r#"
if exists "present.txt"
    shellout "present"
end
if not exists "missing.txt"
    shellout "missing-ok"
end
exists "present.txt" -> $haspresent
exists "missing.txt" -> $hasmissing
exists "has->arrow.txt" -> $hasarrow
shellout "$haspresent/$hasmissing/$hasarrow"
"#,
    )
    .expect("write script");

    let output = run_script_path(script);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "present\nmissing-ok\ntrue/false/true\n");
}

#[test]
fn contains_predicate_supports_string_list_logic_and_capture() {
    let output = run_script(
        r#"
$name = "augustus"
$names = ["Ana", "Luis"]
if contains $name "gus" and contains $names "Luis"
    shellout "contains"
end
if not contains $names "Mia" or false
    shellout "not-mia"
end
contains $names "Luis" -> $hasluis
contains $name "gus" -> $hasgus
shellout "$hasluis/$hasgus"
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "contains\nnot-mia\ntrue/true\n");
}

#[test]
fn check_accepts_predicates_and_rejects_malformed_predicates() {
    let valid = check_script(
        r#"
$names = ["Luis"]
if exists "README.md" and contains $names "Luis"
    shellout "ok"
end
exists "README.md" -> $hasreadme
contains $names "Luis" -> $hasluis
"#,
    );
    assert!(valid.status.success(), "stderr: {}", stderr(&valid));

    for source in [
        "if exists\n    shellout \"bad\"\nend\n",
        "if contains $names\n    shellout \"bad\"\nend\n",
        "exists README.md -> $hasreadme\n",
        "contains $names -> $hasluis\n",
    ] {
        let output = check_script(source);
        assert!(!output.status.success(), "source should fail: {source}");
    }
}

#[test]
fn try_success_skips_catch_and_errors_enter_catch() {
    let output = run_script(
        r#"
try
    shellout "ok"
catch
    shellout "bad"
end
try
    nope
catch
    shellout "caught"
end
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "ok\ncaught\n");
}

#[test]
fn try_captures_structured_runtime_error_and_nested_try_works() {
    let output = run_script(
        r#"
try -> $error
    unknown-command
catch
    shellout "kind=$error.kind message=$error.message"
end
try
    try
        bad-inner
    catch
        shellout "inner"
    end
    shellout "outer ok"
catch
    shellout "outer catch"
end
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let out = stdout(&output);
    assert!(
        out.contains(
            "kind=runtime message=line 3: unknown command or invalid syntax: unknown-command"
        ),
        "stdout: {out}"
    );
    assert!(out.contains("inner\nouter ok\n"), "stdout: {out}");
}

#[test]
fn try_captures_failed_command_as_process_error_with_metadata() {
    let output = run_script(
        r#"
try -> $error
    command sh -c "printf out; printf err >&2; exit 7" -> $captured
catch
    shellout "kind=$error.kind program=$error.program args=$error.args code=$error.code stdout=$error.stdout stderr=$error.stderr message=$error.message"
end
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "kind=process program=sh args=[-c, printf out; printf err >&2; exit 7] code=7 stdout=out stderr=err message=command exited with status 7\n"
    );
}

#[test]
fn try_does_not_catch_exit_break_continue_or_return() {
    let output = run_script("try\n    exit 7\ncatch\n    shellout \"caught\"\nend\n");
    assert_eq!(output.status.code(), Some(7));
    assert_eq!(stdout(&output), "");

    for (source, expected) in [
        (
            "try\n    break\ncatch\n    shellout \"caught\"\nend\n",
            "break outside loop",
        ),
        (
            "try\n    continue\ncatch\n    shellout \"caught\"\nend\n",
            "continue outside loop",
        ),
    ] {
        let output = run_script(source);
        assert!(!output.status.success(), "source succeeded: {source}");
        assert!(
            stderr(&output).contains(expected),
            "stderr: {}",
            stderr(&output)
        );
        assert!(
            !stdout(&output).contains("caught"),
            "stdout: {}",
            stdout(&output)
        );
    }
}

#[test]
fn check_validates_try_catch_structure() {
    let valid = check_script("try -> $error\n    command false\ncatch\n    shellout $error\nend\n");
    assert!(valid.status.success(), "stderr: {}", stderr(&valid));

    for (source, expected) in [
        ("try\n    shellout \"x\"\nend\n", "missing catch"),
        ("try\ncatch\ncatch\nend\n", "duplicate catch"),
        ("catch\n", "catch outside try"),
        (
            "try -> error\ncatch\nend\n",
            "try error destination must be a variable",
        ),
        (
            "try -> $ERROR\ncatch\nend\n",
            "try error destination must be a variable",
        ),
    ] {
        let output = check_script(source);
        assert!(!output.status.success(), "source should fail: {source}");
        assert!(
            stderr(&output).contains(expected),
            "stderr did not contain {expected:?}: {}",
            stderr(&output)
        );
    }
}

#[test]
fn log_block_writes_structured_file_and_keeps_stdout() {
    let dir = test_dir();
    fs::write(
        dir.join("script.shelly"),
        r#"
log "run"
    shellout "visible"
    command printf cmd-out
end
"#,
    )
    .expect("write script");

    let output = run_script_path(dir.join("script.shelly"));

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "visible\ncmd-out");
    let log = read_single_log(&dir, "run-");
    assert!(log.contains("--- run ---"), "log: {log}");
    assert!(log.contains("ENTER"), "log: {log}");
    assert!(log.contains("shellout: visible"), "log: {log}");
    assert!(log.contains("command: printf [\"cmd-out\"]"), "log: {log}");
    assert!(log.contains("EXIT ok"), "log: {log}");
}

#[test]
fn log_block_records_errors_and_preserves_failure() {
    let dir = test_dir();
    fs::write(dir.join("script.shelly"), "log \"err\"\n    unknown\nend\n").expect("write script");

    let output = run_script_path(dir.join("script.shelly"));

    assert!(!output.status.success());
    assert!(stderr(&output).contains("unknown command"));
    let log = read_single_log(&dir, "err-");
    assert!(log.contains("ERROR"), "log: {log}");
    assert!(log.contains("unknown command"), "log: {log}");
}

#[test]
fn log_identifier_is_sanitized_and_nested_blocks_collect() {
    let dir = test_dir();
    fs::write(
        dir.join("script.shelly"),
        r#"
log "bad/id!*"
    log "inner"
        shellout "inner out"
    end
end
"#,
    )
    .expect("write script");

    let output = run_script_path(dir.join("script.shelly"));

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "inner out\n");
    let outer = read_single_log(&dir, "bad-id---");
    assert!(outer.contains("log: inner"), "log: {outer}");
    assert!(outer.contains("EXIT ok"), "log: {outer}");
    let inner = read_single_log(&dir, "inner-");
    assert!(inner.contains("shellout: inner out"), "log: {inner}");
}

#[test]
fn check_accepts_log_blocks_and_rejects_missing_identifier_or_end() {
    let valid = check_script("log \"id\"\n    shellout \"ok\"\nend\n");
    assert!(valid.status.success(), "stderr: {}", stderr(&valid));

    let missing_id = check_script("log\n    shellout \"bad\"\nend\n");
    assert!(!missing_id.status.success());
    assert!(stderr(&missing_id).contains("log identifier required"));

    let missing_end = check_script("log \"id\"\n    shellout \"bad\"\n");
    assert!(!missing_end.status.success());
    assert!(stderr(&missing_end).contains("missing end for log"));
}

#[test]
fn script_arguments_support_positionals_flags_and_iteration() {
    let script = write_script(
        r#"
arg 1 -> $name
arg 3 -> $missing
hasArg "--dry-run" -> $dryrun
hasArg "--force" -> $force
args -> $all
shellout "name=$name missing=<$missing> dry=$dryrun force=$force"
for $item in $all
    shellout "arg:$item"
end
"#,
    );

    let output = run_script_path_with_args(script, &["Ada", "--dry-run"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "name=Ada missing=<> dry=true force=false\narg:Ada\narg:--dry-run\n"
    );
}

#[test]
fn script_argument_index_can_use_numeric_expression_or_variable() {
    let output = run_script_with_args(
        r#"
$which = 1 + 1
arg $which -> $value
hasArg $value -> $found
shellout "$value/$found"
"#,
        &["first", "second"],
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "second/true\n");
}

#[test]
fn check_accepts_script_argument_syntax_and_rejects_bad_forms() {
    let valid = check_script(
        r#"
arg 1 -> $name
hasArg "--dry-run" -> $dryrun
args -> $all
"#,
    );
    assert!(valid.status.success(), "stderr: {}", stderr(&valid));

    for (source, expected) in [
        ("arg 0 -> $name\n", "arg index must be a positive integer"),
        ("arg -1 -> $name\n", "arg index must be a positive integer"),
        (
            "arg \"one\" -> $name\n",
            "arg index must be a positive integer",
        ),
        ("arg 1\n", "arg syntax is arg N -> $var"),
        ("arg 1 -> $NAME\n", "arg destination must be a variable"),
        ("arg 1 -> name\n", "arg destination must be a variable"),
        (
            "arg 1 -> $name extra\n",
            "arg destination must be a variable",
        ),
        ("hasArg -> $found\n", "hasArg value required"),
        (
            "hasArg \"x\" -> $FOUND\n",
            "hasArg destination must be a variable",
        ),
        ("args\n", "args syntax is args -> $var"),
        ("args -> $ALL\n", "args destination must be a variable"),
    ] {
        let output = check_script(source);
        assert!(!output.status.success(), "source should fail: {source}");
        assert!(
            stderr(&output).contains(expected),
            "stderr did not contain {expected:?}: {}",
            stderr(&output)
        );
    }
}

#[test]
fn runtime_rejects_script_argument_index_expressions_that_are_not_positive_integers() {
    for (source, expected) in [
        (
            "$which = 0\narg $which -> $value\n",
            "arg index must be a positive integer",
        ),
        (
            "arg 1.5 -> $value\n",
            "arg index must be a positive integer",
        ),
        (
            "arg missing -> $value\n",
            "arg index must be a positive integer",
        ),
    ] {
        let output = run_script(source);
        assert!(!output.status.success(), "source succeeded: {source}");
        assert!(stderr(&output).contains("line"));
        assert!(
            stderr(&output).contains(expected),
            "stderr: {}",
            stderr(&output)
        );
    }
}

#[test]
fn regex_literals_do_not_confuse_condition_scanning() {
    let output = run_script(
        r#"
$name = "aandb(or)=x/y"
if matches $name /and|or|\(|\)|==|\//
    shellout "matched"
else
    shellout "missed"
end
"#,
    );
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "matched\n");
}

#[test]
fn regex_predicate_supports_runtime_capture_composition_and_checking() {
    let output = run_script(
        r#"
$name = "abc123"
matches $name /^[a-z]+\d+$/ -> $valid
assertTrue $valid
if matches $name /^[a-z]+\d+$/ and not matches $name /xyz/
    shellout "composed"
end
if matches $name /abc\/123/
    shellout "slash"
end
"#,
    );
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "composed\n");

    let valid = check_script("if matches \"abc123\" /^[a-z]+\\d+$/\nend\n");
    assert!(valid.status.success(), "stderr: {}", stderr(&valid));
    let invalid = run_script("if matches \"abc\" /[/\nend\n");
    assert!(!invalid.status.success());
    let invalid_check = check_script("if matches \"abc\" /unterminated\nend\n");
    assert!(!invalid_check.status.success());
}

fn run_script(source: &str) -> std::process::Output {
    run_script_with_args(source, &[])
}

fn run_script_with_args(source: &str, args: &[&str]) -> std::process::Output {
    let script = write_script(source);
    run_script_path_with_args(script, args)
}

fn run_script_with_stdin(source: &str, input: &[u8]) -> std::process::Output {
    let script = write_script(source);
    let mut child = Command::new(env!("CARGO_BIN_EXE_shelly"))
        .arg(script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn shelly");

    child
        .stdin
        .as_mut()
        .expect("stdin is piped")
        .write_all(input)
        .expect("write stdin");
    child.wait_with_output().expect("wait for shelly")
}

fn check_script(source: &str) -> std::process::Output {
    let script = write_script(source);
    Command::new(env!("CARGO_BIN_EXE_shelly"))
        .args(["check"])
        .arg(script)
        .output()
        .expect("run shelly check")
}

#[test]
fn test_requires_manifest() {
    let dir = test_dir();
    let output = run_shelly_test(&dir);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("Shelly.toml"));
}

#[test]
fn test_accepts_custom_test_directory_and_pattern() {
    let dir = test_dir();
    write_manifest_with(
        &dir,
        r#"
[test]
directory = "specs"
pattern = "*_spec.shelly"
"#,
    );
    write_test(&dir, "specs/kept_spec.shelly", "shellout \"kept\\n\"\n");
    write_test(&dir, "specs/ignored.shelly", "exit 1\n");

    let output = run_shelly_test(&dir);
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).contains("PASS specs/kept_spec.shelly"));
    assert!(!stdout(&output).contains("ignored.shelly"));
}

#[test]
fn test_uses_default_test_configuration_when_test_section_is_omitted() {
    let dir = test_dir();
    write_manifest(&dir);
    write_test(
        &dir,
        "tests/default_test.shelly",
        "shellout \"default\\n\"\n",
    );

    let output = run_shelly_test(&dir);
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).contains("PASS tests/default_test.shelly"));
}

#[test]
fn test_fills_missing_test_configuration_fields_with_defaults() {
    let dir = test_dir();
    write_manifest_with(&dir, "\n[test]\ndirectory = \"custom-tests\"\n");
    write_test(
        &dir,
        "custom-tests/partial_test.shelly",
        "shellout \"partial\\n\"\n",
    );

    let output = run_shelly_test(&dir);
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).contains("PASS custom-tests/partial_test.shelly"));
}

#[test]
fn test_rejects_missing_or_invalid_required_project_fields() {
    for (manifest, field) in [
        (
            "[project]\nversion = \"0.1.0\"\nentry = \"main.shelly\"\n",
            "name",
        ),
        (
            "[project]\nname = \"test\"\nentry = \"main.shelly\"\n",
            "version",
        ),
        ("[project]\nname = \"test\"\nversion = \"0.1.0\"\n", "entry"),
        (
            "[project]\nname = \"\"\nversion = \"0.1.0\"\nentry = \"main.shelly\"\n",
            "name",
        ),
        (
            "[project]\nname = \"test\"\nversion = \"\"\nentry = \"main.shelly\"\n",
            "version",
        ),
        (
            "[project]\nname = \"test\"\nversion = \"0.1.0\"\nentry = \"\"\n",
            "entry",
        ),
    ] {
        let dir = test_dir();
        fs::write(dir.join("Shelly.toml"), manifest).expect("write manifest");
        fs::create_dir(dir.join("tests")).expect("create tests directory");
        write_test(
            &dir,
            "tests/placeholder_test.shelly",
            "shellout \"placeholder\\n\"\n",
        );
        let output = run_shelly_test(&dir);
        assert!(!output.status.success(), "accepted invalid {field}");
        assert!(
            stderr(&output).contains(field),
            "stderr: {}",
            stderr(&output)
        );
    }
}

#[test]
fn test_fails_when_tests_directory_is_missing_or_empty() {
    let dir = test_dir();
    write_manifest(&dir);
    let missing = run_shelly_test(&dir);
    assert!(!missing.status.success());
    assert!(stderr(&missing).contains("tests"));

    fs::create_dir(dir.join("tests")).expect("create tests directory");
    let empty = run_shelly_test(&dir);
    assert!(!empty.status.success());
    assert!(stderr(&empty).contains("tests"));
}

#[test]
fn test_recursively_discovers_sorted_test_files_and_reports_relative_paths() {
    let dir = test_dir();
    write_manifest(&dir);
    write_test(&dir, "tests/z_test.shelly", "shellout \"z\\n\"\n");
    write_test(&dir, "tests/nested/a_test.shelly", "shellout \"a\\n\"\n");
    write_test(&dir, "tests/ignored.shelly", "exit 1\n");

    let output = run_shelly_test(&dir);
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let out = stdout(&output);
    assert!(
        out.find("PASS tests/nested/a_test.shelly").unwrap()
            < out.find("PASS tests/z_test.shelly").unwrap()
    );
    assert!(!out.contains("ignored.shelly"));
}

#[test]
fn test_reports_pass_fail_continues_and_aggregates_failure() {
    let dir = test_dir();
    write_manifest(&dir);
    write_test(
        &dir,
        "tests/01_pass_test.shelly",
        "shellout \"before\\n\"\n",
    );
    write_test(&dir, "tests/02_fail_test.shelly", "exit 3\n");
    write_test(
        &dir,
        "tests/03_after_test.shelly",
        "shellout \"after\\n\"\n",
    );

    let output = run_shelly_test(&dir);
    assert!(!output.status.success());
    let out = stdout(&output);
    assert!(out.contains("PASS tests/01_pass_test.shelly"));
    assert!(out.contains("FAIL tests/02_fail_test.shelly"));
    assert!(out.contains("PASS tests/03_after_test.shelly"));
    assert!(out.contains("passed") && out.contains("failed"));
}

#[test]
fn test_succeeds_when_all_discovered_tests_pass() {
    let dir = test_dir();
    write_manifest(&dir);
    write_test(&dir, "tests/one_test.shelly", "shellout \"one\\n\"\n");
    write_test(&dir, "tests/two_test.shelly", "shellout \"two\\n\"\n");

    let output = run_shelly_test(&dir);
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).contains("passed"));
}

#[test]
fn test_runs_each_file_in_an_isolated_shelly_process() {
    let dir = test_dir();
    write_manifest(&dir);
    write_test(&dir, "tests/first_test.shelly", "$value = \"first\"\n");
    write_test(&dir, "tests/second_test.shelly", "shellout $value\n");

    let output = run_shelly_test(&dir);
    assert!(!output.status.success());
    assert!(stdout(&output).contains("PASS tests/first_test.shelly"));
    assert!(stdout(&output).contains("FAIL tests/second_test.shelly"));
}

#[test]
fn test_assertions_accept_successful_values_variables_and_collections() {
    let dir = test_dir();
    write_manifest(&dir);
    write_test(
        &dir,
        "tests/assertions_test.shelly",
        r#"
$value = "ready"
$items = ["one", "two"]
assertEqual $value "ready"
assertTrue true
assertFalse false
assertContains $items "two"
"#,
    );

    let output = run_shelly_test(&dir);
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).contains("PASS tests/assertions_test.shelly"));
}

#[test]
fn test_assertions_report_expected_actual_and_stop_later_code() {
    let dir = test_dir();
    write_manifest(&dir);
    write_test(
        &dir,
        "tests/assertions_test.shelly",
        r#"
assertEqual "actual" "expected"
shellout "MUST_NOT_RUN"
"#,
    );

    let output = run_shelly_test(&dir);
    assert!(!output.status.success());
    let combined = format!("{}{}", stdout(&output), stderr(&output));
    assert!(combined.contains("expected"));
    assert!(combined.contains("actual"));
    assert!(!combined.contains("MUST_NOT_RUN"));
}

#[test]
fn test_assert_false_reports_failure() {
    let dir = test_dir();
    write_manifest(&dir);
    write_test(&dir, "tests/assertions_test.shelly", "assertFalse true\n");

    let output = run_shelly_test(&dir);
    assert!(!output.status.success());
    let combined = format!("{}{}", stdout(&output), stderr(&output));
    assert!(combined.contains("assertFalse") || combined.contains("false"));
}

#[test]
fn test_assert_contains_reports_failure() {
    let dir = test_dir();
    write_manifest(&dir);
    write_test(
        &dir,
        "tests/assertions_test.shelly",
        "assertContains [\"one\"] \"missing\"\n",
    );

    let output = run_shelly_test(&dir);
    assert!(!output.status.success());
    let combined = format!("{}{}", stdout(&output), stderr(&output));
    assert!(combined.contains("missing"));
}

#[test]
fn check_rejects_assertions_with_wrong_argument_counts() {
    for source in [
        "assertEqual 1\n",
        "assertTrue true extra\n",
        "assertFalse\n",
        "assertContains [1] \"x\" extra\n",
    ] {
        let output = check_script(source);
        assert!(
            !output.status.success(),
            "accepted invalid assertion: {source:?}"
        );
    }
}

#[test]
fn test_command_discovers_a_failing_assertion() {
    let dir = test_dir();
    write_manifest(&dir);
    write_test(&dir, "tests/assertion_test.shelly", "assertFalse true\n");

    let output = run_shelly_test(&dir);
    assert!(!output.status.success());
    assert!(stdout(&output).contains("FAIL tests/assertion_test.shelly"));
}

#[test]
fn run_executes_manifest_entry_from_current_directory() {
    let dir = test_dir();
    write_manifest_with_entry(&dir, "main.shelly");
    fs::write(dir.join("main.shelly"), "shellout \"from-entry\\n\"\n").unwrap();

    let output = run_shelly_run(&dir, &[]);
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "from-entry\n\n");
}

#[test]
fn run_forwards_all_arguments_after_double_dash_unchanged() {
    let dir = test_dir();
    write_manifest_with_entry(&dir, "main.shelly");
    fs::write(
        dir.join("main.shelly"),
        "args -> $all\nfor $item in $all\n    shellout $item\nend\n",
    )
    .unwrap();

    let output = run_shelly_run(&dir, &["--", "--flag", "value with spaces", "-"]);
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "--flag\nvalue with spaces\n-\n");
}

#[test]
fn check_without_path_validates_configured_entry_without_executing_it() {
    let dir = test_dir();
    write_manifest_with_entry(&dir, "main.shelly");
    fs::write(
        dir.join("main.shelly"),
        "shellout \"MUST_NOT_EXECUTE\"\nmissing_instruction\n",
    )
    .unwrap();

    let output = run_shelly_check(&dir, &[]);
    assert!(!output.status.success());
    assert_eq!(stdout(&output), "");
    assert!(stderr(&output).contains("line 2"));
    assert!(stderr(&output).contains("missing_instruction"));
}

#[test]
fn check_without_path_emits_configured_entry_diagnostics_as_json() {
    let dir = test_dir();
    write_manifest_with_entry(&dir, "main.shelly");
    fs::write(dir.join("main.shelly"), "break\n").unwrap();

    let output = run_shelly_check(&dir, &["--format", "json"]);
    assert!(!output.status.success());
    assert_eq!(stderr(&output), "");
    let diagnostics = stdout(&output);
    assert!(diagnostics.starts_with('['));
    assert!(diagnostics.contains("\"file\""));
    assert!(diagnostics.contains("\"line\":1"));
    assert!(diagnostics.contains("break outside loop"));
}

#[test]
fn check_without_path_rejects_missing_or_invalid_manifest_entry() {
    let missing = test_dir();
    let output = run_shelly_check(&missing, &[]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("Shelly.toml"));

    let cases = [
        ("[project]\nname = \"test\"\nversion = \"0.1.0\"\n", "entry"),
        ("not valid toml", "manifest"),
        (
            "[project]\nname = \"test\"\nversion = \"0.1.0\"\nentry = \"missing.shelly\"\n",
            "missing.shelly",
        ),
    ];

    for (manifest, expected) in cases {
        let dir = test_dir();
        fs::write(dir.join("Shelly.toml"), manifest).unwrap();
        let output = run_shelly_check(&dir, &[]);
        assert!(!output.status.success(), "accepted manifest: {manifest:?}");
        assert!(
            stderr(&output).contains(expected),
            "stderr: {}",
            stderr(&output)
        );
    }
}

#[test]
fn check_explicit_path_remains_compatible_with_configured_projects() {
    let dir = test_dir();
    write_manifest_with_entry(&dir, "main.shelly");
    fs::write(dir.join("main.shelly"), "break\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_shelly"))
        .args(["check", "main.shelly"])
        .current_dir(&dir)
        .output()
        .expect("run shelly check");
    assert!(!output.status.success());
    assert!(stderr(&output).contains("break outside loop"));
}

#[test]
fn run_requires_a_manifest() {
    let dir = test_dir();
    let output = run_shelly_run(&dir, &[]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("Shelly.toml"));
}

#[test]
fn run_rejects_invalid_or_missing_manifest_entry() {
    for entry in ["", "missing.shelly"] {
        let dir = test_dir();
        write_manifest_with_entry(&dir, entry);
        if entry.is_empty() {
            fs::write(
                dir.join("Shelly.toml"),
                "[project]\nname=\"test\"\nversion=\"0.1.0\"\n",
            )
            .unwrap();
        }
        let output = run_shelly_run(&dir, &[]);
        assert!(!output.status.success(), "accepted entry {entry:?}");
        assert!(
            stderr(&output).contains("entry"),
            "stderr: {}",
            stderr(&output)
        );
    }
}

#[test]
fn run_rejects_manifest_entry_file_that_is_missing() {
    let dir = test_dir();
    write_manifest_with_entry(&dir, "main.shelly");
    let output = run_shelly_run(&dir, &[]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("main.shelly"));
}

#[test]
fn direct_script_execution_remains_unchanged() {
    let output = run_script("shellout \"direct\\n\"\n");
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "direct\n\n");
}

fn run_shelly_check(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_shelly"))
        .arg("check")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("run shelly check")
}

fn run_shelly_run(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_shelly"))
        .arg("run")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("run shelly run")
}

fn write_manifest_with_entry(dir: &std::path::Path, entry: &str) {
    fs::write(
        dir.join("Shelly.toml"),
        format!("[project]\nname = \"test\"\nversion = \"0.1.0\"\nentry = \"{entry}\"\n"),
    )
    .unwrap();
}

fn run_shelly_test(dir: &std::path::Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_shelly"))
        .arg("test")
        .current_dir(dir)
        .output()
        .expect("run shelly test")
}

fn write_manifest(dir: &std::path::Path) {
    write_manifest_with(dir, "");
}

fn write_manifest_with(dir: &std::path::Path, extra: &str) {
    fs::write(
        dir.join("Shelly.toml"),
        format!(
            "[project]\\nname = \\\"test\\\"\\nversion = \\\"0.1.0\\\"\\nentry = \\\"main.shelly\\\"\\n{extra}"
        ),
    )
    .expect("write manifest");
}

fn write_test(dir: &std::path::Path, path: &str, source: &str) {
    let path = dir.join(path);
    fs::create_dir_all(path.parent().expect("test parent")).expect("create test parent");
    fs::write(path, source).expect("write test");
}

fn write_script(source: &str) -> std::path::PathBuf {
    let path = test_dir().join("script.shelly");
    fs::write(&path, source).expect("write script");
    path
}

fn run_script_path(path: std::path::PathBuf) -> std::process::Output {
    run_script_path_with_args(path, &[])
}

fn run_script_path_with_args(path: std::path::PathBuf, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_shelly"))
        .arg(path)
        .args(args)
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

fn read_single_log(dir: &std::path::Path, prefix: &str) -> String {
    let logs = dir.join("logs");
    let matches: Vec<_> = fs::read_dir(&logs)
        .expect("read logs dir")
        .map(|entry| entry.expect("log entry").path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(prefix) && name.ends_with(".log"))
        })
        .collect();
    assert_eq!(matches.len(), 1, "expected one log matching {prefix}");
    fs::read_to_string(&matches[0]).expect("read log")
}

#[test]
fn json_helpers_nested_lists_dictionaries_and_null() {
    let output = run_script(
        r#"
$text = "{\"users\":[{\"name\":\"Ada\",\"role\":\"admin\"},{\"name\":\"Luis\",\"role\":\"user\"}],\"matrix\":[[1,2],[3,4]],\"empty\":null}"
jsonParse $text -> $data
shellout $data.users[1].name
shellout $data.matrix[2][1]
shellout "null=$data.empty"
jsonStringify $data -> $json
jsonParse $json -> $again
shellout $again.users[2].role
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "Ada\n3\nnull=\nuser\n");
}

#[test]
fn dictionary_helpers_return_key_ordered_nested_values() {
    let output = run_script(
        r#"
$data = { b: [2, 1], a: { name: "Ada" } }
keys $data -> $keys
values $data -> $values
shellout $keys
shellout $values[1].name
shellout $values[2][1]
isDict $values[1] -> $ok
shellout $ok
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "[a, b]\nAda\n2\ntrue\n");
}

#[test]
fn nested_list_and_dictionary_access_interpolates() {
    let output = run_script(
        r#"
$users = [{ name: "Ada", role: "admin" }, { name: "Luis", role: "user" }]
$matrix = [[1, 2], [3, 4]]
shellout $users[1].name
shellout $matrix[2][1]
shellout "name=$users[2].name cell=$matrix[1][2]"
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "Ada\n3\nname=Luis cell=2\n");
}

#[test]
fn scalar_list_helpers_still_work_and_nested_limitations_are_clear() {
    let output = run_script(
        r#"
$items = ["b", "a", "b"]
sort $items -> $sorted
unique $sorted -> $unique
join $unique "," -> $joined
shellout $joined
$nested = [[2], [1]]
sort $nested -> $nope
"#,
    );

    assert!(!output.status.success());
    assert!(stdout(&output).contains("a,b"));
    assert!(stderr(&output).contains("sort requires a scalar list"));
}

#[test]
fn json_helper_checker_and_runtime_errors_are_clear() {
    let checked = check_script("jsonParse \"{}\" -> name\n");
    assert!(!checked.status.success());
    assert!(stderr(&checked).contains("jsonParse destination must be a variable"));

    let output = run_script("jsonParse \"{\" -> $value\n");
    assert!(!output.status.success());
    assert!(stderr(&output).contains("jsonParse failed"));
}

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout is valid UTF-8")
}

#[test]
fn text_and_date_helpers_honor_character_safe_padding_and_utc_contracts() {
    let output = run_script(
        r#"
substring "héllo" 2 3 -> $slice
padLeft "42" 5 "0" -> $left
padRight "42" 5 "." -> $right
padLeft "already-wide" 5 "0" -> $unchanged_left
padRight "already-wide" 5 "0" -> $unchanged_right
formatDate "2024-03-05T16:07:08Z" "YYYY-MM-DD HH:mm:ss" -> $date
shellout "$slice/$left/$right/$unchanged_left/$unchanged_right/$date"
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "éll/00042/42.../already-wide/already-wide/2024-03-05 16:07:08\n"
    );
}

#[test]
fn text_and_date_helpers_reject_invalid_inputs_and_arities() {
    for (source, expected) in [
        ("substring \"text\" 0 1 -> $value\n", "substring"),
        ("substring \"text\" 1 -> $value\n", "substring"),
        ("padLeft \"x\" 3 \"ab\" -> $value\n", "padleft"),
        ("padRight \"x\" 3 -> $value\n", "padright"),
        (
            "formatDate \"not-an-iso-date\" \"YYYY\" -> $value\n",
            "formatdate",
        ),
        (
            "formatDate \"2024-01-01T00:00:00Z\" \"QQ\" -> $value\n",
            "formatdate",
        ),
    ] {
        let output = run_script(source);
        assert!(!output.status.success(), "source succeeded: {source}");
        assert!(
            stderr(&output)
                .to_lowercase()
                .contains(&expected.to_lowercase()),
            "stderr: {}",
            stderr(&output)
        );
    }
}

#[test]
fn standard_helper_commands_cover_text_lists_types_paths_time_and_random() {
    let output = run_script(
        r#"
$text = "  Hello,hello,World  "
trim $text -> $trimmed
lower $trimmed -> $lowered
upper $lowered -> $uppered
replace $lowered "hello" "hi" -> $replaced
split $replaced "," -> $items
startsWith $trimmed "Hello" -> $starts
endsWith $trimmed "World" -> $ends
len $trimmed -> $text_len
len $items -> $list_len
join $items "|" -> $joined
sort $items -> $sorted
reverse $items -> $reversed
unique ["b", "a", "b"] -> $unique
$number_text = "007.50"
toNumber $number_text -> $number
$flag = true
toString $items -> $items_text
isEmpty "" -> $empty_text
isEmpty [] -> $empty_list
isNumber $number -> $is_number
isText "hello" -> $is_text
isList $items -> $is_list
pathJoin "dir" "nested" "archive.tar.gz" -> $joined_path
basename "dir/archive.tar.gz" -> $base
basename "dir/archive.tar.gz" ".tar.gz" -> $base_without_ext
dirname "dir/archive.tar.gz" -> $dir
extension "dir/archive.tar.gz" -> $ext
withoutExtension "dir/archive.tar.gz" -> $stem
now -> $timestamp
timestamp -> $timestamp_alias
today -> $date
commandExists "sh" -> $has_sh
commandExists "definitely-not-a-shelly-command" -> $has_missing
which "sh" -> $sh_path
random -> $random_value
randomInt 5 5 -> $random_int
shellout $trimmed
shellout $lowered
shellout $uppered
shellout $replaced
shellout $items
shellout "$starts/$ends/$text_len/$list_len/$joined"
shellout $sorted
shellout $reversed
shellout $unique
shellout "$number/$items_text/$empty_text/$empty_list/$is_number/$is_text/$is_list"
shellout "$joined_path/$base/$base_without_ext/$dir/$ext/$stem/$random_int"
shellout "$has_sh/$has_missing"
if $timestamp != ""
    shellout "timestamp-ok"
end
if $timestamp_alias != ""
    shellout "timestamp-alias-ok"
end
if $sh_path != ""
    shellout "which-ok"
end
if $date != ""
    shellout "date-ok"
end
if $random_value != ""
    shellout "random-ok"
end
"#,
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "Hello,hello,World\nhello,hello,world\nHELLO,HELLO,WORLD\nhi,hi,world\n[hi, hi, world]\ntrue/true/17/3/hi|hi|world\n[hi, hi, world]\n[world, hi, hi]\n[b, a]\n7.5/[hi, hi, world]/true/true/true/true/true\ndir/nested/archive.tar.gz/archive.tar.gz/archive/dir/gz/archive.tar/5\ntrue/false\ntimestamp-ok\ntimestamp-alias-ok\nwhich-ok\ndate-ok\nrandom-ok\n"
    );
}

#[test]
fn standard_helpers_check_valid_syntax_and_reject_representative_errors() {
    let valid = check_script(
        r#"
trim " x " -> $x
split "" "," -> $items
join $items "," -> $text
pathJoin "a" "b" -> $path
basename "a/b.txt" ".txt" -> $name
timestamp -> $timestamp
commandExists "sh" -> $has_sh
which "sh" -> $sh
now -> $now
randomInt 1 2 -> $n
"#,
    );
    assert!(valid.status.success(), "stderr: {}", stderr(&valid));

    for (source, expected) in [
        ("trim \"x\"\n", "trim syntax requires"),
        ("join \"x\" \",\" -> $out\n", "join requires a list"),
        ("toNumber \"no\" -> $n\n", "toNumber requires numeric input"),
        (
            "pathJoin \"only\" -> $path\n",
            "pathJoin expects at least 2 argument",
        ),
        (
            "basename \"a\" \"b\" \"c\" -> $name\n",
            "basename expects 1 or 2 argument",
        ),
        ("commandExists -> $ok\n", "commandExists expects 1 argument"),
        ("randomInt 9 1 -> $n\n", "min cannot be greater than max"),
        (
            "upper \"x\" -> $OUT\n",
            "upper destination must be a variable",
        ),
    ] {
        let output = if source.contains("join")
            || source.contains("toNumber")
            || source.contains("randomInt")
        {
            run_script(source)
        } else {
            check_script(source)
        };
        assert!(!output.status.success(), "source succeeded: {source}");
        assert!(
            stderr(&output).contains(expected),
            "stderr: {}",
            stderr(&output)
        );
    }
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr is valid UTF-8")
}
