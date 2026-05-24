#![forbid(unsafe_code)]

use syn::visit::Visit;

pub struct TestVisitor {
    pub test_fns: usize,
    pub production_fns: usize,
    pub assert_count: u32,
}

impl TestVisitor {
    pub fn new() -> Self {
        Self { test_fns: 0, production_fns: 0, assert_count: 0 }
    }

    /// (test_coverage_score, confidence)
    pub fn score(&self) -> (f64, f64) {
        if self.production_fns == 0 {
            // لا دوال إنتاج في هذا الملف — الاختبارات في tests/ منفصل
            return (1.0, 0.30);
        }
        let ratio = (self.test_fns as f64 / self.production_fns as f64)
            .min(1.0);
        let conf = if self.test_fns == 0 { 0.25 } else { 0.50 };
        (ratio, conf)
    }
}

impl<'ast> Visit<'ast> for TestVisitor {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if node.attrs.iter().any(|a| a.path().is_ident("test")) {
            self.test_fns += 1;
        } else {
            self.production_fns += 1;
        }
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        let name = node.path.segments.last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();
        if matches!(name.as_str(), "assert" | "assert_eq" | "assert_ne") {
            self.assert_count += 1;
        }
        syn::visit::visit_macro(self, node);
    }
}
