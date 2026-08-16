#![forbid(unsafe_code)]

use std::collections::HashSet;
use syn::visit::Visit;

/// يقيس التماسك المعماري عبر عدّ استيرادات الملف.
///
/// # سياسة العدّ
/// - `std/core/alloc` → `std_uses` (عقوبة خفيفة).
/// - جذر يبدأ بـ`ec_` (workspace) → متجاهل.
/// - `crate::` / `self::` / `super::` → متجاهل (مسار داخلي).
/// - `pub use X::...` حيث `X` موديول محلي معلَن في نفس الملف → متجاهل.
/// - غير ذلك → `external_uses` (عقوبة كاملة).
#[derive(Default)]
pub struct CouplingVisitor {
    pub external_uses: usize,
    pub std_uses: usize,
    local_mods: HashSet<String>,
}

impl CouplingVisitor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn score(&self) -> (f64, f64) {
        let weighted = (self.external_uses as f64 * 0.12 + self.std_uses as f64 * 0.03).min(1.0);
        ((1.0 - weighted).clamp(0.0, 1.0), 0.75)
    }
}

impl<'ast> Visit<'ast> for CouplingVisitor {
    /// تمريرة أولى حقيقية: نجمع كل أسماء `mod`/`pub mod` في الملف
    /// بغض النظر عن موضعها النصي، ثم نكمل الزيارة القياسية.
    /// بدون هذا التجاوز، `pub use X::_` الذي يسبق `mod X;` نصيًا
    /// يُصنَّف خطأً كتبعية خارجية.
    fn visit_file(&mut self, file: &'ast syn::File) {
        for item in &file.items {
            if let syn::Item::Mod(m) = item {
                self.local_mods.insert(m.ident.to_string());
            }
        }
        syn::visit::visit_file(self, file);
    }

    /// تغطية إضافية آمنة للموديولات المتداخلة.
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        self.local_mods.insert(node.ident.to_string());
        syn::visit::visit_item_mod(self, node);
    }

    /// تصنيف كل `use` مرة واحدة بمعرفة `vis`.
    fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
        let root = match extract_root(&node.tree) {
            Some(r) => r,
            None => return,
        };

        match root.as_str() {
            "crate" | "self" | "super" => return,
            _ => {}
        }

        let is_public = matches!(node.vis, syn::Visibility::Public(_));
        if is_public && self.local_mods.contains(&root) {
            return;
        }

        match root.as_str() {
            "std" | "core" | "alloc" => self.std_uses += 1,
            s if s.starts_with("ec_") => {}
            _ => self.external_uses += 1,
        }
    }
}

