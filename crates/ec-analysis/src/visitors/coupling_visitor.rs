#![forbid(unsafe_code)]

use syn::visit::Visit;

pub struct CouplingVisitor {
    pub external_uses: usize,
    pub std_uses: usize,
}

impl Default for CouplingVisitor {
    fn default() -> Self { Self::new() }
}

impl CouplingVisitor {
    pub fn new() -> Self {
        Self { external_uses: 0, std_uses: 0 }
    }

    /// (architectural_stability_score, confidence)
    pub fn score(&self) -> (f64, f64) {
        let weighted = (self.external_uses as f64 * 0.12
            + self.std_uses as f64 * 0.03)
            .min(1.0);
        ((1.0 - weighted).clamp(0.0, 1.0), 0.75)
    }
}

impl<'ast> Visit<'ast> for CouplingVisitor {
    fn visit_use_path(&mut self, node: &'ast syn::UsePath) {
        let root = node.ident.to_string();
        match root.as_str() {
            "std" | "core" | "alloc" => self.std_uses += 1,
            _ => self.external_uses += 1,
        }
        syn::visit::visit_use_path(self, node);
    }
}
