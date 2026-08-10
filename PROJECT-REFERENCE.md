# Engineering Civilization — الوثيقة المرجعية

## v1.9.2 · Phase 2 CLOSED + compiler.rs gap fix + Phase 3.1 CLOSED · 2026-08-01

هذه الوثيقة مرجع حالة مضغوط ودقيق. لا تحتوي أرقامًا إلا إذا كانت:
1) متحققة مباشرة من تشغيل فعلي، أو
2) منقولة صراحةً من إصدار سابق متحقق، مع وسمها بأنها **موروثة** لا جديدة.

---

## ⚠️ حالة التحقق الحالية

### ما تحقق فعليًا في هذا التحديث (Phase 3.1)

| البند | الحالة | الدليل |
|---|---|---|
| `cargo build --workspace --locked` | ✅ | تشغيل فعلي ناجح |
| `cargo clippy --workspace --tests --locked -- -D warnings` | ✅ | تشغيل فعلي ناجح |
| `cargo test --workspace --locked` | ✅ | كل suites خضراء، **0 failed** |
| `cargo run --bin ec -- check .` | ✅ | **129 scanned / 115 passed / 14 failed / score=0.892** |
| تقاطع مستقل للنتيجة | ✅ | النتيجة `71/129 → 115/129` و`0.874 → 0.892` طابقت تشغيلًا مستقلًا في بيئة ثانية |

### ما هو موروث ومتحقَّق سابقًا من v1.9.1 (ولم يتغير في هذه المرحلة)

| البند | الحالة | المصدر |
|---|---|---|
| اختبارات `test-fast` | ✅ | **640 passed / 0 failed / 51 ignored** — v1.9.1 |
| اختبارات `ec-sandbox --features docker_tests` | ✅ | **141 passed / 0 failed / 1 ignored** — v1.9.1 |
| اختبارات `ec-app --features docker_tests` | ✅ | **101 passed / 0 failed / 1 ignored** — v1.9.1 |
| إجمالي الاختبارات المكتشفة | ✅ | **691** — v1.9.1 |
| تصنيف اختبارات Docker/slow | ✅ | **49 `docker_tests` + 2 `slow_tests`** — v1.9.1 |

### بيئة التحقق المرجعية

- محليًا: WSL2 Linux
- CI: GitHub Actions / ubuntu-24.04
- Rust toolchain المرجعي: `1.96.0`

---

## 1. نظرة عامة (الحقائق المتحققة فقط)

المشروع workspace من **11 crates** يطبق نموذج "حوكمة دستورية" على الكود:
- تقييم ساكن عبر `FitnessVector`
- تمثيل معرفي/بايزي للثقة
- تنفيذ معزول عبر Docker
- ذاكرة سببية append-only
- واجهات CLI وAPI فوق النواة

---

## 2. ما الجديد في v1.9.2 (Phase 3.1)

### خط الأساس قبل الإصلاح

من تشغيل فعلي لـ:

