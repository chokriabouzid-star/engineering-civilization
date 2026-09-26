# ADR-001: Fitness Vector

> ملاحظة توثيقية (2026-09-26): هذا الملف كان بعنوان "Self-Check: Week 1" وهو المحتوى التأسيسي لمفهوم FitnessVector وPareto. أُضيف عنوان ADR-001 لمطابقة اسم الملف؛ المحتوى الأصلي محفوظ كما هو أدناه.

## Self-Check: Week 1 (المحتوى الأصلي)

## Q1: هل A يهيمن على B؟

A = (security=0.9, changeability=0.3, ...)
B = (security=0.7, changeability=0.8, ...)

الإجابة: لا، A لا يهيمن على B.

السبب:
في Pareto Dominance يجب أن يكون A:
- أفضل أو مساوياً لـ B في جميع الأبعاد
- وأفضل منه بشكل صارم في بُعد واحد على الأقل

هنا:
- A أفضل في security
- لكن B أفضل في changeability

وبالتالي شرط dominance يفشل.
العلاقتان هما:
- ¬(A ≻ B)
- ¬(B ≻ A)

أي أن العنصرين incomparable ويقعان معاً على Pareto Frontier.

---

## Q2: الفرق بين Dominance و Frontier

### Dominance
هي علاقة مقارنة ثنائية بين عنصرين.

التعريف:

A ≻ B iff:
- A ≥ B في كل dimensions
- و A > B في بُعد واحد على الأقل

مثال:
A يهيمن على C.

---

### Pareto Frontier
هي مجموعة جميع العناصر التي لا يهيمن عليها أي عنصر آخر.

التعريف:

P(S) = {
    x ∈ S :
    ¬∃y ∈ S such that y ≻ x
}

أي:
أفضل الحلول غير القابلة للتحسين دون تضحية.

---

الفرق الجوهري:

- Dominance = relation
- Frontier = set of non-dominated solutions

---

## Q3: لماذا Option<CatastrophicDimension> وليس bool؟

لأن bool يجيب فقط:

- هل يوجد failure؟
  true / false

لكن النظام يحتاج معرفة:
- أي dimension تحديداً فشل؟

لذلك:

```rust
Option<CatastrophicDimension>
