# Engineering Civilization — الوثيقة المرجعية

## v1.9.3 · Phase 3.2 CLOSED · 2026-08-16

هذه الوثيقة مرجع حالة مضغوط ودقيق. لا تحتوي أرقامًا إلا إذا كانت:
1) متحققة مباشرة من تشغيل فعلي، أو
2) منقولة صراحةً من إصدار سابق متحقق، مع وسمها بأنها **موروثة** لا جديدة.

---

## ⚠️ حالة التحقق الحالية

### ما تحقق فعليًا في هذا التحديث (Phase 3.2)

| البند | الحالة | الدليل |
|---|---|---|
| `cargo build --workspace --locked` | ✅ | تشغيل فعلي ناجح (بيئتين مستقلتين) |
| `cargo clippy --workspace --tests --locked -- -D warnings` | ✅ | صفر تحذيرات |
| `cargo fmt --all -- --check` | ✅ | نظيف |
| `cargo test -p ec-analysis --locked coupling_visitor` | ✅ | **11 passed / 0 failed** |
| `cargo test --workspace --locked --no-fail-fast` | ✅ | **651 passed / 0 failed / 51 ignored** (كانت 640/0/51) |
| `cargo run --bin ec -- check .` | ✅ | **129 scanned / 127 passed / 2 failed / score=0.915** (كانت 115/14/0.892) |
| commit | ✅ | `706b978` على `master`، بيئة chokri (rustc/cargo 1.96.0) |
| CI | ✅ | GitHub Actions Run #14 · **Success · 2m 56s · 4/4 jobs** |
| تقاطع مستقل للنتيجة | ✅ | مطابقة حرفية بين بيئة chokri (1.96.0) وبيئة مستقلة ثانية (1.91.1) |

### ما هو موروث ومتحقَّق سابقًا من v1.9.2 (لم يتغيّر في هذه المرحلة)

| البند | الحالة | المصدر |
|---|---|---|
| اختبارات `ec-sandbox --features docker_tests` | ✅ | **141 passed / 0 failed / 1 ignored** — v1.9.1، غير مرتبط بهذه المرحلة |
| اختبارات `ec-app --features docker_tests` | ✅ | **101 passed / 0 failed / 1 ignored** — v1.9.1، غير مرتبط بهذه المرحلة |
| إجمالي الاختبارات المكتشفة | ⚠️ | **691** — v1.9.1، لم يُعَد احتسابه بهذا المنهج تحديدًا في Phase 3.2 |
| تصنيف اختبارات Docker/slow | ✅ | **49 `docker_tests` + 2 `slow_tests`** — v1.9.1 |

### بيئة التحقق المرجعية

- محليًا: WSL2 Linux، `rustc/cargo 1.96.0`
- CI: GitHub Actions / ubuntu-24.04
- Rust toolchain المرجعي: `1.96.0` (مثبَّت في `rust-toolchain.toml`)

---

## 1. نظرة عامة (الحقائق المتحققة فقط)

المشروع workspace من **11 crates** يطبق نموذج "حوكمة دستورية" على الكود:
- تقييم ساكن عبر `FitnessVector`
- تمثيل معرفي/بايزي للثقة
- تنفيذ معزول عبر Docker
- ذاكرة سببية append-only
- واجهات CLI وAPI فوق النواة

---

## 2. Phase 3.1 (v1.9.2 — تاريخي، بلا تغيير)

خط الأساس: 129 ملفًا، 71 نجح، 58 فشل، score=0.874. أُصلحت الأنماط أ+ب+د
(commit `359b6d7`) — النمط ج شُخِّص وأُجِّل عمدًا (ADR-022). النتيجة:
129/115/14/0.892. تفاصيل كاملة في `docs/adr/ADR-022-check-blind-spots.md`.

---

## 3. ما الجديد في v1.9.3 (Phase 3.2)

