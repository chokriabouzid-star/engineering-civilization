# ADR-022: إصلاح النقاط العمياء الناتجة عن التحليل المعزول لكل ملف (`ec check`)

**الحالة:** مُنفَّذ ومُتحقَّق منه فعليًا (الأنماط أ، ب، د) — النمط ج مؤجَّل عمدًا بتشخيص كامل
**التاريخ:** 2026-08-01
**السياق:** Phase 3.1 — بعد تشغيل خط أساس فعلي (`ec check .`): 129 ملفًا، 71 نجح، 58 فشل، score=0.874.
**الـcommit:** 359b6d7

---

## 1. المشكلة (بالدليل من الكود والتشغيل الفعلي)

فحص قائمة الانتهاكات الـ58 وقراءة الكود المصدري كشفا **أربعة أنماط منفصلة**:

### النمط أ: `test_coverage=0.00` على ملفات مصدر منظَّمة
**السبب المؤكَّد من `test_visitor.rs`:** `TestVisitor::visit_item_fn` يحسب
`test_fns`/`production_fns` داخل الملف الواحد فقط. المشروع يضع اختباراته
في `tests/weekN_gate.rs` منفصلة — نمط شائع في Rust — فتُحسَب كل ملفات
`src/` "بلا تغطية" رغم أنها مغطاة على مستوى الـcrate.

### النمط ب: `reversibility` منخفضة على ملفات الاختبار
**السبب المؤكَّد من `side_effect_visitor.rs`:** هذا المسار يعاقب
`println!`/`print!`/`eprintln!`/`eprint!` و`static mut`. الحقل `io_calls`
موجود لكنه غير مُفعَّل في هذا visitor الحالي. ضجيج الطباعة التشخيصية
الطبيعي في كود الاختبار كان يُسقط الملفات دون أي خطر حقيقي.

### النمط ج: `architectural_stability` منخفضة على بعض ملفات `lib.rs` — **مؤجَّل عمدًا**
**السبب مؤكَّد بمطابقة حسابية:** `CouplingVisitor` يُصنِّف كل `pub use`
بجذر محلي (مثل `pub use bayesian::BayesianEvidence`) كـ"external coupling"
لأن الجذر (`bayesian`) ليس `std`/`core`/`alloc` ولا يبدأ بـ`ec_`.

مثال: `ec-epistemic/src/lib.rs` — 7 أسطر `pub use` محلية × 0.12 = 0.84
→ score = 1.0 - 0.84 = **0.16** — يطابق المخرج الفعلي حرفيًا.

**مؤجَّل:** يحتاج تعديلًا داخل `ec-analysis` (تمييز `pub use` إعادة تصدير
عن `use` تبعية خارجية حقيقية) — تغيير يستحق مراجعة مستقلة لا دمجًا هنا.

### النمط د: سمات الاختبار المؤهَّلة غير مُتعرَّف عليها — **اكتُشف أثناء التنفيذ**
**السبب:** `visit_item_fn` كان يتحقق بـ`a.path().is_ident("test")` — يتطلب
مسارًا بمقطع واحد. `#[tokio::test]` مساره بمقطعين فيفشل التطابق،
وتُصنَّف اختبارات `ec-api` الأربعة عشر خطأً كدوال إنتاجية.

---

## 2. القرار

**نطاق Phase 3.1:** الأنماط أ + ب + د — منفَّذة في commit واحد.
**النمط ج:** مؤجَّل — سببه مؤكَّد، تصميم حله يستحق ADR مستقل.

**ما لا يدخل النطاق (بقرار سابق):**
- v1 (`analyzer.rs::analyze_code` في `pipeline.rs`) — Phase 3.4
- `ec analyze <ملف واحد>` — يبقى بمنطق الملف الواحد، طلب صريح من المستخدم

---

## 3. التصميم التقني المُنفَّذ

### 3.1 `ec-analysis/src/report.rs`
إضافة حقلين لـ`AnalysisReport`:
```rust
pub test_fns: usize,
pub production_fns: usize,
منقولان من TestVisitor (كانا محسوبين ومُهدَرين بعد score()).
AnalysisReport::unparseable() يُعيدهما صفرًا.

3.2 ec-analysis/src/ast_analyzer.rs
سطران فقط عند بناء AnalysisReport:

Rust

test_fns: test_v.test_fns,
production_fns: test_v.production_fns,
3.3 ec-analysis/src/visitors/test_visitor.rs — النمط د
Rust

// قبل:
if node.attrs.iter().any(|a| a.path().is_ident("test")) {

// بعد:
if node.attrs.iter().any(|a| {
    a.path()
        .segments
        .last()
        .map(|s| s.ident == "test")
        .unwrap_or(false)
}) {
يلتقط #[test]، #[tokio::test]، #[async_std::test]، إلخ.

3.4 ec-cli/src/check.rs — التغيير الأكبر
إعادة هيكلة check_workspace إلى تمريرتين:

تمريرة 1 (collect_files):

مسح متكرر ينتج Vec<ScannedFile>
كل ملف يحمل: path, crate_root (أقرب Cargo.toml أب),
is_test_or_bench, AnalysisReport
تجميع بين التمريرتين:

Rust

if production_fns == 0 { 1.0 }
else { (test_fns as f64 / production_fns as f64).min(1.0) }
تمريرة 2:

ملفات tests//benches/: تبقى قيمة test_coverage المحلية كما هي،
وتُستثنى هي وreversibility من فحص العتبات (لا من الحساب/العرض)
ملفات src/: تُستبدل test_coverage بنسبة الـcrate قبل فحص العتبات
4. النتيجة المُتحقَّق منها
المرحلة	passed	failed	score
خط الأساس	71/129	58	0.874
Phase 3.1 (هذا الـcommit)	115/129	14	0.892
الـ14 المتبقية:

13 × architectural_stability — النمط ج (مؤجَّل عمدًا)
1 × reversibility على ec-cli/src/main.rs — فحص شرعي على ملف إنتاج
التحقق:

cargo build --workspace --locked: ناجح
cargo clippy --workspace --locked -- -D warnings: ناجح
cargo test --workspace --locked: 640 passed / 0 failed / 51 ignored
ADR-020 محترم: لا تغييرات تنتهك قيود نقاء الـKernel (لا I/O ولا async أُضيفا)
5. البدائل المرفوضة
خفض عتبة reversibility لملفات الاختبار بدل الإعفاء: مرفوض —
السبب ضجيج طباعة، لا خطر فعلي، فأي عتبة مسموحة سيكون سؤالًا بلا إجابة هندسية.
دمج النمط ج في هذا الباتش: مرفوض — يحتاج تعديلًا داخل kernel
ec-analysis يستحق مراجعة مستقلة.
نقل منطق تجميع الـcrate إلى ec-analysis: مرفوض — ADR-020 يحصر
معرفة نظام الملفات وI/O في ec-cli.
