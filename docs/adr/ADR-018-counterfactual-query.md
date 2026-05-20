# ADR-018: Counterfactual Query System

**الحالة:** مقبول  
**التاريخ:** Week 23 — Phase 3  
**المؤلف:** Engineering Civilization Team  

---

## السياق

بعد Week 20 (ec-memory)، أصبح لدينا ذاكرة سببية append-only تُسجّل كل قرار
مع بدائله المرفوضة. لكن الذاكرة كانت **مكتوبة فقط** — لا يمكن الاستعلام عنها.

السؤال الذي لا نستطيع الإجابة عليه:
> "ماذا لو اخترنا البديل المرفوض؟"

هذا السؤال جوهري لنظام حضاري:
- بدون إجابة، لا نتعلم من أخطائنا
- بدون إجابة، لا نكتشف متى كان الدستور متحفظاً أكثر من اللازم
- بدون إجابة، الذاكرة مجرد أرشيف ميت

## القرار

إنشاء `MemoryQuery` — كائن استعلام **يقرأ فقط** الذاكرة السببية ويُجيب على
ثلاثة أنواع من الأسئلة:

### 1. Counterfactual Gain — "هل البديل كان أفضل؟"

```rust
pub fn counterfactual_gain(
    &self,
    chosen: &FitnessVector,
    alternative: &FitnessVector,
) -> CounterfactualGain
يستخدم Pareto ordering لأربع حالات:

AlternativeWasBetter — البديل يُسيطر باريتو (كان يجب اختياره)
ChoiceWasCorrect — الاختيار يُسيطر (القرار صحيح)
NoMeaningfulDifference — متساويان (لا فرق)
TradeoffDependent — non-dominated (يعتمد على المفاضلة)
2. Fitness Evolution — "كيف تطورت اللياقة؟"
Rust

pub fn fitness_evolution(
    &self,
    artifact_id: &str,
) -> Vec<FitnessSnapshot>
يتتبع الليافة عبر الـ iterations لـ artifact معين. يُجيب على:

هل نتحسن؟
هل وصلنا لهضبة؟
كم استغرق التحسن؟
3. Find Similar — "أي القرارات أشبه بهذا؟"
Rust

pub fn find_similar(
    &self,
    target: &FitnessVector,
    k: usize,
) -> Vec<SimilarDecision>
يستخدم cosine similarity بسيطة (بدون vector DB). يُجيب على:

هل رأينا مشكلة مشابهة من قبل؟
ماذا اخترنا حينها؟
هل نجح الاختيار؟
الخيارات المدروسة
خيار 1: SQL-like query language
Rust

memory.query("SELECT * WHERE security > 0.8 ORDER BY fitness")
مميزات: مرن، معروف للمطورين
عيوب: يكسر type safety، يحتاج parser، يُشجع استعلامات معقدة

خيار 2: Vector DB (approximate nearest neighbor)
Rust

memory.find_similar_embedding(embedding, k=5)
مميزات: سريع للبحث في آلاف القرارات
عيوب: overhead ضخم لـ MVP، يحتاج embedding model، non-deterministic

خيار 3: Typed query methods (الخيار المختار)
Rust

memory.counterfactual_gain(chosen, alt)
memory.fitness_evolution("artifact_id")
memory.find_similar(target, k)
مميزات: type-safe، بسيط، deterministic، لا dependencies خارجية
عيوب: أقل مرونة، O(n) للبحث (وليس O(log n))

لماذا cosine similarity وليس Euclidean distance?
Cosine similarity يقيس اتجاه المتجه وليس طوله. في سياق FitnessVector:

text

نفس النمط، شدة مختلفة:
  A = [0.9, 0.9, 0.9, 0.9, 0.9, 0.9]  ← كود ممتاز
  B = [0.6, 0.6, 0.6, 0.6, 0.6, 0.6]  ← كود متوسط

  cosine(A, B) = 1.0  ← نفس النمط!
  euclidean(A, B) = 0.73  ← "بعيدان"
نحن نهتم بـ "هل نفس الأبعاد قوية/ضعيفة؟" وليس "هل القيم متطابقة؟".
Cosine يُجيب على هذا السؤال بشكل أفضل.

Design Invariants
1. Read-only
MemoryQuery يحمل &CausalMemoryGraph (مرجع ثابت).
لا يمكنه تعديل الذاكرة بأي شكل.

Rust

pub struct MemoryQuery<'a> {
    graph: &'a CausalMemoryGraph,  // ثابت
}
2. No external dependencies
لا يعتمد على vector DB، ولا على مكتبة similarity خارجية.
كل الحسابات مُنفذة يدوياً — يمكن تدقيقها بالكامل.

3. Deterministic
نفس المدخلات → نفس المخرجات. لا عشوائية، لا تقريب.
هذا ضروري للاختبارات وللثقة في النتائج.

4. Graceful on empty
كل دالة تُرجع نتائج فارغة (وليس خطأ) عندما الذاكرة فارغة:

fitness_evolution("x") → vec![]
find_similar(target, 5) → vec![]
best_rejected_alternative(id) → None
التأثير على الأداء
العملية	التعقيد	ملاحظة
counterfactual_gain	O(1)	مقارنة متجهين فقط
fitness_evolution	O(n)	تصفية + enumerate
find_similar	O(n log n)	حساب + فرز
cosine_similarity	O(1)	6 عمليات ضرب
لـ n < 10,000 (أكثر من كافي لـ Phase 3): كل الاستعلامات < 1ms.

المخاطر والتخفيف
خطر 1: Cosine similarity يُضلّل عندما المتجهات قريبة من الصفر
تخفيف: cosine_similarity تُرجع 0.0 عندما أحد المتجهات صفري.
الاختبارات تتحقق من هذا.

خطر 2: Pareto ordering لا يلتقط المفاضلات
تخفيف: TradeoffDependent يعترف صراحة بأن البديل ليس أفضل ولا أسوأ.
القرار النهائي يعود للدستور.

خطر 3: find_similar يُرجع نتائج غير ذات صلة عندما k كبير
تخفيف: المستخدم يتحكم في k. الاختبارات تتحقق من أن النتائج مُرتبة
تنازلياً حسب similarity.

الاختبارات
23 اختبار في week23_gate.rs:

4 اختبارات fitness_evolution (فارغ، تتبع، تحسن، was_accepted)
6 اختبارات find_similar (at-most-k، مرتب، فارغ، k=0، k>len، حقول)
4 اختبارات cosine_similarity (متطابق، صفري، متماثل، ذاتي أعلى)
4 اختبارات counterfactual_gain (أفضل، صحيح، متساوي، مفاضلة)
3 اختبارات best_rejected_alternative (أفضل، بدون بدائل، عقدة مفقودة)
1 اختبار artifact filtering
1 اختبار gate نهائي
المراجع
ADR-015: Causal Memory Graph (Week 20)
ADR-016: Truth ≠ Fitness (Week 19)
ADR-013: Reality Feedback Loop (Week 15)
Pareto, V. (1896): Cours d'économie politique
Salton, G. (1983): Introduction to Modern Information Retrieval (cosine similarity)