/// استخراج الجذر (أول segment) من `syn::UseTree`.
fn extract_root(tree: &syn::UseTree) -> Option<String> {
    match tree {
        syn::UseTree::Path(p) => Some(p.ident.to_string()),
        syn::UseTree::Name(n) => Some(n.ident.to_string()),
        syn::UseTree::Rename(r) => Some(r.ident.to_string()),
        syn::UseTree::Glob(_) => None,
        syn::UseTree::Group(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn analyze(code: &str) -> CouplingVisitor {
        let file = syn::parse_str::<syn::File>(code).expect("كود Rust صالح");
        let mut v = CouplingVisitor::new();
        v.visit_file(&file);
        v
    }

    #[test]
    fn pub_use_of_local_mod_no_penalty() {
        let code = r#"
            pub mod bayesian;
            pub use bayesian::BayesianEvidence;
        "#;
        let v = analyze(code);
        assert_eq!(v.external_uses, 0, "pub use محلي لا يُعاقَب");
        assert_eq!(v.std_uses, 0);
    }

    #[test]
    fn pub_use_of_local_mod_glob_no_penalty() {
        let code = r#"
            pub mod state;
            pub use state::*;
        "#;
        let v = analyze(code);
        assert_eq!(v.external_uses, 0, "pub use * محلي لا يُعاقَب");
    }

    #[test]
    fn plain_use_of_external_still_penalized() {
        let code = r#"
            use serde::Serialize;
        "#;
        let v = analyze(code);
        assert_eq!(v.external_uses, 1, "use خارجي عادي يُعاقَب");
    }

    #[test]
    fn pub_use_of_external_still_penalized() {
        let code = r#"
            pub use serde::Serialize;
        "#;
        let v = analyze(code);
        assert_eq!(v.external_uses, 1, "pub use خارجي يُعاقَب");
    }

    #[test]
    fn std_uses_counted_separately() {
        let code = r#"
            use std::collections::HashMap;
            use std::fmt;
        "#;
        let v = analyze(code);
        assert_eq!(v.std_uses, 2, "استيرادات std تذهب إلى std_uses");
        assert_eq!(v.external_uses, 0);
    }

    #[test]
    fn ec_workspace_uses_ignored() {
        let code = r#"
            use ec_fitness::FitnessVector;
            use ec_epistemic::EpistemicState;
        "#;
        let v = analyze(code);
        assert_eq!(v.external_uses, 0, "استيرادات workspace لا عقوبة");
        assert_eq!(v.std_uses, 0);
    }

    #[test]
    fn mixed_case_epistemic_pattern() {
        let code = r#"
            pub mod bayesian;
            pub mod calibration;
            pub mod decay;
            pub mod error;
            pub mod propagation;
            pub mod state;
            pub mod bayesian_calibration;
            pub use bayesian::BayesianEvidence;
            pub use calibration::CalibrationAdjustment;
            pub use decay::TemporalDecay;
            pub use error::EpistemicError;
            pub use propagation::UncertaintyPropagation;
            pub use state::EpistemicState;
            pub use bayesian_calibration::BayesianCalibration;
        "#;
        let v = analyze(code);
        assert_eq!(v.external_uses, 0, "كل pub use محلية في نمط lib.rs");
        assert_eq!(v.std_uses, 0);
        let (score, _) = v.score();
        assert!(
            (score - 1.0).abs() < 1e-9,
            "score يجب أن يكون 1.0 لملف بلا تبعيات خارجية"
        );
    }

    #[test]
    fn regression_baseline_from_epistemic_lib() {
        let code = r#"
            pub mod state;
            pub use state::EpistemicState;
            pub mod error;
            pub use error::EpistemicError;
            pub mod bayesian;
            pub use bayesian::BayesianEvidence;
            pub mod decay;
            pub use decay::TemporalDecay;
            pub mod propagation;
            pub use propagation::UncertaintyPropagation;
            pub mod calibration;
            pub use calibration::CalibrationAdjustment;
            pub mod bayesian_calibration;
            pub use bayesian_calibration::BayesianCalibration;
        "#;
        let v = analyze(code);
        assert_eq!(v.external_uses, 0);
        let (score, _) = v.score();
        assert!(
            (score - 1.0).abs() < 1e-9,
            "انحدار: كان يُعطي 0.16 قبل الإصلاح"
        );
    }

    #[test]
    fn crate_path_no_penalty() {
        let code = r#"
            use crate::complexity::ComplexityMetrics;
            use crate::coverage::CoverageMetrics;
            use crate::metrics::count_pattern;
            use crate::reversibility::ReversibilityMetrics;
            use crate::security::SecurityMetrics;
        "#;
        let v = analyze(code);
        assert_eq!(v.external_uses, 0, "use crate::_ مسار داخلي — لا عقوبة");
    }

    #[test]
    fn super_in_test_mod_no_penalty() {
        let code = r#"
            fn production_fn() {}

            #[cfg(test)]
            mod tests {
                use super::*;

                #[test]
                fn it_works() {}
            }
        "#;
        let v = analyze(code);
        assert_eq!(v.external_uses, 0, "use super::* داخل mod tests — لا عقوبة");
    }

    #[test]
    fn pub_use_before_mod_declaration_still_exempted() {
        let code = r#"
            pub use bayesian::BayesianEvidence;
            pub mod bayesian;
        "#;
        let v = analyze(code);
        assert_eq!(
            v.external_uses, 0,
            "pub use قبل mod يجب ألا يُغيّر النتيجة — \
             إن فشل هذا، التصميم أحادي التمريرة لا تمريرتين"
        );
    }
}
