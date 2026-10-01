#![forbid(unsafe_code)]
//! F2 -- CODEGEN-COMPILE-CHECK (gate-revision: 3)
//!
//! Scope: test-generation path in ec-codegen only.
//! Covered numeric: i32 n=1 n=5, f64 n=1, f32 n=2, pure i32 n=3, default f64 n=0.
//! Oracle independent: default sum n(n+1)/2 with distinct 1..n, pure factorial n!.
//! Ninth-param oracles document b1: default 1..9 expects 45, pure 1..9 expects 362880.
//! No-test-block expected for: bool char String str Vec-i32 n=1, todo i32 n=0,
//! mixed i32-f64, overflow pure u8 n=6. Also isize usize i128 u128 are out of scope.
//! Overflow rule for fix: checked on i128 vs T MIN MAX, f32 limit 2^24, f64 2^53,
//! pure f32 skipped at n>=11. Skipped cases are documented, not silent.
//! Mutant rule: anchor pub fn NAME( once, insert after first brace, verify once.
//! int uses let a = a + 1, float uses let a = a + 1.0, n0 f64 uses 0.0_f64 to 1.0_f64.
//! Kill means: compile ok, run exit nonzero, stdout+stderr contain panicked and assertion.
//! Mutant compile failure is gate bug, not kill. Oracle runs gate_oracle::check --exact
//! and must show test gate_oracle::check line, because empty filter exits 0 with 0 passed.
//! Temp: pid plus AtomicUsize plus tag, cleaned by Drop. No tempfile dep, no Cargo.lock change.
//! Diagnostics are indented with "  | " via gate_fail so inner binary output
//! (test .. / ---- .. stdout ---- / failures:) is never read as outer cargo output.
//! How verified: generate then write temp then rustc --test then run binary.
use ec_codegen::{CodeGenerator, GenerationSpec};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static GATE_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn gate_fail(reason: &str, detail: &str) -> ! {
    let indented: String = detail.lines().map(|l| format!("  | {l}\n")).collect();
    panic!("GATE_REASON={reason}\n{indented}")
}

struct GateTempDir {
    path: std::path::PathBuf,
}
impl GateTempDir {
    fn new(tag: &str) -> Self {
        let n = GATE_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("ec-f2-{}-{}-{}", tag, std::process::id(), n));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        Self { path: dir }
    }
    fn path(&self) -> &std::path::Path {
        &self.path
    }
}
impl Drop for GateTempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn gate_generate(spec: &GenerationSpec) -> String {
    let g = CodeGenerator::new();
    let r = g.generate(spec);
    if !r.succeeded() {
        gate_fail("generation_failed", "generator did not succeed");
    }
    r.code().expect("code").to_string()
}

fn gate_compile(
    code: &str,
    dir: &GateTempDir,
    src_name: &str,
    bin_name: &str,
) -> (bool, String, std::path::PathBuf) {
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let src = dir.path().join(src_name);
    let bin = dir.path().join(bin_name);
    std::fs::write(&src, code).expect("write src");
    let out = Command::new(rustc)
        .args(["--edition", "2021", "--test"])
        .arg(&src)
        .arg("-o")
        .arg(&bin)
        .output()
        .expect("rustc");
    let ok = out.status.success();
    let err = String::from_utf8_lossy(&out.stderr).to_string();
    (ok, err, bin)
}

fn gate_run(bin: &std::path::Path, args: &[&str]) -> (bool, String, String) {
    let out = Command::new(bin).args(args).output().expect("run bin");
    let ok = out.status.success();
    let so = String::from_utf8_lossy(&out.stdout).to_string();
    let se = String::from_utf8_lossy(&out.stderr).to_string();
    (ok, so, se)
}

fn gate_assert_generated_passes(name: &str, types: Vec<&str>, output: &str, tag: &str) {
    let spec = GenerationSpec::simple(name, types, output);
    let code = gate_generate(&spec);
    let tmp = GateTempDir::new(tag);
    let (compiled, err, bin) = gate_compile(&code, &tmp, "generated.rs", "generated_test");
    if !compiled {
        gate_fail(
            "compile_failed",
            &format!("{tag}\nstderr:\n{err}\nsource:\n{code}"),
        );
    }
    let (ok, so, se) = gate_run(&bin, &[]);
    if !(ok && so.contains("1 passed")) {
        gate_fail(
            "run_failed",
            &format!("{tag}\nstdout:\n{so}\nstderr:\n{se}\nsource:\n{code}"),
        );
    }
}

