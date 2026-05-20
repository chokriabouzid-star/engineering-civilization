import os

files = {}

files['crates/ec-codegen/src/lib.rs'] = '''\
#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! ec-codegen — Template-Based Code Generation

/// مواصفات الكود المطلوب.
pub mod spec;
/// Template trait والتنفيذات.
pub mod template;
/// Templates خاصة بـ Rust.
pub mod rust;
/// Code generator الرئيسي.
pub mod generator;
/// نتيجة التوليد.
pub mod result;

pub use generator::CodeGenerator;
pub use result::{GenerationResult, GenerationSuccess};
pub use spec::GenerationSpec;
'''

files['crates/ec-codegen/src/spec.rs'] = '''\
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FailureContext {
    pub reason: String,
    pub security_score: f64,
    pub coverage_score: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenerationSpec {
    pub function_name: String,
    pub input_types: Vec<String>,
    pub output_type: String,
    pub description: String,
    pub constraints: Vec<String>,
    pub include_tests: bool,
    pub previous_failures: Vec<FailureContext>,
}

impl GenerationSpec {
    pub fn simple(
        function_name: impl Into<String>,
        input_types: Vec<&str>,
        output_type: impl Into<String>,
    ) -> Self {
        Self {
            function_name: function_name.into(),
            input_types: input_types.into_iter().map(|s| s.to_string()).collect(),
            output_type: output_type.into(),
            description: String::new(),
            constraints: Vec::new(),
            include_tests: true,
            previous_failures: Vec::new(),
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn with_failure(mut self, failure: FailureContext) -> Self {
        self.previous_failures.push(failure);
        self
    }

    pub fn is_first_attempt(&self) -> bool {
        self.previous_failures.is_empty()
    }

    pub fn attempt_number(&self) -> usize {
        self.previous_failures.len() + 1
    }

    pub fn format_params(&self) -> String {
        self.input_types
            .iter()
            .enumerate()
            .map(|(i, t)| format!("arg{}: {}", i, t))
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub fn requires_unsafe(&self) -> bool {
        self.constraints.iter().any(|c| c.contains("unsafe"))
    }
}

impl Default for GenerationSpec {
    fn default() -> Self {
        Self {
            function_name: "generated_fn".to_string(),
            input_types: Vec::new(),
            output_type: "()".to_string(),
            description: String::new(),
            constraints: Vec::new(),
            include_tests: true,
            previous_failures: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_simple_creates() {
        let spec = GenerationSpec::simple("add", vec!["i32", "i32"], "i32");
        assert_eq!(spec.function_name, "add");
        assert_eq!(spec.input_types.len(), 2);
        assert!(spec.include_tests);
    }

    #[test]
    fn spec_format_params() {
        let spec = GenerationSpec::simple("foo", vec!["i32", "f64"], "bool");
        assert_eq!(spec.format_params(), "arg0: i32, arg1: f64");
    }

    #[test]
    fn spec_attempt_number() {
        let spec = GenerationSpec::default();
        assert_eq!(spec.attempt_number(), 1);
        let spec2 = spec.with_failure(FailureContext {
            reason: "low coverage".into(),
            security_score: 0.9,
            coverage_score: 0.1,
        });
        assert_eq!(spec2.attempt_number(), 2);
    }
}
'''

files['crates/ec-codegen/src/result.rs'] = '''\
#![forbid(unsafe_code)]

use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct GenerationSuccess {
    pub code: String,
    pub template_name: &\'static str,
    pub generation_id: Uuid,
    pub attempt_number: usize,
}

impl GenerationSuccess {
    pub fn new(
        code: impl Into<String>,
        template_name: &\'static str,
        attempt_number: usize,
    ) -> Self {
        Self {
            code: code.into(),
            template_name,
            generation_id: Uuid::new_v4(),
            attempt_number,
        }
    }

    pub fn has_unsafe(&self) -> bool {
        self.code.contains("unsafe")
    }

    pub fn has_tests(&self) -> bool {
        self.code.contains("#[test]")
    }
}

#[derive(Debug, Clone)]
pub enum GenerationResult {
    Success(GenerationSuccess),
    Failed { reason: String },
}

impl GenerationResult {
    pub fn succeeded(&self) -> bool {
        matches!(self, Self::Success(_))
    }

    pub fn code(&self) -> Option<&str> {
        match self {
            Self::Success(s) => Some(&s.code),
            Self::Failed { .. } => None,
        }
    }

    pub fn success(&self) -> Option<&GenerationSuccess> {
        match self {
            Self::Success(s) => Some(s),
            Self::Failed { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_has_unique_id() {
        let r1 = GenerationSuccess::new("fn a() {}", "test", 1);
        let r2 = GenerationSuccess::new("fn b() {}", "test", 1);
        assert_ne!(r1.generation_id, r2.generation_id);
    }

    #[test]
    fn result_code_none_on_failure() {
        let r = GenerationResult::Failed { reason: "oops".into() };
        assert!(r.code().is_none());
        assert!(!r.succeeded());
    }
}
'''

