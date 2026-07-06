# Engineering Civilization — الوثيقة المرجعية
## v1.7 · Phase 0 Verified · 2026-07-06

---

## ⚠️ حالة التحقق (Phase 0)

آخر تحقق نظيف من الصفر: **2026-07-06** — بيئة: WSL2, rustc 1.96.0, commit `4260f37` + 3 تعديلات (أدناه).

### الأرقام المُتحقَّقة فعليًا في هذه المرحلة (لا ادّعاءات)

| البند | القيمة المقاسة |
|---|---|
| البناء (`cargo build --workspace --locked`) | ✅ نجح |
| clippy (`--workspace --tests --locked -D warnings`) | ✅ 0 تحذيرات |
| الاختبارات: نجح | **673** |
| الاختبارات: فشل | **1** (تفصيل أدناه) |
| الاختبارات: ignored | **16** |
| unwrap() في كود الإنتاج | **2** (`ec-cli/src/main.rs`, الأسطر 111 و177) |
| cargo fmt --check | ⚠️ توجد فروقات تنسيق (راجع `phase0_artifacts/fmt.log`) |
| أسطر Rust (`crates/**/*.rs`) | **20635** |
| Docker متاح | نعم (WSL2 — Server 29.3.1) |

**ملاحظة منهجية (اكتُشفت أثناء المراجعة النهائية):** كان في مسودة سابقة حقل "عدّ #[test] نصي = 697"، حُذِف لأنه غير دقيق. السبب: عدّ نصي أعمى (`grep`) لا يميّز بين إعلانات اختبار حقيقية وأسطر `#[test]` تظهر كنص داخل `string literals` (عيّنات كود يستخدمها `ec-analysis` لاختبار نفسه) أو داخل ملفات بيانات مثل `tests/fixtures/tier1_excellent/pure_math.rs`. الفارق الفعلي: 697 − 690 = 7، مطابق تمامًا لعدد هذه الحالات (3 في fixture + 4 في string literals عبر `coverage.rs` و`metrics.rs`). **الرقم الموثوق الوحيد هو 690 (المُكتشَف فعليًا بواسطة `cargo test`)، لا أي عدّ نصي مستقل.**

### الاختبار الفاشل في هذا التشغيل الكامل

```
ec-app / week18_phase2_gate / gate_network_isolation
```
**السبب المسجَّل:** `Timeout { duration_secs: 60 }` أثناء التشغيل الكامل المتوازي.
**عند إعادة تشغيله منفردًا:** نجح في 4.48 ثانية (`cargo test -p ec-app --test week18_phase2_gate gate_network_isolation -- --exact`).
**التصنيف الحالي:** على الأرجح تزاحم موارد (resource contention) على WSL2 أثناء تشغيل حاويات Docker متعددة بالتوازي، وليس خللًا منطقيًا ثابتًا — مدعوم بأن نفس فئة الاختبارات فشلت بأعداد مختلفة تمامًا بين تشغيلتين متتاليتين (~16 فشلًا في تشغيل سابق، فشل واحد فقط هنا).
**درجة اليقين:** نجاح منفرد واحد مؤشر جيد، لا إثبات إحصائي كامل. لم يُغلَق كـ"لا حاجة لعمل" — أُدرِج كبند مفتوح فعلي في القسم 6، لا كعذر لتجاهله.

### unwrap() في كود الإنتاج (الاثنان فقط، مُتحقَّق منهما بدقة)
```
crates/ec-cli/src/main.rs:111  — serde_json::to_string_pretty(..).unwrap()
crates/ec-cli/src/main.rs:177  — serde_json::to_string_pretty(..).unwrap()
```
كلاهما على `serde_json::Value` داخلي — لا يفشلان عمليًا في أي مسار معروف، لكنهما `unwrap()` حقيقيان موثَّقان، لا "صفر" كما ادّعت v1.6.

### البيئة المرجعية لهذا التحقق
```
rustc:  1.96.0 (ac68faa20 2026-05-25)
cargo:  1.96.0
OS:     WSL2 Linux x86_64
commit: 4260f373c737bb69379ae262c2826707bf384ca5 + 3 تعديلات Phase 0 (غير مُلتزَم بها بعد وقت هذا التقرير)
```

### ⚠️ ادّعاءات موروثة من v1.6 — لم تُعَد فحصها في Phase 0
هذه الأرقام **لم تُقَس في هذه المرحلة**، منقولة كما هي من الوثيقة السابقة. لا تُقرأ كجزء من "Phase 0 Verified":

| الادّعاء | آخر مصدر | الحالة |
|---|---|---|
| `ec check: 71/129 files passing · score 0.874` | v1.6 (على مشروع خارجي `sel-agent-v4`) | غير مُعاد فحصه؛ خاضع أصلًا لتحفظ منهجي حول تصميم `test_coverage` (انظر تدقيق سابق) |
| `10 design invariants · 12 ADRs` | v1.6 | لم يُعَد عدّه فعليًا في Phase 0 |
| `pre-commit hook نشط على sel-agent-v4` | v1.6 | لم يُعَد التحقق من كونه لا يزال مُفعَّلًا |