استُكمل النمط ج المؤجَّل من ADR-022، واكتُشف أثناء التنفيذ نمط إضافي
(النمط هـ: `crate::`/`self::`/`super::`) ودُمج في نفس الإصلاح بعد تحقق
تجريبي أن الدمج لا يُدخل أي انحدار. تفاصيل التشخيص الكامل، البدائل
المرفوضة، والتحقق المضاد في **`docs/adr/ADR-023-kernel-purity-and-coupling-fix.md`**.

**الـcommit:** `706b978` — `fix(analysis): resolve same-crate misclassification in CouplingVisitor`
**الملف المعدَّل:** `crates/ec-analysis/src/visitors/coupling_visitor.rs` (فقط)

| الحالة | scanned | passed | failed | score |
|---|---|---|---|---|
| قبل (= بعد Phase 3.1) | 129 | 115 | 14 | 0.892 |
| بعد Phase 3.2 | 129 | **127** | **2** | **0.915** |

**الفشلان المتبقيان (كلاهما شرعي، موثَّق، لا يُلمَس):**
- `ec-cli/src/main.rs` — `reversibility=0.00` (فحص شرعي على ملف إنتاج، منذ Phase 3.1)
- `ec-memory/src/types.rs` — `architectural_stability=0.43` (تبعية خارجية حقيقية: chrono/serde/uuid — راجع ADR-023 §5)

مبدأ نقاء الـKernel: لا تغييرات تنتهك القيود — كل التعديل داخل `ec-analysis`، منطقي بحت.

---

## 4. ADRs ذات الصلة

- **ADR-023**: مبدأ نقاء الـKernel (موثَّق رسميًا لأول مرة) + إصلاح CouplingVisitor (النمطان ج وهـ) — **جديد**
- ADR-021: سياسة فصل docker_tests عن slow_tests
- ADR-022: إصلاح النقاط العمياء في ec check (الأنماط أ+ب+د منفَّذة، ج مؤجَّل ← أُغلق الآن بـADR-023)

---

## 5. الـ Crates — مرجع بنيوي مختصر

| Crate | الدور |
|---|---|
| `ec-fitness` | تمثيل FitnessVector وPareto |
| `ec-epistemic` | الثقة/عدم اليقين والنمذجة البايزية |
| `ec-constitutional` | التقييم الدستوري |
| `ec-sandbox` | التنفيذ المعزول وRealityVector |
| `ec-analysis` | التحليل الساكن عبر AST |
| `ec-memory` | الذاكرة السببية append-only |
| `ec-codegen` | توليد الكود |
| `ec-governance` | الحوكمة والمقترحات والتدقيق |
| `ec-api` | REST API |
| `ec-cli` | واجهة سطر الأوامر |
| `ec-app` | تكامل النظام بالكامل |

---

## 6. البنود المفتوحة الحالية

| البند | الأولوية | ملاحظة |
|---|---|---|
| `ec check` على `sel-agent-v4` بالأداة المُصلَحة بالكامل | عالية | غير محجوب بعد الآن — Pattern ج وهـ أُغلقا |
| قرار معماري بشأن `ec-memory/types.rs` (0.43) | متوسطة | ليس خطأ أداة — راجع ADR-023 |
| الملف الضال `scripts/check_replay_determinism.sh` | منخفضة | untracked، على الأرجح من `sel-agent-engine` |
| v1 (`analyzer.rs::analyze_code` في `ec-app/pipeline.rs`) | منخفضة | قرار معماري مستقل، مُتحقَّق أنه معزول تمامًا عن CouplingVisitor |
| 2× `unwrap()` في `ec-cli/src/main.rs` | منخفضة | معروف آمن عمليًا |
| مراقبة `gate_network_isolation` عبر CI | منخفضة | مستمر |
| README / badge / عدّ ADRs الفعلي | منخفضة | لاحقًا |

---

## 7. أوامر الصيانة

```bash
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
نهاية الوثيقة المرجعية — Engineering Civilization v1.9.3 (2026-08-16)
