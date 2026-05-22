#![forbid(unsafe_code)]

use syn::visit::Visit;

#[derive(Debug, Clone)]
pub struct FnComplexity {
    pub name: String,
    pub cc: u32,
    pub is_test: bool,
}

pub struct ComplexityVisitor {
    pub functions: Vec<FnComplexity>,
    current_name: Option<String>,
    current_decisions: u32,
    current_is_test: bool,
}

impl Default for ComplexityVisitor {
    fn default() -> Self { Self::new() }
}

impl ComplexityVisitor {
    pub fn new() -> Self {
        Self {
            functions: vec![],
            current_name: None,
            current_decisions: 0,
            current_is_test: false,
        }
    }

    /// (maintainability_score, confidence)
    pub fn score(&self) -> (f64, f64) {
        let prod: Vec<_> = self.functions.iter()
            .filter(|f| !f.is_test)
            .collect();

        if prod.is_empty() {
            return (1.0, 0.80);
        }

        let avg = prod.iter().map(|f| f.cc as f64).sum::<f64>()
            / prod.len() as f64;
        let max = prod.iter().map(|f| f.cc).max().unwrap_or(1);

        let penalty = match max {
            31..=u32::MAX => 0.25,
            21..=30       => 0.15,
            _             => 0.0,
        };

        let score = ((1.0 - (avg - 1.0) / 19.0).clamp(0.0, 1.0) - penalty)
            .max(0.0);
        (score, 0.88)
    }

    pub fn high_complexity(&self) -> Vec<(String, u32)> {
        self.functions.iter()
            .filter(|f| f.cc > 10)
            .map(|f| (f.name.clone(), f.cc))
            .collect()
    }
}

impl<'ast> Visit<'ast> for ComplexityVisitor {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let saved_name      = self.current_name.take();
        let saved_decisions = self.current_decisions;
        let saved_is_test   = self.current_is_test;

        self.current_name      = Some(node.sig.ident.to_string());
        self.current_decisions = 0;
        self.current_is_test   = node.attrs.iter()
            .any(|a| a.path().is_ident("test"));

        syn::visit::visit_item_fn(self, node);

        if let Some(name) = self.current_name.take() {
            self.functions.push(FnComplexity {
                name,
                cc: self.current_decisions + 1,
                is_test: self.current_is_test,
            });
        }

        self.current_name      = saved_name;
        self.current_decisions = saved_decisions;
        self.current_is_test   = saved_is_test;
    }

    fn visit_expr_if(&mut self, n: &'ast syn::ExprIf) {
        self.current_decisions += 1;
        syn::visit::visit_expr_if(self, n);
    }
    fn visit_expr_while(&mut self, n: &'ast syn::ExprWhile) {
        self.current_decisions += 1;
        syn::visit::visit_expr_while(self, n);
    }
    fn visit_expr_for_loop(&mut self, n: &'ast syn::ExprForLoop) {
        self.current_decisions += 1;
        syn::visit::visit_expr_for_loop(self, n);
    }
    fn visit_arm(&mut self, n: &'ast syn::Arm) {
        if !matches!(n.pat, syn::Pat::Wild(_)) {
            self.current_decisions += 1;
        }
        syn::visit::visit_arm(self, n);
    }
    fn visit_expr_binary(&mut self, n: &'ast syn::ExprBinary) {
        if matches!(n.op, syn::BinOp::And(_) | syn::BinOp::Or(_)) {
            self.current_decisions += 1;
        }
        syn::visit::visit_expr_binary(self, n);
    }
    fn visit_expr_try(&mut self, n: &'ast syn::ExprTry) {
        self.current_decisions += 1;
        syn::visit::visit_expr_try(self, n);
    }
}
