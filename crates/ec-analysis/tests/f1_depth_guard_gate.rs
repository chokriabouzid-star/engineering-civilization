#![forbid(unsafe_code)]

//! F1 Depth Guard Regression Gate
//!
//! Contract:
//! ✓ التعشيق العميق للأقواس ({}, (), []) لا يسقط العملية ويُرجع unparseable
//! ✓ تكرار العمليات الأحادية (&&&&, ****) لا يسقط العملية ويُرجع unparseable
//! ✓ الكود السليم ذو العمق الطبيعي يُحلَّل بنجاح كامل

use ec_analysis::analyze_code_full;

#[test]
fn f1_gate_deep_braces_handled_safely() {
    let code = format!("fn f() {}let x = 1;{}", "{".repeat(512), "}".repeat(512));
    let report = analyze_code_full(&code);
    assert!(!report.parse_successful);
    assert_eq!(report.warnings.len(), 1);
    let warning_msg = format!("{:?}", report.warnings[0]);
    assert!(warning_msg.contains("Nesting depth"));
}

#[test]
fn f1_gate_deep_parens_handled_safely() {
    let code = format!(
        "fn f(){{ let x = {}1{}; }}",
        "(".repeat(1200),
        ")".repeat(1200)
    );
    let report = analyze_code_full(&code);
    assert!(!report.parse_successful);
    assert_eq!(report.warnings.len(), 1);
}

#[test]
fn f1_gate_deep_refs_handled_safely() {
    let code = format!("fn f(){{ let x: {}u8 = todo!(); }}", "& ".repeat(1200));
    let report = analyze_code_full(&code);
    assert!(!report.parse_successful);
    assert_eq!(report.warnings.len(), 1);
}

#[test]
fn f1_gate_valid_nested_code_passes() {
    let code = r#"
        fn compute(a: i32) -> i32 {
            if a > 0 {
                let mut sum = 0;
                for i in 0..a {
                    sum += match i % 2 {
                        0 => i * 2,
                        _ => i + 1,
                    };
                }
                sum
            } else {
                0
            }
        }
    "#;
    let report = analyze_code_full(code);
    assert!(report.parse_successful);
    assert!(report.fitness.maintainability > 0.0);
}
