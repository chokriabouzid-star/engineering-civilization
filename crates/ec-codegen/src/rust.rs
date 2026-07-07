use crate::result::{GenerationResult, GenerationSuccess};
use crate::spec::GenerationSpec;
use crate::template::CodeTemplate;

// ─── دوال مساعدة ──────────────────────────────────────────────────

fn is_struct_like(spec: &GenerationSpec) -> bool {
    spec.function_name
        .chars()
        .next()
        .map(|c| c.is_uppercase())
        .unwrap_or(false)
        || spec.constraints.iter().any(|c| c.contains("struct"))
}

fn generate_default_body(spec: &GenerationSpec) -> String {
    let letters = ["a", "b", "c", "d", "e", "f", "g", "h"];
    let args: Vec<&str> = letters
        .iter()
        .take(spec.input_types.len())
        .copied()
        .collect();

    match spec.input_types.len() {
        0 => {
            if spec.output_type == "String" {
                format!("    String::from(\"{}\")", spec.function_name)
            } else if spec.output_type == "f64" {
                "    0.0_f64".to_string()
            } else if spec.output_type == "f32" {
                "    0.0_f32".to_string()
            } else {
                "    todo!()".to_string()
            }
        }
        1 => {
            let a = args[0];
            if spec.output_type == spec.input_types[0] {
                format!("    {}", a)
            } else {
                format!("    {} as {}", a, spec.output_type)
            }
        }
        2 => {
            let (a, b) = (args[0], args[1]);
            format!("    {} + {}", a, b)
        }
        _ => {
            let sum: String = args
                .iter()
                .map(|a| a.to_string())
                .collect::<Vec<_>>()
                .join(" + ");
            format!("    {}", sum)
        }
    }
}

fn generate_pure_body(spec: &GenerationSpec) -> String {
    let letters = ["a", "b", "c", "d", "e", "f", "g", "h"];
    let args: Vec<&str> = letters
        .iter()
        .take(spec.input_types.len())
        .copied()
        .collect();

    if spec.input_types.is_empty() {
        return "    42".to_string();
    }

    match spec.input_types.len() {
        1 => format!("    {}", args[0]),
        2 => format!("    {} * {}", args[0], args[1]),
        _ => {
            let sum = args
                .iter()
                .map(|a| a.to_string())
                .collect::<Vec<_>>()
                .join(" * ");
            format!("    {}", sum)
        }
    }
}

fn generate_tests(spec: &GenerationSpec, _body: &str) -> String {
    let fn_name = &spec.function_name;
    let letters = ["a", "b", "c", "d"];
    let args: Vec<&str> = letters
        .iter()
        .take(spec.input_types.len())
        .copied()
        .collect();
    let call_args = args.join(", ");

    format!(
        "\n#[cfg(test)]\nmod tests {{\n    use super::*;\n\n    #[test]\n    fn test_{fn_name}() {{\n        let result = {fn_name}({call_args});\n        let _ = result;\n    }}\n}}\n",
        fn_name = fn_name,
        call_args = call_args,
    )
}

// ─── RustPureTemplate ──────────────────────────────────────────────

#[derive(Debug)]
pub struct RustPureTemplate;

impl CodeTemplate for RustPureTemplate {
    fn name(&self) -> &'static str {
        "RustPureTemplate"
    }
    fn matches(&self, spec: &GenerationSpec) -> bool {
        spec.constraints
            .iter()
            .any(|c| c.contains("pure") || c.contains("no_side_effects"))
    }
    fn priority(&self) -> u8 {
        80
    }

    fn generate(&self, spec: &GenerationSpec) -> GenerationResult {
        let body = generate_pure_body(spec);
        let tests = if spec.include_tests {
            generate_tests(spec, &body)
        } else {
            String::new()
        };
        let desc = if spec.description.is_empty() {
            format!("Pure function: {}", spec.function_name)
        } else {
            spec.description.clone()
        };
        let code = format!(
            "/// {desc}\n#[inline]\npub fn {name}({params}) -> {out} {{\n{body}\n}}\n{tests}",
            desc = desc,
            name = spec.function_name,
            params = spec.format_params(),
            out = spec.output_type,
            body = body,
            tests = tests,
        );
        GenerationResult::Success(GenerationSuccess::new(
            code,
            self.name(),
            spec.attempt_number(),
        ))
    }
}

// ─── RustFunctionTemplate ───────────────────────────────────────────

#[derive(Debug)]
pub struct RustFunctionTemplate;

