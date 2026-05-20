use uuid::Uuid;

/// نتيجة ناجحة من التوليد
#[derive(Debug, Clone)]
pub struct GenerationSuccess {
    /// الكود المُولَّد
    pub code: String,
    /// اسم الـ template المستخدم
    pub template_name: &'static str,
    /// معرف فريد لهذه العملية
    pub generation_id: Uuid,
    /// عدد المحاولة (1-based)
    pub attempt_number: usize,
}

impl GenerationSuccess {
    /// إنشاء نتيجة ناجحة
    pub fn new(
        code: impl Into<String>,
        template_name: &'static str,
        attempt_number: usize,
    ) -> Self {
        Self {
            code: code.into(),
            template_name,
            generation_id: Uuid::new_v4(),
            attempt_number,
        }
    }

    /// هل الكود المُولَّد يحتوي على unsafe؟
    pub fn has_unsafe(&self) -> bool {
        self.code.contains("unsafe")
    }

    /// هل الكود المُولَّد يحتوي على اختبارات؟
    pub fn has_tests(&self) -> bool {
        self.code.contains("#[test]")
    }
}

/// نتيجة عملية التوليد
#[derive(Debug, Clone)]
pub enum GenerationResult {
    /// نجاح التوليد
    Success(GenerationSuccess),
    /// فشل التوليد مع سبب
    Failed {
        /// سبب الفشل
        reason: String,
    },
}

impl GenerationResult {
    /// هل نجح التوليد؟
    pub fn succeeded(&self) -> bool {
        matches!(self, Self::Success(_))
    }

    /// استخراج الكود إذا نجح
    pub fn code(&self) -> Option<&str> {
        match self {
            Self::Success(s) => Some(&s.code),
            _ => None,
        }
    }

    /// استخراج الـ success
    pub fn success(&self) -> Option<&GenerationSuccess> {
        match self {
            Self::Success(s) => Some(s),
            _ => None,
        }
    }
}