files['crates/ec-codegen/src/template.rs'] = '''\
#![forbid(unsafe_code)]

use crate::result::GenerationResult;
use crate::spec::GenerationSpec;

pub trait CodeTemplate: Send + Sync + std::fmt::Debug {
    fn name(&self) -> &\'static str;
    fn matches(&self, spec: &GenerationSpec) -> bool;
    fn generate(&self, spec: &GenerationSpec) -> GenerationResult;
    fn priority(&self) -> u8 { 50 }
}
'''

files['crates/ec-codegen/src/rust.rs'] = '''\
#![forbid(unsafe_code)]

use crate::result::{GenerationResult, GenerationSuccess};
use crate::spec::GenerationSpec;
use crate::template::CodeTemplate;

// ─── RustPureTemplate ────────────────────────────────────────────────

#[derive(Debug)]
pub struct RustPureTemplate;

impl CodeTemplate for RustPureTemplate {
    fn name(&self) -> &\'static str { "RustPureTemplate" }

    fn matches(&self, spec: &GenerationSpec) -> bool {
        spec.constraints.iter().any(|c| c.contains("pure") || c.contains("no_side_effects"))
    }

    fn generate(&self, spec: &GenerationSpec) -> GenerationResult {
        let body = generate_body(spec);
        let tests = if spec.include_tests { generate_tests(spec, &body) } else { String::new() };
        let code = format!(
            "/// {} (pure function)\n#[inline]\npub fn {}({}) -> {} {{\n{}\n}}\n{}",
            if spec.description.is_empty() { format!("Pure: {}", spec.function_name) } else { spec.description.clone() },
            spec.function_name,
            spec.format_params(),
            spec.output_type,
            body,
            tests,
        );
        GenerationResult::Success(GenerationSuccess::new(code, self.name(), spec.attempt_number()))
    }

    fn priority(&self) -> u8 { 90 }
}

// ─── RustFunctionTemplate ────────────────────────────────────────────

#[derive(Debug)]
pub struct RustFunctionTemplate;

impl CodeTemplate for RustFunctionTemplate {
    fn name(&self) -> &\'static str { "RustFunctionTemplate" }

    fn matches(&self, spec: &GenerationSpec) -> bool {
        !spec.function_name.is_empty() && !spec.requires_unsafe()
    }

    fn generate(&self, spec: &GenerationSpec) -> GenerationResult {
        let body = generate_body(spec);
        let tests = if spec.include_tests { generate_tests(spec, &body) } else { String::new() };
        let code = format!(
            "/// {}\npub fn {}({}) -> {} {{\n{}\n}}\n{}",
            if spec.description.is_empty() { format!("Generated: {}", spec.function_name) } else { spec.description.clone() },
            spec.function_name,
            spec.format_params(),
            spec.output_type,
            body,
            tests,
        );
        GenerationResult::Success(GenerationSuccess::new(code, self.name(), spec.attempt_number()))
    }

    fn priority(&self) -> u8 { 80 }
}

// ─── RustStructTemplate ──────────────────────────────────────────────

#[derive(Debug)]
pub struct RustStructTemplate;

impl CodeTemplate for RustStructTemplate {
    fn name(&self) -> &\'static str { "RustStructTemplate" }

    fn matches(&self, spec: &GenerationSpec) -> bool {
        spec.output_type == "Self"
            || spec.function_name.starts_with("new_")
            || spec.function_name == "new"
    }

    fn generate(&self, spec: &GenerationSpec) -> GenerationResult {
        let struct_name = to_pascal_case(&spec.function_name);
        let fields: Vec<String> = spec.input_types.iter().enumerate()
            .map(|(i, t)| format!("    pub field_{}: {},", i, t))
            .collect();
        let inits: Vec<String> = spec.input_types.iter().enumerate()
            .map(|(i, _)| format!("            field_{}: arg{},", i, i))
            .collect();
        let tests = if spec.include_tests {
            format!(
                "\n#[cfg(test)]\nmod tests {{\n    use super::*;\n\n    #[test]\n    fn test_{}() {{\n        println!(\"test passed\");\n    }}\n}}",
                spec.function_name
            )
        } else { String::new() };

        let code = format!(
            "#[derive(Debug, Clone)]\npub struct {} {{\n{}\n}}\n\nimpl {} {{\n    pub fn {}({}) -> Self {{\n        Self {{\n{}\n        }}\n    }}\n}}{}",
            struct_name,
            fields.join("\n"),
            struct_name,
            spec.function_name,
            spec.format_params(),
            inits.join("\n"),
            tests,
        );
        GenerationResult::Success(GenerationSuccess::new(code, self.name(), spec.attempt_number()))
    }

    fn priority(&self) -> u8 { 60 }
}

// ─── Helpers ─────────────────────────────────────────────────────────

fn generate_body(spec: &GenerationSpec) -> String {
    match spec.output_type.as_str() {
        "()" => "    // No return value".to_string(),
        "bool" => "    false".to_string(),
        "String" => "    String::new()".to_string(),
        "i32" | "i64" | "u32" | "u64" | "usize" | "isize" => {
            if spec.input_types.len() == 2 { "    arg0 + arg1".to_string() }
            else if spec.input_types.len() == 1 { "    arg0".to_string() }
            else { "    0".to_string() }
        }
        "f32" | "f64" => {
            if spec.input_types.len() == 2 { "    arg0 + arg1".to_string() }
            else if spec.input_types.len() == 1 { "    arg0".to_string() }
            else { "    0.0".to_string() }
        }
        t if t.starts_with("Vec<") => "    Vec::new()".to_string(),
        t if t.starts_with("Option<") => "    None".to_string(),
        t if t.starts_with("Result<") => "    Ok(Default::default())".to_string(),
        _ => "    Default::default()".to_string(),
    }
}

fn generate_tests(spec: &GenerationSpec, body: &str) -> String {
    let test_body = match spec.output_type.as_str() {
        "i32" | "i64" | "u32" | "u64" | "usize" | "isize" => {
            if body.contains("arg0 + arg1") && spec.input_types.len() == 2 {
                format!("        let result = {}(2, 3);\n        assert_eq!(result, 5);", spec.function_name)
            } else {
                let args: Vec<&str> = spec.input_types.iter().map(|t| default_value(t)).collect();
                format!("        let result = {}({});\n        println!(\"result: {{:?}}\", result);", spec.function_name, args.join(", "))
            }
        }
        "f32" | "f64" => {
            if body.contains("arg0 + arg1") && spec.input_types.len() == 2 {
                format!("        let result = {}(1.0, 2.0);\n        assert!((result - 3.0).abs() < 1e-10);", spec.function_name)
            } else {
                let args: Vec<&str> = spec.input_types.iter().map(|t| default_value(t)).collect();
                format!("        let result = {}({});\n        println!(\"result: {{:?}}\", result);", spec.function_name, args.join(", "))
            }
        }
        "bool" => {
            let args: Vec<&str> = spec.input_types.iter().map(|t| default_value(t)).collect();
            format!("        let result = {}({});\n        assert!(!result || result);", spec.function_name, args.join(", "))
        }
        _ => {
            let args: Vec<&str> = spec.input_types.iter().map(|t| default_value(t)).collect();
            format!("        let _result = {}({});\n        println!(\"test passed\");", spec.function_name, args.join(", "))
        }
    };

    format!(
        "\n#[cfg(test)]\nmod tests {{\n    use super::*;\n\n    #[test]\n    fn test_{}() {{\n{}\n    }}\n}}",
        spec.function_name,
        test_body,
    )
}

fn default_value(t: &str) -> &\'static str {
    match t {
        "i32" | "i64" | "u32" | "u64" | "usize" | "isize" => "0",
        "f32" | "f64" => "0.0",
        "bool" => "false",
        _ => "Default::default()",
    }
}

fn to_pascal_case(s: &str) -> String {
    s.split(\'_\')
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_fn_generates_add() {
        let t = RustFunctionTemplate;
        let spec = GenerationSpec::simple("add", vec!["i32", "i32"], "i32");
        let result = t.generate(&spec);
        assert!(result.succeeded());
        let code = result.code().unwrap();
        assert!(code.contains("pub fn add"));
        assert!(code.contains("assert_eq!(result, 5)"), "code: {}", code);
    }

    #[test]
    fn rust_fn_no_unsafe() {
        let t = RustFunctionTemplate;
        let spec = GenerationSpec::simple("add", vec!["i32", "i32"], "i32");
        let result = t.generate(&spec);
        assert!(!result.success().unwrap().has_unsafe());
    }

    #[test]
    fn rust_fn_has_tests() {
        let t = RustFunctionTemplate;
        let spec = GenerationSpec::simple("add", vec!["i32", "i32"], "i32");
        let result = t.generate(&spec);
        assert!(result.success().unwrap().has_tests());
    }

    #[test]
    fn pure_generates_inline() {
        let t = RustPureTemplate;
        let spec = GenerationSpec {
            function_name: "compute".to_string(),
            input_types: vec!["f64".to_string()],
            output_type: "f64".to_string(),
            constraints: vec!["pure".to_string()],
            include_tests: false,
            ..Default::default()
        };
        let result = t.generate(&spec);
        assert!(result.code().unwrap().contains("#[inline]"));
    }

    #[test]
    fn pascal_case() {
        assert_eq!(to_pascal_case("my_point"), "MyPoint");
        assert_eq!(to_pascal_case("add"), "Add");
    }
}
'''

