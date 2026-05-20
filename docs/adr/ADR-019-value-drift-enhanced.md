# ADR-019: Value Drift Detection from Historical Memory

**الحالة:** مقبول — Week 24  
**السياق:** Week 24 — Phase 3

---

## المشكلة

`ValueDriftDetector` في Week 6 يعمل على زاويتين فقط (before/after).
لا يستطيع كشف الانجراف **عبر سلسلة طويلة من القرارات**.

السؤال: كيف نعرف إذا كانت قيم النظام تنحرف تدريجياً؟

---

## القرار

إضافة `HistoricalDriftAnalyzer` في `ec-memory/src/drift.rs`:
CausalMemoryGraph → [baseline: أول N] vs [current: آخر M]
→ average FitnessVector لكل نافذة
→ cosine angle بين المتوسطين
→ DriftClassification + DriftAction

text


---

## التصنيفات الأربعة

| التصنيف | الشرط | الإجراء |
|---------|-------|---------|
| Stable | angle < 10° | None |
| LearningProgress | angle ≥ 10° + Pareto تحسّن | Monitor |
| ValueShift | angle ≥ 10° بدون تحسن | ReviewConstitution / HumanIntervention |
| Corruption | rejection_increase > 20% | HumanIntervention |

---

## الفرق عن ADR-005 (ValueDriftDetector)

| ADR-005 | ADR-019 |
|---------|---------|
| زاويتان فقط | سلسلة كاملة (baseline vs current) |
| يُغذّى يدوياً | يقرأ من CausalMemoryGraph مباشرة |
| لا يعرف معدل الرفض | يحسب rejection_increase |
| لا يُميّز تعلم من فساد | LearningProgress vs Corruption |

---

## الضمانات

1. `HistoricalDriftAnalyzer` يحمل `&CausalMemoryGraph` — قراءة فقط
2. لا يُعدّل الذاكرة (append-only محفوظ)
3. `InsufficientData` عند بيانات غير كافية — لا panic
4. الزاوية دائماً في [0°, 180°]

---

## البدائل المرفوضة

- **تعديل ValueDriftDetector**: رفض — لا نُعدّل ما يعمل
- **إضافة لـ ec-constitutional**: رفض — الذاكرة في ec-memory
- **Statistical drift (CUSUM)**: مستقبل — يحتاج بيانات أكثر

---

## الاختبارات

- `t01`: InsufficientData عند memory صغيرة
- `t02`: Stable عند vectors متطابقة
- `t03`: ValueShift عند تحول الأولويات
- `t04`: Corruption عند ارتفاع الرفض
- `t05`: الأعداد صحيحة في التقرير
- `t06`: الزاوية في [0, 180]
- `t07`: HumanIntervention عند > 45°
- `t08`: Stable لا يحتاج إجراء
- `t09`: Empty memory → InsufficientData
- `t10`: Gate الكامل

---

*Engineering Civilization — Week 24*