```bash
cargo run --bin ec -- check .
كانت النتيجة:

129 files scanned
71 passed
58 failed
project score = 0.874
الأنماط التي كُشفت
النمط	الوصف	الحالة
أ	test_coverage كان يُحسب على مستوى الملف فقط	✅ أُصلح
ب	ملفات tests/ وbenches/ كانت تُعاقَب بعتبتي test_coverage وreversibility	✅ أُصلح
ج	architectural_stability منخفض بسبب سوء تصنيف pub use المحلي في CouplingVisitor	⏸️ مؤجل عمدًا
د	#[tokio::test] ونظائرها لم تكن تُحسب كاختبارات	✅ أُصلح
التعديل المنفذ
الـcommit البرمجي:

359b6d7
fix(check): aggregate test_coverage per-crate, exempt test files from reversibility threshold
الملفات المعدلة:

crates/ec-analysis/src/report.rs
crates/ec-analysis/src/ast_analyzer.rs
crates/ec-analysis/src/visitors/test_visitor.rs
crates/ec-cli/src/check.rs
ما الذي تغير فعليًا
تجميع test_coverage على مستوى الـcrate داخل ec-cli/src/check.rs

ملفات src/ لم تعد تُقيَّم بعزل أعمى إذا كانت اختبارات الـcrate موجودة في tests/
إعفاء ملفات tests/ وbenches/ من فحص العتبتين فقط

test_coverage
reversibility
القيم ما زالت تُحسب وتظهر؛ الذي تغيّر هو احتساب الانتهاك فقط
إصلاح اكتشاف سمات الاختبار المؤهلة

#[test]
#[tokio::test]
#[async_std::test]
وغيرها مما ينتهي آخر segment فيه إلى test
مبدأ نقاء الـKernel بقي محترمًا

لا تغييرات تنتهك قيود نقاء الـKernel
لم يُضف I/O أو async إلى ec-analysis
النتيجة النهائية المتحققة
الحالة	Files scanned	Files passed	Files failed	Project score
قبل الإصلاح	129	71	58	0.874
بعد Phase 3.1	129	115	14	0.892
ما تبقى بعد الإصلاح
الـ14 انتهاكًا المتبقية هي:

13 × architectural_stability
./crates/ec-epistemic/src/lib.rs
./crates/ec-codegen/src/generator.rs
./crates/ec-constitutional/src/lib.rs
./crates/ec-constitutional/src/engine.rs
./crates/ec-analysis/src/visitors/mod.rs
./crates/ec-analysis/src/analyzer.rs
./crates/ec-api/src/handlers.rs
./crates/ec-sandbox/src/lib.rs
./crates/ec-sandbox/src/executor.rs
./crates/ec-memory/src/lib.rs
./crates/ec-memory/src/types.rs
./crates/ec-memory/src/query.rs
./crates/ec-memory/src/graph.rs
1 × reversibility
./crates/ec-cli/src/main.rs
هذه الأخيرة على ملف إنتاج حقيقي، وليست أثرًا جانبيًا من إصلاح Phase 3.1.

3. ADRs ذات الصلة
مبدأ نقاء الـKernel: الفصل بين الـKernel النقي وطبقة I/O (غير موثق كـADR مستقل بعد)
ADR-021: سياسة فصل docker_tests عن slow_tests
ADR-022: إصلاح النقاط العمياء في ec check
الأنماط أ + ب + د: منفذة
النمط ج: مؤجل بتشخيص مؤكد
4. الـ Crates — مرجع بنيوي مختصر
Crate	الدور
ec-fitness	تمثيل FitnessVector وPareto
ec-epistemic	الثقة/عدم اليقين والنمذجة البايزية
ec-constitutional	التقييم الدستوري
ec-sandbox	التنفيذ المعزول وRealityVector
ec-analysis	التحليل الساكن عبر AST
ec-memory	الذاكرة السببية append-only
ec-codegen	توليد الكود
ec-governance	الحوكمة والمقترحات والتدقيق
ec-api	REST API
ec-cli	واجهة سطر الأوامر
ec-app	تكامل النظام بالكامل
5. البنود المفتوحة الحالية
البند	الأولوية	المرحلة المقترحة
توثيق مبدأ نقاء الـKernel كـ ADR-023	متوسطة	Phase 3.2 وما بعدها
إصلاح النمط ج: CouplingVisitor يسيء تصنيف pub use المحلي	عالية	المرحلة التالية
عدم تشغيل ec check على sel-agent-v4 قبل إصلاح النمط ج	عالية	بعد إغلاق النمط ج فقط
2 unwrap() في ec-cli/src/main.rs	منخفضة	Phase 5
مراقبة gate_network_isolation عبر CI	منخفضة	مستمر
README / badge / عدّ ADRs الفعلي	منخفضة	لاحقًا
6. أوامر الصيانة
Bash

# البناء والفحص السريع
cargo build --workspace --locked
cargo clippy --workspace --tests --locked -- -D warnings
cargo fmt --all -- --check
cargo test --workspace --locked --no-fail-fast

# فحص المشروع نفسه
cargo run --bin ec -- check .

# اختبارات Docker
cargo test -p ec-sandbox --locked --features docker_tests -- --test-threads=1
cargo test -p ec-app --locked --features docker_tests -- --test-threads=1

# تحقق سياسة التصنيف
./phase2_verify.sh
نهاية الوثيقة المرجعية — Engineering Civilization v1.9.2 (2026-08-01)
