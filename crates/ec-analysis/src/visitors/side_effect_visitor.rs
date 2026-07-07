#![forbid(unsafe_code)]

use syn::visit::Visit;

pub struct SideEffectVisitor {
    pub stdout_writes: usize,
    pub static_muts: usize,
    pub io_calls: usize,
}

impl Default for SideEffectVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl SideEffectVisitor {
    pub fn new() -> Self {
        Self {
            stdout_writes: 0,
            static_muts: 0,
            io_calls: 0,
        }
    }

    /// (reversibility_score, confidence)
    pub fn score(&self) -> (f64, f64) {
        let total = self.stdout_writes + self.static_muts * 2 + self.io_calls;
        ((1.0 - (total as f64 * 0.12).min(1.0)).max(0.0), 0.70)
    }
}

impl<'ast> Visit<'ast> for SideEffectVisitor {
    fn visit_item_static(&mut self, node: &'ast syn::ItemStatic) {
        if matches!(node.mutability, syn::StaticMutability::Mut(_)) {
            self.static_muts += 1;
        }
        syn::visit::visit_item_static(self, node);
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        let name = node
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();
        match name.as_str() {
            "println" | "print" | "eprintln" | "eprint" => {
                self.stdout_writes += 1;
            }
            _ => {}
        }
        syn::visit::visit_macro(self, node);
    }
}