fn gate_mutate_default(code: &str, fn_name: &str, float: bool) -> String {
    let anchor = format!("pub fn {}(", fn_name);
    let count = code.matches(anchor.as_str()).count();
    if count != 1 {
        gate_fail(
            "anchor_count",
            &format!("anchor {anchor} count={count} expected 1"),
        );
    }
    let pos = code.find(anchor.as_str()).expect("anchor");
    let after = &code[pos..];
    let brace_rel = after.find('{').expect("brace");
    let insert_at = pos + brace_rel + 1;
    let line = if float {
        "\n    let a = a + 1.0;"
    } else {
        "\n    let a = a + 1;"
    };
    let mut out = String::with_capacity(code.len() + line.len());
    out.push_str(&code[..insert_at]);
    out.push_str(line);
    out.push_str(&code[insert_at..]);
    if out == code {
        gate_fail("mutant_no_change", "mutant equals original");
    }
    let needle = if float {
        "let a = a + 1.0;"
    } else {
        "let a = a + 1;"
    };
    let c = out.matches(needle).count();
    if c != 1 {
        gate_fail("mutant_line_count", &format!("needle count={c} expected 1"));
    }
    out
}

fn gate_mutate_n0_f64(code: &str) -> String {
    let anchor = "pub fn zero_f64(";
    let count = code.matches(anchor).count();
    if count != 1 {
        gate_fail(
            "anchor_count",
            &format!("anchor {anchor} count={count} expected 1"),
        );
    }
    let pos = code.find(anchor).expect("anchor");
    let open = pos + code[pos..].find('{').expect("open brace");
    let close = open + code[open..].find("\n}\n").expect("close brace");
    let body = &code[open..close];
    let old = "0.0_f64";
    let c = body.matches(old).count();
    if c != 1 {
        gate_fail(
            "n0_marker_count",
            &format!("0.0_f64 inside body count={c} expected 1"),
        );
    }
    let mut mutated = String::with_capacity(code.len());
    mutated.push_str(&code[..open]);
    mutated.push_str(&body.replacen(old, "1.0_f64", 1));
    mutated.push_str(&code[close..]);
    if mutated == code {
        gate_fail("n0_mutant_no_change", "mutant equals original");
    }
    mutated
}

fn gate_append_oracle(code: &str, fn_name: &str, call: &str, expected: &str) -> String {
    let mut out = String::from(code);
    out.push_str(&format!(
        "\n#[cfg(test)]\nmod gate_oracle {{\n    use super::*;\n    #[test]\n    fn check() {{\n        let got = {}({});\n        assert_eq!(got, {});\n    }}\n}}\n",
        fn_name, call, expected
    ));
    out
}

fn gate_assert_oracle_passes(code_with_oracle: &str, tag: &str, orig: &str) {
    let tmp = GateTempDir::new(tag);
    let (compiled, err, bin) = gate_compile(code_with_oracle, &tmp, "oracle.rs", "oracle_test");
    if !compiled {
        gate_fail(
            "oracle_compile_failed",
            &format!("{tag}\nstderr:\n{err}\nsource:\n{code_with_oracle}"),
        );
    }
    let (ok, so, se) = gate_run(&bin, &["gate_oracle::check", "--exact"]);
    let combined = format!("{}\n{}", so, se);
    if !combined.contains("test gate_oracle::check") {
        gate_fail(
            "oracle_line_missing",
            &format!("{tag}\nstdout:\n{so}\nstderr:\n{se}\norig:\n{orig}"),
        );
    }
    if !(ok && so.contains("1 passed")) {
        gate_fail(
            "oracle_run_failed",
            &format!("{tag}\nstdout:\n{so}\nstderr:\n{se}\nsource:\n{code_with_oracle}"),
        );
    }
}

fn gate_assert_mutant_killed(mut_code: &str, tag: &str, orig: &str) {
    let tmp = GateTempDir::new(tag);
    let (compiled, err, bin) = gate_compile(mut_code, &tmp, "mut.rs", "mut_test");
    if !compiled {
        gate_fail(
            "mutant_compile_failed_gate_bug",
            &format!("{tag}\nstderr:\n{err}\nsource:\n{mut_code}"),
        );
    }
    let (ok, so, se) = gate_run(&bin, &[]);
    let combined = format!("{}\n{}", so, se);
    if ok {
        gate_fail(
            "mutant_NOT_killed",
            &format!("{tag} tautology passes\nstdout:\n{so}\nstderr:\n{se}\nmut:\n{mut_code}\norig:\n{orig}"),
        );
    }
    if !(combined.contains("panicked") && combined.contains("assertion")) {
        gate_fail(
            "mutant_wrong_failure",
            &format!("{tag} missing panicked/assertion\nstdout:\n{so}\nstderr:\n{se}"),
        );
    }
}

