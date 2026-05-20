use crate::result::GenerationResult;
use crate::spec::GenerationSpec;

/// Template trait — كل template يُنتج نوعاً من الكود
pub trait CodeTemplate: Send + Sync + std::fmt::Debug {
    /// اسم الـ template
    fn name(&self) -> &'static str;
    /// هل هذا الـ template مناسب للـ spec؟
    fn matches(&self, spec: &GenerationSpec) -> bool;
    /// توليد الكود
    fn generate(&self, spec: &GenerationSpec) -> GenerationResult;
    /// أولوية الـ template (أعلى = يُجرَّب أولاً)
    fn priority(&self) -> u8 {
        50
    }
}
