#![forbid(unsafe_code)]

use ec_analysis::analyze_code;
use ec_codegen::{CodeGenerator, GenerationResult, GenerationSpec};

#[test]
fn gate_generator_creates() {
    let g = CodeGenerator::new();
    assert!(g.template_count() > 0);
}

#[test]
fn gate_generates_add_function() {
    let g = CodeGenerator::new();
    let spec = GenerationSpec::simple("add", vec!["i32", "i32"], "i32");
    let result = g.generate(&spec);
    assert!(result.succeeded());
    let code = result.code().unwrap();
    assert!(code.contains("pub fn add"), "code: {}", code);
    assert!(code.contains("i32"));
}

#[test]
fn gate_generates_float_computation() {
    let g = CodeGenerator::new();
    let spec = GenerationSpec::simple("area", vec!["f64", "f64"], "f64");
    let result = g.generate(&spec);
    assert!(result.succeeded());
    let code = result.code().unwrap();
    assert!(code.contains("pub fn area"), "code: {}", code);
}

#[test]
fn gate_generated_code_has_tests() {
    let g = CodeGenerator::new();
    let spec = GenerationSpec::simple("multiply", vec!["i32", "i32"], "i32");
    let result = g.generate(&spec);
    let code = result.code().unwrap();
    assert!(code.contains("#[test]"), "code: {}", code);
}

#[test]
fn gate_generated_code_no_unsafe() {
    let g = CodeGenerator::new();
    let spec = GenerationSpec::simple("safe_fn", vec!["i32"], "i32");
    let result = g.generate(&spec);
    let code = result.code().unwrap();
    assert!(!code.contains("unsafe"), "code contains unsafe: {}", code);
}

#[test]
fn gate_analyze_code_works_on_generated() {
    let g = CodeGenerator::new();
    let spec = GenerationSpec::simple("calc", vec!["i32", "i32"], "i32");
    let result = g.generate(&spec);
    let code = result.code().unwrap();
    let fitness = analyze_code(code);
    assert!(fitness.security > 0.0, "security should be nonzero");
}

#[test]
fn gate_pure_template_used_for_pure_constraint() {
    let g = CodeGenerator::new();
    let mut spec = GenerationSpec::simple("pure_calc", vec!["f64"], "f64");
    spec.constraints.push("pure".into());
    spec.constraints.push("no_side_effects".into());
    let result = g.generate(&spec);
    assert!(result.succeeded());
    let success = result.success().unwrap();
    assert_eq!(success.template_name, "RustPureTemplate");
}

#[test]
fn gate_generates_struct() {
    let g = CodeGenerator::new();
    let mut spec = GenerationSpec::simple("Point", vec!["f64", "f64"], "()");
    spec.constraints.push("struct".into());
    let result = g.generate(&spec);
    assert!(result.succeeded());
    let code = result.code().unwrap();
    assert!(code.contains("pub struct Point"), "code: {}", code);
}

#[test]
fn gate_generation_result_api() {
    let r: GenerationResult = GenerationResult::Failed {
        reason: "oops".into(),
    };
    assert!(!r.succeeded());
    assert!(r.code().is_none());
}

#[test]
fn gate_adaptive_with_failure() {
    let g = CodeGenerator::new();
    let mut spec = GenerationSpec::simple("adaptive_fn", vec!["i32"], "i32");
    spec.include_tests = true;
    let result = g.generate(&spec);
    assert!(result.succeeded());
}