fn gate_assert_no_test_block_and_compiles(code: &str, tag: &str) {
    if code.contains("#[test]") {
        gate_fail(
            "test_block_present",
            &format!("{tag} #[test] present\nsource:\n{code}"),
        );
    }
    let tmp = GateTempDir::new(tag);
    let (compiled, err, _bin) = gate_compile(code, &tmp, "notest.rs", "notest_test");
    if !compiled {
        gate_fail(
            "compile_failed",
            &format!("{tag}\nstderr:\n{err}\nsource:\n{code}"),
        );
    }
}

fn gate_assert_no_test_block_only(code: &str, tag: &str) {
    if code.contains("#[test]") {
        gate_fail(
            "test_block_present",
            &format!("{tag} #[test] present\nsource:\n{code}"),
        );
    }
}

// n=3: 1+2+3 == 1*2*3 == 6, so a pure n=3 oracle cannot tell the templates
// apart. The template actually selected is therefore asserted explicitly.
fn gate_template_of(spec: &GenerationSpec) -> &'static str {
    let r = CodeGenerator::new().generate(spec);
    r.success().map_or("none", |s| s.template_name)
}

fn gate_pure_spec(name: &str, types: Vec<&str>, output: &str) -> GenerationSpec {
    let mut spec = GenerationSpec::simple(name, types, output);
    spec.constraints.push("pure".into());
    spec.constraints.push("no_side_effects".into());
    let picked = gate_template_of(&spec);
    if picked != "RustPureTemplate" {
        gate_fail(
            "wrong_template",
            &format!("{name} picked={picked} expected RustPureTemplate"),
        );
    }
    spec
}

#[test]
fn gate_generated_code_compiles_one_param() {
    gate_assert_generated_passes("identity", vec!["i32"], "i32", "n1");
}

#[test]
fn gate_generated_code_compiles_five_params() {
    gate_assert_generated_passes("sum5", vec!["i32", "i32", "i32", "i32", "i32"], "i32", "n5");
}

#[test]
fn gate_generated_code_compiles_float_param() {
    gate_assert_generated_passes("identity_f64", vec!["f64"], "f64", "f64n1");
}

#[test]
fn gate_generated_code_compiles_two_f32_params() {
    gate_assert_generated_passes("add_f32", vec!["f32", "f32"], "f32", "f32n2");
}

#[test]
fn gate_oracle_pure_i32_n3() {
    let spec = gate_pure_spec("pure_mul3", vec!["i32", "i32", "i32"], "i32");
    let code = gate_generate(&spec);
    let with = gate_append_oracle(&code, "pure_mul3", "1, 2, 3", "6");
    gate_assert_oracle_passes(&with, "pure-n3", &code);
}

#[test]
fn gate_n0_f64_generated_test_passes() {
    gate_assert_generated_passes("zero_f64", vec![], "f64", "n0f64");
}

#[test]
fn gate_mutant_killed_one_param() {
    let code = gate_generate(&GenerationSpec::simple("identity", vec!["i32"], "i32"));
    let m = gate_mutate_default(&code, "identity", false);
    gate_assert_mutant_killed(&m, "mut-n1", &code);
}

#[test]
fn gate_mutant_killed_five_params() {
    let code = gate_generate(&GenerationSpec::simple(
        "sum5",
        vec!["i32", "i32", "i32", "i32", "i32"],
        "i32",
    ));
    let m = gate_mutate_default(&code, "sum5", false);
    gate_assert_mutant_killed(&m, "mut-n5", &code);
}

#[test]
fn gate_mutant_killed_float_param() {
    let code = gate_generate(&GenerationSpec::simple("identity_f64", vec!["f64"], "f64"));
    let m = gate_mutate_default(&code, "identity_f64", true);
    gate_assert_mutant_killed(&m, "mut-f64", &code);
}

#[test]
fn gate_mutant_killed_two_f32_params() {
    let code = gate_generate(&GenerationSpec::simple(
        "add_f32",
        vec!["f32", "f32"],
        "f32",
    ));
    let m = gate_mutate_default(&code, "add_f32", true);
    gate_assert_mutant_killed(&m, "mut-f32", &code);
}