---

## 1. نظرة عامة (الحقائق المُتحقَّقة فقط)
11 crates · ~20,635 سطر Rust · 690 اختبارًا مكتشفًا في آخر تشغيل نظيف (673 ناجح + 1 فاشل قابل لإعادة الإنتاج بنجاح منفردًا + 16 ignored)
0 تحذيرات clippy · 2 unwrap() موثَّقان في كود الإنتاج

للادّعاءات غير المُعاد فحصها (نطاق التحليل، عدد ADRs، حالة الـ hook)، راجع القسم أعلاه صراحةً — لا تُذكر هنا كحقائق حالية.

---

## 2. ما الجديد في v1.7 (Phase 0)

### التعديلات الثلاثة المطبَّقة

| # | الملف | التعديل | السبب |
|---|-------|---------|-------|
| 1 | `crates/ec-constitutional/tests/week6_meta.rs` | استبدال `HighRejectionRate(1.0)` بـ `HighRejectionRate(v) if v == 1.0` | float literal في pattern — إجراء وقائي (لم يُثبَت أنه يكسر البناء على هذا التوليكشين تحديدًا، لكنه غير سليم أسلوبيًا بغض النظر) |
| 2 | `crates/ec-analysis/src/visitors/test_visitor.rs` | إضافة `#[derive(Default)]` | clippy `new_without_default` |
| 3 | `crates/ec-analysis/src/visitors/coupling_visitor.rs` | إضافة `#[derive(Default)]` | clippy `new_without_default` |

### ما تغيّر من v1.6 إلى v1.7
- v1.6 ادّعى: "662 tests · 0 failed · 0 clippy warnings · 0 unwrap()"
- v1.7 يقيس فعليًا: **673 passed · 1 قابل لإعادة الإنتاج بنجاح منفردًا · 0 clippy warnings · 2 unwrap()**
- الفرق ليس تراجعًا في الجودة — بل توقف عن كتابة أرقام لم تُتحقَّق آليًا.

---

## 3. الـ Crates — مرجع بنيوي (لم يتغيّر محتواه في Phase 0، غير مُعاد تدقيقه هنا)

| Crate | الدور |
|---|---|
| `ec-fitness` | FitnessVector (6 أبعاد) + Pareto + Cosine similarity |
| `ec-epistemic` | EpistemicState + Bayesian calibration |
| `ec-constitutional` | محرك تقييم ضد 8 ثوابت دستورية |
| `ec-sandbox` | تنفيذ Docker + RealityVector — **ملاحظة WSL2:** بعض اختبارات Docker حساسة لمهلة 60 ثانية تحت الحمل المتوازي (انظر القسم 6) |
| `ec-analysis` | 6 AST visitors (syn) — الآن جميعها تدعم `Default` |
| `ec-memory` | DAG سببي + SQLite append-only |
| `ec-codegen` | توليد كود بالقوالب |
| `ec-governance` | حوكمة دستورية (proposals/audit) |
| `ec-api` | REST API (axum) — 11 endpoint |
| `ec-cli` | CLI (clap) — يحتوي 2 unwrap() موثَّقان أعلاه |
| `ec-app` | خط أنابيب التكامل الرئيسي |

Dependency graph وD1-D10 وتفاصيل الـ ADRs لم تتغيّر في Phase 0 ولم تُعَد مراجعتها — راجع النسخة الكاملة السابقة أو `docs/adr/` مباشرة.

---

## 4. البنود المفتوحة (منقولة صراحة للمراحل القادمة، لا مخفية)

| البند | الأولوية | المرحلة المقترحة |
|---|---|---|
| `gate_network_isolation` حساس للحمل على WSL2 | متوسطة | Phase 2 (CI) — رفع المهلة أو تشغيل `--test-threads=1` للتأكد إحصائيًا |
| 2 `unwrap()` في `ec-cli/src/main.rs` | منخفضة | Phase 3 |
| فروقات `cargo fmt` | منخفضة | Phase 3 |
| إعادة فحص `ec check` على `sel-agent-v4` (الرقم موروث وغير مُعاد قياسه) | متوسطة | Phase 4 (التحقق الخارجي) |
| عدّ ADRs والـ invariants الفعلي | منخفضة | Phase 5 (تنظيف الوثائق) |

---

## 5. أوامر الصيانة (المرجعية من الآن فصاعدًا)

```bash
cargo build --workspace --locked
cargo clippy --workspace --tests --locked -- -D warnings
cargo test --workspace --no-fail-fast --locked
```

---

*نهاية الوثيقة المرجعية — Engineering Civilization v1.7 · Phase 0 CLOSED (2026-07-06)*
