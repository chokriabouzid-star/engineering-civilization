use crate::result::GenerationResult;
use crate::rust::{RustFunctionTemplate, RustPureTemplate, RustStructTemplate};
use crate::spec::GenerationSpec;
use crate::template::CodeTemplate;

/// Code Generator — يختار أنسب template وينفذه
#[derive(Debug)]
pub struct CodeGenerator {
    templates: Vec<Box<dyn CodeTemplate>>,
}

impl CodeGenerator {
    /// إنشاء generator مع templates الافتراضية
    pub fn new() -> Self {
        let mut templates: Vec<Box<dyn CodeTemplate>> = vec![
            Box::new(RustPureTemplate),
            Box::new(RustFunctionTemplate),
            Box::new(RustStructTemplate),
        ];
        templates.sort_by_key(|b| std::cmp::Reverse(b.priority()));
        Self { templates }
    }

    /// توليد الكود من المواصفات
    pub fn generate(&self, spec: &GenerationSpec) -> GenerationResult {
        for template in &self.templates {
            if template.matches(spec) {
                return template.generate(spec);
            }
        }
        GenerationResult::Failed {
            reason: format!(
                "No template matches spec for function '{}' with {} input(s)",
                spec.function_name,
                spec.input_types.len()
            ),
        }
    }

    /// عدد الـ templates المسجلة
    pub fn template_count(&self) -> usize {
        self.templates.len()
    }

    /// أسماء الـ templates المتاحة
    pub fn template_names(&self) -> Vec<&'static str> {
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
    fn test_generator_creates() {
        let g = CodeGenerator::new();
        assert!(g.template_count() > 0);
        assert!(g.template_names().contains(&"RustFunctionTemplate"));
    }

    #[test]
    fn test_generate_add_function() {
        let g = CodeGenerator::new();
        let spec = GenerationSpec::simple("add", vec!["i32", "i32"], "i32");
        let result = g.generate(&spec);
        assert!(result.succeeded());
        let code = result.code().unwrap();
        assert!(code.contains("pub fn add"));
    }

    #[test]
    fn test_no_matching_template() {
        let g = CodeGenerator::new();
        let mut spec = GenerationSpec::simple("ptrmagic", vec!["*const u8"], "*mut u8");
        spec.constraints.push("unsafe".into());
        spec.constraints.push("raw_ptr".into());
        let result = g.generate(&spec);
        assert!(!result.succeeded());
    }
}