#[test]
fn gate_mutant_killed_pure_i32_n3() {
    let spec = gate_pure_spec("pure_mul3", vec!["i32", "i32", "i32"], "i32");
    let code = gate_generate(&spec);
    let m = gate_mutate_default(&code, "pure_mul3", false);
    gate_assert_mutant_killed(&m, "mut-pure3", &code);
}

#[test]
fn gate_mutant_killed_default_f64_n0() {
    let code = gate_generate(&GenerationSpec::simple("zero_f64", vec![], "f64"));
    let m = gate_mutate_n0_f64(&code);
    gate_assert_mutant_killed(&m, "mut-n0f64", &code);
}

#[test]
fn gate_oracle_ninth_param_default_i32_n9() {
    let code = gate_generate(&GenerationSpec::simple(
        "sum9",
        vec![
            "i32", "i32", "i32", "i32", "i32", "i32", "i32", "i32", "i32",
        ],
        "i32",
    ));
    let with = gate_append_oracle(&code, "sum9", "1, 2, 3, 4, 5, 6, 7, 8, 9", "45");
    gate_assert_oracle_passes(&with, "oracle-sum9", &code);
}

#[test]
fn gate_oracle_ninth_param_pure_i32_n9() {
    let spec = gate_pure_spec(
        "pure_mul9",
        vec![
            "i32", "i32", "i32", "i32", "i32", "i32", "i32", "i32", "i32",
        ],
        "i32",
    );
    let code = gate_generate(&spec);
    let with = gate_append_oracle(&code, "pure_mul9", "1, 2, 3, 4, 5, 6, 7, 8, 9", "362880");
    gate_assert_oracle_passes(&with, "oracle-pure9", &code);
}

#[test]
fn gate_no_test_block_bool_n1() {
    let code = gate_generate(&GenerationSpec::simple(
        "identity_bool",
        vec!["bool"],
        "bool",
    ));
    gate_assert_no_test_block_and_compiles(&code, "bool-n1");
}

#[test]
fn gate_no_test_block_char_n1() {
    let code = gate_generate(&GenerationSpec::simple(
        "identity_char",
        vec!["char"],
        "char",
    ));
    gate_assert_no_test_block_and_compiles(&code, "char-n1");
}

#[test]
fn gate_no_test_block_string_n1() {
    let code = gate_generate(&GenerationSpec::simple(
        "identity_string",
        vec!["String"],
        "String",
    ));
    gate_assert_no_test_block_and_compiles(&code, "string-n1");
}

#[test]
fn gate_no_test_block_str_n1() {
    let code = gate_generate(&GenerationSpec::simple(
        "identity_str",
        vec!["&str"],
        "&str",
    ));
    gate_assert_no_test_block_and_compiles(&code, "str-n1");
}

#[test]
fn gate_no_test_block_vec_i32_n1() {
    let code = gate_generate(&GenerationSpec::simple(
        "identity_vec",
        vec!["Vec<i32>"],
        "Vec<i32>",
    ));
    gate_assert_no_test_block_and_compiles(&code, "vec-n1");
}

#[test]
fn gate_no_test_block_todo_i32_n0() {
    let code = gate_generate(&GenerationSpec::simple("todo_fn", vec![], "i32"));
    gate_assert_no_test_block_and_compiles(&code, "todo-n0");
}

#[test]
fn gate_no_test_block_mixed_i32_f64() {
    let code = gate_generate(&GenerationSpec::simple(
        "mixed_fn",
        vec!["i32", "f64"],
        "f64",
    ));
    gate_assert_no_test_block_only(&code, "mixed");
}

#[test]
fn gate_no_test_block_overflow_u8_pure_n6() {
    let spec = gate_pure_spec(
        "overflow_pure",
        vec!["u8", "u8", "u8", "u8", "u8", "u8"],
        "u8",
    );
    let code = gate_generate(&spec);
    gate_assert_no_test_block_and_compiles(&code, "overflow-u8");
}

#[test]
fn gate_template_probe_discriminates() {
    let plain = GenerationSpec::simple("probe_plain", vec!["i32", "i32", "i32"], "i32");
    let pure = gate_pure_spec("probe_pure", vec!["i32", "i32", "i32"], "i32");
    let a = gate_template_of(&plain);
    let b = gate_template_of(&pure);
    if a != "RustFunctionTemplate" || b != "RustPureTemplate" {
        gate_fail("probe_not_discriminating", &format!("plain={a} pure={b}"));
    }
}