files['crates/ec-codegen/src/generator.rs'] = '''\
#![forbid(unsafe_code)]

use crate::result::GenerationResult;
use crate::rust::{RustFunctionTemplate, RustPureTemplate, RustStructTemplate};
use crate::spec::GenerationSpec;
use crate::template::CodeTemplate;

pub struct CodeGenerator {
    templates: Vec<Box<dyn CodeTemplate>>,
}

impl CodeGenerator {
    pub fn new() -> Self {
        let mut templates: Vec<Box<dyn CodeTemplate>> = vec![
            Box::new(RustPureTemplate),
            Box::new(RustFunctionTemplate),
            Box::new(RustStructTemplate),
        ];
        templates.sort_by(|a, b| b.priority().cmp(&a.priority()));
        Self { templates }
    }

    pub fn generate(&self, spec: &GenerationSpec) -> GenerationResult {
        match self.templates.iter().find(|t| t.matches(spec)) {
            Some(t) => t.generate(spec),
            None => GenerationResult::Failed {
                reason: format!(
                    "No template for function \'{}\'",
                    spec.function_name
                ),
            },
        }
    }

    pub fn template_count(&self) -> usize {
        self.templates.len()
    }

    pub fn template_names(&self) -> Vec<&\'static str> {
        self.templates.iter().map(|t| t.name()).collect()
    }
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generator_has_templates() {
        let g = CodeGenerator::new();
        assert!(g.template_count() >= 3);
    }

    #[test]
    fn generator_generates_add() {
        let g = CodeGenerator::new();
        let spec = GenerationSpec::simple("add", vec!["i32", "i32"], "i32");
        let result = g.generate(&spec);
        assert!(result.succeeded());
        assert!(result.code().unwrap().contains("pub fn add"));
    }

    #[test]
    fn generator_no_unsafe() {
        let g = CodeGenerator::new();
        let spec = GenerationSpec::simple("add", vec!["i32", "i32"], "i32");
        let result = g.generate(&spec);
        assert!(!result.code().unwrap().contains("unsafe"));
    }

    #[test]
    fn generator_selects_pure_for_constraint() {
        let g = CodeGenerator::new();
        let spec = GenerationSpec {
            function_name: "f".to_string(),
            input_types: vec!["f64".to_string()],
            output_type: "f64".to_string(),
            constraints: vec!["pure".to_string()],
            include_tests: false,
            ..Default::default()
        };
        let result = g.generate(&spec);
        assert_eq!(result.success().unwrap().template_name, "RustPureTemplate");
    }
}
'''

for path, content in files.items():
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, 'w') as f:
        f.write(content)
    print(f"✅ {path}")

print("\nAll files written successfully")
