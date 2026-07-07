#![forbid(unsafe_code)]

use syn::visit::Visit;

#[derive(Debug, Clone)]
pub struct UnsafeBlock {
    pub kind: UnsafeKind,
    pub has_safety_comment: bool,
}

#[derive(Debug, Clone)]
pub enum UnsafeKind {
    ExprBlock,
    UnsafeFn,
    UnsafeImpl,
    UnsafeTrait,
}

impl UnsafeKind {
    pub fn risk(&self) -> f64 {
        match self {
            Self::ExprBlock => 0.40,
            Self::UnsafeFn => 0.30,
            Self::UnsafeImpl => 0.20,
            Self::UnsafeTrait => 0.15,
        }
    }
}

fn has_doc_comment(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| a.path().is_ident("doc"))
}

pub struct UnsafeVisitor {
    pub blocks: Vec<UnsafeBlock>,
}

impl Default for UnsafeVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl UnsafeVisitor {
    pub fn new() -> Self {
        Self { blocks: vec![] }
    }

    /// (security_score, confidence)
    ///
    /// divisor = 2.0:
    /// - 1 unsafe block (0.40) → 0.80
    /// - 1 unsafe fn   (0.30) → 0.85
    /// - 3 unsafe blocks      → 0.40
    /// - doc comment يُخفّض risk بنسبة 40%
    pub fn score(&self) -> (f64, f64) {
        if self.blocks.is_empty() {
            return (1.0, 0.95);
        }
        let risk: f64 = self
            .blocks
            .iter()
            .map(|b| {
                if b.has_safety_comment {
                    b.kind.risk() * 0.60
                } else {
                    b.kind.risk()
                }
            })
            .sum();
        let score = (1.0 - (risk / 2.0).min(1.0)).max(0.0);
        (score, 0.90)
    }

    pub fn unjustified_count(&self) -> usize {
        self.blocks.iter().filter(|b| !b.has_safety_comment).count()
    }
}

impl<'ast> Visit<'ast> for UnsafeVisitor {
    fn visit_expr_unsafe(&mut self, node: &'ast syn::ExprUnsafe) {
        self.blocks.push(UnsafeBlock {
            kind: UnsafeKind::ExprBlock,
            has_safety_comment: false,
        });
        syn::visit::visit_expr_unsafe(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if node.sig.unsafety.is_some() {
            self.blocks.push(UnsafeBlock {
                kind: UnsafeKind::UnsafeFn,
                has_safety_comment: has_doc_comment(&node.attrs),
            });
        }
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        if node.unsafety.is_some() {
            self.blocks.push(UnsafeBlock {
                kind: UnsafeKind::UnsafeImpl,
                has_safety_comment: has_doc_comment(&node.attrs),
            });
        }
        syn::visit::visit_item_impl(self, node);
    }

    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
        if node.unsafety.is_some() {
            self.blocks.push(UnsafeBlock {
                kind: UnsafeKind::UnsafeTrait,
                has_safety_comment: has_doc_comment(&node.attrs),
            });
        }
        syn::visit::visit_item_trait(self, node);
    }
}