impl CodeTemplate for RustFunctionTemplate {
    fn name(&self) -> &'static str {
        "RustFunctionTemplate"
    }
    fn matches(&self, spec: &GenerationSpec) -> bool {
        // لا تطابق الأسماء التي تبدأ بحرف كبير (تلك للـ structs)
        // ولا specs فيها "struct" أو unsafe
        !spec.function_name.is_empty()
            && !spec.requires_unsafe()
            && !is_struct_like(spec)
            && spec.output_type != "()"
    }

    fn generate(&self, spec: &GenerationSpec) -> GenerationResult {
        let body = generate_default_body(spec);
        let tests = if spec.include_tests {
            generate_tests(spec, &body)
        } else {
            String::new()
        };
        let desc = if spec.description.is_empty() {
            format!("Compute: {}", spec.function_name)
        } else {
            spec.description.clone()
        };
        let code = format!(
            "/// {desc}\npub fn {name}({params}) -> {out} {{\n{body}\n}}\n{tests}",
            desc = desc,
            name = spec.function_name,
            params = spec.format_params(),
            out = spec.output_type,
            body = body,
            tests = tests,
        );
        GenerationResult::Success(GenerationSuccess::new(
            code,
            self.name(),
            spec.attempt_number(),
        ))
    }
}

// ─── RustStructTemplate ─────────────────────────────────────────────

#[derive(Debug)]
pub struct RustStructTemplate;

impl CodeTemplate for RustStructTemplate {
    fn name(&self) -> &'static str {
        "RustStructTemplate"
    }
    fn matches(&self, spec: &GenerationSpec) -> bool {
        is_struct_like(spec) || spec.output_type == "()"
    }
    fn priority(&self) -> u8 {
        60
    }

    fn generate(&self, spec: &GenerationSpec) -> GenerationResult {
        let struct_name = &spec.function_name;
        let fields: Vec<String> = spec
            .input_types
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let name = if i == 0 {
                    "value".to_string()
                } else {
                    format!("field_{}", i)
                };
                format!("    pub {}: {}", name, t)
            })
            .collect();

        let field_names: Vec<String> = spec
            .input_types
            .iter()
            .enumerate()
            .map(|(i, _)| {
                if i == 0 {
                    "value".to_string()
                } else {
                    format!("field_{}", i)
                }
            })
            .collect();

        let code = format!(
            "/// {desc}\n#[derive(Debug, Clone)]\npub struct {name} {{\n{fields}\n}}\n\nimpl {name} {{\n    pub fn new({params}) -> Self {{\n        Self {{ {field_names} }}\n    }}\n}}\n",
            desc = if spec.description.is_empty() { format!("Struct: {}", struct_name) } else { spec.description.clone() },
            name = struct_name,
            fields = fields.join(",\n"),
            params = spec.format_params(),
            field_names = field_names.join(", "),
        );
        GenerationResult::Success(GenerationSuccess::new(
            code,
            self.name(),
            spec.attempt_number(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_function_template_generates() {
        let spec = GenerationSpec::simple("add", vec!["i32", "i32"], "i32");
        let tmpl = RustFunctionTemplate;
        assert!(tmpl.matches(&spec));
        let result = tmpl.generate(&spec);
        assert!(result.succeeded());
        let code = result.code().unwrap();
        assert!(code.contains("pub fn add"));
        assert!(code.contains("-> i32"));
        assert!(!code.contains("unsafe"));
    }

    #[test]
    fn test_pure_template_generates_inline() {
        let mut spec = GenerationSpec::simple("square", vec!["f64"], "f64");
        spec.constraints.push("pure".into());
        spec.constraints.push("no_side_effects".into());
        let tmpl = RustPureTemplate;
        assert!(tmpl.matches(&spec));
        let result = tmpl.generate(&spec);
        let code = result.code().unwrap();
        assert!(code.contains("#[inline]"));
        assert!(code.contains("pub fn square"));
    }

    #[test]
    fn test_includes_tests() {
        let spec = GenerationSpec::simple("sum", vec!["i32", "i32", "i32"], "i32");
        let tmpl = RustFunctionTemplate;
        let result = tmpl.generate(&spec);
        let code = result.code().unwrap();
        assert!(code.contains("#[test]"));
    }

    #[test]
    fn test_function_template_rejects_struct_name() {
        let spec = GenerationSpec::simple("Point", vec!["f64", "f64"], "()");
        let tmpl = RustFunctionTemplate;
        assert!(!tmpl.matches(&spec));
    }

    #[test]
    fn test_struct_template_matches_uppercase() {
        let mut spec = GenerationSpec::simple("Point", vec!["f64", "f64"], "()");
        spec.constraints.push("struct".into());
        let tmpl = RustStructTemplate;
        assert!(tmpl.matches(&spec));
        let result = tmpl.generate(&spec);
        let code = result.code().unwrap();
        assert!(code.contains("pub struct Point"), "code: {}", code);
    }
}
