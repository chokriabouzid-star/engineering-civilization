#![forbid(unsafe_code)]

use syn::visit::Visit;

pub struct PerformanceVisitor {
    pub allocations: usize,
    pub clones: usize,
}

impl Default for PerformanceVisitor {
    fn default() -> Self { Self::new() }
}

impl PerformanceVisitor {
    pub fn new() -> Self {
        Self { allocations: 0, clones: 0 }
    }

    /// (performance_score, confidence)
    pub fn score(&self) -> (f64, f64) {
        let total = self.allocations + self.clones;
        let score = (1.0 - (total as f64 * 0.04).min(0.7)).clamp(0.3, 1.0);
        (score, 0.75)
    }
}

impl<'ast> Visit<'ast> for PerformanceVisitor {
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(ref path) = *node.func {
            let segments: Vec<String> = path.path.segments.iter()
                .map(|s| s.ident.to_string())
                .collect();
            let full = segments.join("::");

            match full.as_str() {
                "Vec::new" | "Vec::with_capacity" |
                "String::new" | "String::from" | "String::with_capacity" |
                "Box::new" | "HashMap::new" | "HashSet::new" |
                "BTreeMap::new" | "BTreeSet::new" |
                "Rc::new" | "Arc::new" => {
                    self.allocations += 1;
                }
                _ => {}
            }
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let method = node.method.to_string();
        match method.as_str() {
            "clone" | "to_string" | "to_owned" => {
                self.clones += 1;
            }
            _ => {}
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        let name = node.path.segments.last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();
        if name == "format" || name == "vec" {
            self.allocations += 1;
        }
        syn::visit::visit_macro(self, node);
    }
}
