use serde::{Deserialize, Serialize};

/// سياق محاولة فاشلة سابقة للتوليد التكيفي
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FailureContext {
    /// سبب الفشل
    pub reason: String,
    /// درجة الأمان السابقة
    pub security_score: f64,
    /// درجة التغطية السابقة
    pub coverage_score: f64,
}

/// مواصفات الكود المطلوب
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenerationSpec {
    /// اسم الدالة
    pub function_name: String,
    /// أنواع المدخلات
    pub input_types: Vec<String>,
    /// نوع المخرجات
    pub output_type: String,
    /// وصف نصي للوظيفة
    pub description: String,
    /// قيود إضافية
    pub constraints: Vec<String>,
    /// هل نُولّد اختبارات؟
    pub include_tests: bool,
    /// سياق محاولات سابقة
    pub previous_failures: Vec<FailureContext>,
}

impl GenerationSpec {
    /// إنشاء مواصفات بسيطة بدون وصف
    pub fn simple(
        function_name: impl Into<String>,
        input_types: Vec<&str>,
        output_type: impl Into<String>,
    ) -> Self {
        Self {
            function_name: function_name.into(),
            input_types: input_types.into_iter().map(|s| s.to_string()).collect(),
            output_type: output_type.into(),
            description: String::new(),
            constraints: Vec::new(),
            include_tests: true,
            previous_failures: Vec::new(),
        }
    }

    /// إضافة وصف
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// إضافة فشل سابق للتعلم منه
    pub fn with_failure(mut self, failure: FailureContext) -> Self {
        self.previous_failures.push(failure);
        self
    }

    /// هل هذه أول محاولة؟
    pub fn is_first_attempt(&self) -> bool {
        self.previous_failures.is_empty()
    }

    /// رقم المحاولة الحالية
    pub fn attempt_number(&self) -> usize {
        self.previous_failures.len() + 1
    }

    /// تنسيق المعاملات: `a: i32, b: String`
    pub fn format_params(&self) -> String {
        let letters: Vec<&str> = "abcdefghijklmnopqrstuvwxyz".split("").filter(|s| !s.is_empty()).collect();
        self.input_types
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let name = letters.get(i).copied().unwrap_or("x");
                format!("{}: {}", name, t)
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// هل المواصفات تتطلب unsafe؟
    pub fn requires_unsafe(&self) -> bool {
        self.constraints.iter().any(|c| c.contains("unsafe") || c.contains("raw_ptr"))
    }
}
