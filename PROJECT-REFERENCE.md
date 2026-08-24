# Engineering Civilization — الوثيقة المرجعية

## v1.9.5 · Phase 4 + Post-Audit Remediation (G1/G2) · 2026-08-23

هذه الوثيقة مرجع حالة مضغوط ودقيق. لا تحتوي أرقامًا إلا إذا كانت متحققة مباشرة من تشغيل فعلي.

---

## ⚠️ حالة التحقق الحالية

| البند | الحالة | الدليل التنفيذي الفعلي |
|---|---|---|
| `cargo build --workspace --locked` | ✅ | تشغيل فعلي ناجح (صفر أخطاء) |
| `cargo clippy --workspace --tests --locked -- -D warnings` | ✅ | صفر تحذيرات |
| `cargo fmt --all -- --check` | ✅ | نظيف بالكامل |
| `cargo test --workspace --locked --no-fail-fast` | ✅ | **0 failed** — العدد الكلي لـ`passed` يتذبذب طفيفًا بين البيئات (لوحظ 659–666 عبر جلسات/بيئات مختلفة، لا رقم "صحيح" واحد يُعتمَد بلا تشغيل مباشر لكل بيئة) |
| `cargo test -p ec-sandbox --features docker_tests` | ⚠️ | **غير مُتحقَّق رقميًا في هذه الجلسة تحديدًا** — نجح بالكامل على بيئة chokri سابقًا (بلا Docker هنا لتأكيد رقم دقيق). أي رقم "713" لم يُشغَّل فعليًا وقت كتابته — أُزيل حتى يُعاد تأكيده |
| `cargo run --bin ec -- check .` | ✅ | **130 scanned / 128 passed / 2 failed / score=0.916** (Exit Code = 1) — مؤكَّد بثلاث بيئات مستقلة على الأقل |
| المصادقة على `ec-api` | ✅ | `X-API-Key` إجباري، مؤكَّد سلوكيًا عبر `curl` حقيقي (401/401/200/200) |
| وضع الـSandbox الحقيقي في الـpipelines | ✅ | مُصحَّح (ADR-025 G1) — كان مُصلَّدًا Simulated في 3 بنى (`IntegrationPipeline`, `IterativePipeline`, `BayesianPipeline`)، الآن قابل للاختيار فعليًا عبر `new_docker`/`with_sandbox_config` |
| seccomp في مسار الإنتاج | ⚠️ | مُستعاد (ADR-025 G2) بعد تحقيق جذري (`clone3` غائبة عن allowlist) — **افتراض قوي غير مؤكَّد بتشغيل CI فعلي بعد** |
| نقاء الـ Kernel | ✅ | `ec-constitutional` خالٍ من `tokio`/`async` |
| عدد وثائق الـ ADRs | ✅ | **19 ADR** رسمية (`ls docs/adr/ | wc -l`) |

---

## 1. ما الجديد في v1.9.5 (بعد مراجعتين عدائيتين إضافيتين)

مراجعتان مستقلتان إضافيتان بعد إغلاق Phase 4 كشفتا فجوتين لم تُعالَجا سابقًا:

- **G1**: ثلاث بنى pipeline (لا واحدة) تُصلِّد `SandboxMode::Simulated` رغم أن `SandboxExecutor` نفسه "إلزامي التحصين" — مُصحَّح بإضافة منشئات صريحة (`new_docker`, `with_sandbox_config`) بلا كسر التوافق مع الاختبارات القديمة.
- **G2**: commit لاحق (`6817805`) عطَّل `seccomp` في الإنتاج بلا تحقيق — مُصحَّح بإضافة `clone3`/`close_range` لملف الـallowlist (فرضية جذر موثَّقة، لا يقين مؤكَّد) واستعادة `HardenedConfig::default()`.

كلا الإصلاحين رفضا صراحةً مقترحات سابقة كانت ستُغلق ملاحظة التدقيق **شكليًا** بلا تغيير جوهري (حقل بيانات لا يقرؤه أحد لـG1، إعادة صياغة تعليق بلا إصلاح لـG2). التفاصيل الكاملة في **`docs/adr/ADR-025-post-phase4-g1-g2-remediation.md`**.

بنود Phase 4 الأصلية (F1–F10) موثَّقة في **`docs/adr/ADR-024-multi-model-audit-resolutions.md`**.

بنود مؤجَّلة صراحة (من مراجعة Arena): تغطية اختبارية منخفضة لملفات `ec-sandbox` بلا Docker (G5)، ومعايرة عتبة `architectural_stability` ضد corpus مشاريع خارجية بعد إيجابيات كاذبة على `ripgrep` (G6) — كلاهما يحتاج عملاً مستقلاً، لا تصحيحًا سريعًا.

---

## 2. الـ Crates — مرجع بنيوي مختصر (11 Crates)

| Crate | الدور | حالة النقاء |
|---|---|---|
| `ec-fitness` | تمثيل FitnessVector وPareto | Kernel نقي |
| `ec-epistemic` | الثقة والنمذجة المعرفية | Kernel نقي |
| `ec-constitutional` | التقييم الدستوري | Kernel نقي (أُزيل tokio) |
| `ec-analysis` | التحليل الساكن عبر AST | Kernel نقي |
| `ec-memory` | الذاكرة السببية append-only | Kernel (باستثناء storage) |
| `ec-codegen` | توليد الكود | Kernel نقي |
| `ec-sandbox` | التنفيذ المعزول وRealityVector | I/O (Hardened Docker) |
| `ec-governance` | الحوكمة والمقترحات والتدقيق | I/O |
| `ec-api` | REST API | I/O (X-API-Key Protected) |
| `ec-cli` | واجهة سطر الأوامر | I/O (Strict Exit Code) |
| `ec-app` | تكامل النظام بالكامل | I/O (Sandbox Mode قابل للاختيار فعليًا) |

---

## 3. أوامر الصيانة القياسية

```bash
# الفحص والبناء الصارم
cargo build --workspace --locked
cargo clippy --workspace --tests --locked -- -D warnings
cargo fmt --all -- --check
cargo test --workspace --locked --no-fail-fast

# تشغيل اختبارات Docker المعزولة
cargo test -p ec-sandbox --locked --features docker_tests -- --test-threads=1

# فحص المشروع الذاتي
cargo run --bin ec -- check .
```

نهاية الوثيقة المرجعية — Engineering Civilization v1.9.5 (2026-08-23)
