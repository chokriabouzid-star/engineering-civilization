# Engineering Civilization — الوثيقة المرجعية

## v1.9.6 · Sprint 1 (ADR-026 → ADR-031) · 2026-09-25

هذه الوثيقة مرجع حالة مضغوط. **كل رقم فيها مقاس بتشغيل فعلي ومربوط بالتزام محدد**؛ أي رقم بلا التزام مرجعي لا يُعتمد.

**الالتزام المرجعي:** `93950e8` على `master`.

---

## ⚠️ حالة التحقق الحالية

| البند | الحالة | الدليل التنفيذي الفعلي |
|---|---|---|
| بوابات CI (fmt · clippy `-D warnings` · build · rustdoc `-D warnings` · cargo audit · tests بلا Docker · tests Docker) | ✅ | 7/7 ناجحة على `93950e8`. `Constitutional Check` ما زال **استشاريًا** عمدًا لأن العتبات غير معايرة |
| `cargo test --workspace --locked --no-fail-fast` | ✅ | **697 passed / 0 failed / 51 ignored** على `93950e8` محليًا. العدد قد يختلف بين البيئات؛ الرقم صالح لهذا الالتزام وهذه البيئة فقط |
| `cargo test -p ec-sandbox --features docker_tests -- --test-threads=1` | ✅ | **152 passed / 0 failed / 1 ignored** محليًا على شجرة مطابقة حرفيًا لـ`93950e8` (صورة البصمة، ADR-031)، صفر حاويات `ec-sbx-*` متبقية قبل وبعد. رقم "713" القديم لم يُشغَّل وقت كتابته ويبقى مسحوبًا |
| `cargo test -p ec-app --features docker_tests -- --test-threads=1` | ✅ | **107 passed / 0 failed / 1 ignored** بنفس الظروف |
| `cargo run --bin ec -- check .` | ✅ | **141 scanned / 136 passed / 5 failed / score=0.911** (Exit Code = 1) على `93950e8` — exit=1 سلوك مقصود عند وجود انتهاكات (Strict Exit Code). رقم v1.9.5 (130/128/2/0.916) صار تاريخيًا |
| المصادقة على `ec-api` | ✅ | `X-API-Key` إجباري؛ مغطّى آليًا بـ`week50_gate` ضمن سويت workspace. تأكيد `curl` اليدوي (401/401/200/200) يعود إلى v1.9.5 |
| وضع الـSandbox الحقيقي في الـpipelines | ✅ | ADR-025 G1 — قابل للاختيار عبر `new_docker`/`with_sandbox_config` |
| seccomp في مسار الإنتاج | ✅ | الجذر المقاس في ADR-026 هو `statfs`/`fstatfs`، وهو يحل محل فرضية `clone3` الواردة في ADR-025 G2. تكافؤ مقاس (ADR-028)؛ تقييد `clone`/`clone3` (ADR-029) مع `seccomp_policy_gate` (11 اختبارًا) و`least_privilege_compat_gate` (3)؛ مهمة Docker في CI ناجحة |
| صورة الـSandbox | ✅ | `rust@sha256:31ee7fc65186be7e0e0ccb3f2ca305f14e4739e7642a1ae65753aa5d7b874523` — rustc 1.96.1، glibc 2.41، Debian 13، `linux/amd64` (ADR-031) |
| بقاء الخادم/CLI أمام انهيار المحلل | ✅ | ADR-030: التحليل في عملية فرعية؛ `f1_server_survival` و`f1_cli_survival` بوابتان دائمتان |
| نقاء الـ Kernel | ✅ | `ec-constitutional` خالٍ من `tokio`/`async` — مفروض آليًا بـ`crates/ec-constitutional/tests/kernel_purity.rs` مع تحكّم سلبي (ADR-027) |
| عدد وثائق الـ ADRs | ✅ | **26** ملفًا (`find docs/adr -maxdepth 1 -name '*.md' \| wc -l`). الترقيم غير متصل: 006–008 و011 و020 غير موجودة، و001–003 بلا بادئة `ADR-` |

---

## 1. ما الجديد منذ v1.9.5 (Sprint 1)

| PR | الالتزام | المحتوى | ADR |
|---|---|---|---|
| #1 | `0debf1c` | جذر فشل seccomp (`fstatfs`/`statfs`)، دورة حياة الحاويات، أوضاع fail-closed، تدقيق حوكمة دائم | ADR-026 |
| #2 | `2c0c507` | بيانات وصفية صادقة (LICENSE، rust-version)، بوابة نقاء النواة، تصليب CI | ADR-027 |
| #3 | `6937912` | T1.1: بوابة تنظيف الحاويات عند انتهاء المهلة | ADR-026 C1 |
| #4 | `c025fc3` | T1.2: بوابة تكافؤ seccomp ومتجهات الهروب | ADR-028 |
| #5 | `94eba48` | توثيق بوابتي T1.1/T1.2 | — |
| #6 | `e631b33` | T1.3: تقييد `clone` بقناع namespace، و`clone3` → `ENOSYS` | ADR-029 |
| #7 | `32d520f` | F1: عزل التحليل في عملية فرعية؛ `/analyze` يعيد 502/504/500 دون سقوط الخادم | ADR-030 |
| #8 | `93950e8` | صورة الـSandbox مثبتة بالبصمة على سلسلة Rust 1.96.x | ADR-031 |

---

## 2. ما كان جديدًا في v1.9.5 (سجل تاريخي — النص محفوظ كما هو)

> ملاحظة لاحقة: فرضية G2 أدناه (`clone3`/`close_range`) حلّ محلها الجذر المقاس في ADR-026 (`statfs`/`fstatfs`).

مراجعتان مستقلتان إضافيتان بعد إغلاق Phase 4 كشفتا فجوتين لم تُعالَجا سابقًا:

- **G1**: ثلاث بنى pipeline (لا واحدة) تُصلِّد `SandboxMode::Simulated` رغم أن `SandboxExecutor` نفسه "إلزامي التحصين" — مُصحَّح بإضافة منشئات صريحة (`new_docker`, `with_sandbox_config`) بلا كسر التوافق مع الاختبارات القديمة.
- **G2**: commit لاحق (`6817805`) عطَّل `seccomp` في الإنتاج بلا تحقيق — مُصحَّح بإضافة `clone3`/`close_range` لملف الـallowlist (فرضية جذر موثَّقة، لا يقين مؤكَّد) واستعادة `HardenedConfig::default()`.

كلا الإصلاحين رفضا صراحةً مقترحات سابقة كانت ستُغلق ملاحظة التدقيق **شكليًا** بلا تغيير جوهري (حقل بيانات لا يقرؤه أحد لـG1، إعادة صياغة تعليق بلا إصلاح لـG2). التفاصيل الكاملة في **`docs/adr/ADR-025-post-phase4-g1-g2-remediation.md`**.

بنود Phase 4 الأصلية (F1–F10) موثَّقة في **`docs/adr/ADR-024-multi-model-audit-resolutions.md`**.

بنود مؤجَّلة صراحة (من مراجعة Arena): تغطية اختبارية منخفضة لملفات `ec-sandbox` بلا Docker (G5)، ومعايرة عتبة `architectural_stability` ضد corpus مشاريع خارجية بعد إيجابيات كاذبة على `ripgrep` (G6) — كلاهما يحتاج عملاً مستقلاً، لا تصحيحًا سريعًا.

---

## 3. قيود معروفة مفتوحة (لا تُطوى ضمنيًا)

- **F1 (ADR-030):** العزل يحمي بقاء العملية الأم فقط؛ لا حد لذاكرة العامل، ولا سقف تزامن على `/analyze`، وسلوك core dump غير مقاس، و`ec check` يتخطى الملفات غير UTF-8 بصمت.
- **F2 مفتوحة:** `generate_tests` في `ec-codegen` يولّد متغيرات غير معرّفة (E0425)، وتظهر فقط عند التصريف بـ`rustc --test`.
- **توصيل L2 غير مكتمل:** `POST /analyze` عديم الحالة (لا جدول `evaluation_runs`)؛ ذاكرة الـAPI بلا تغذية ولا استعادة من القرص؛ انتهاكات مسار Docker الناجح لا تُبنى.
- **المنصة:** seccomp والصورة مُتحقَّقتان على `x86_64` / `linux/amd64` فقط.
- **`without_seccomp()`** يعني سياسة daemon الافتراضية (`builtin`) لا `unconfined` (ADR-028).
- **G5/G6** من v1.9.5 ما زالا مؤجلين.
- **لا بوابة آلية لصدق هذه الوثيقة بعد (docs-truth)**؛ تُحدَّث يدويًا.

---

## 4. الـ Crates — مرجع بنيوي مختصر (11 Crates)

| Crate | الدور | حالة النقاء |
|---|---|---|
| `ec-fitness` | تمثيل FitnessVector وPareto | Kernel نقي |
| `ec-epistemic` | الثقة والنمذجة المعرفية | Kernel نقي |
| `ec-constitutional` | التقييم الدستوري | Kernel نقي (مفروض آليًا، ADR-027) |
| `ec-analysis` | التحليل الساكن عبر AST + عزل التحليل | منطق التحليل نقي؛ وحدة `isolation` تُطلق عملية فرعية (I/O) منذ ADR-030 — لم يعد الصندوق كله "Kernel نقي" |
| `ec-memory` | الذاكرة السببية append-only | Kernel (باستثناء storage) |
| `ec-codegen` | توليد الكود | Kernel نقي |
| `ec-sandbox` | التنفيذ المعزول وRealityVector | I/O (Hardened Docker، صورة مثبتة بالبصمة، ADR-031) |
| `ec-governance` | الحوكمة والمقترحات والتدقيق | I/O |
| `ec-api` | REST API | I/O (X-API-Key؛ حد جسم 2 MiB على `/analyze`؛ 502/504/500 عند فشل العامل) |
| `ec-cli` | واجهة سطر الأوامر | I/O (Strict Exit Code؛ `ec analyze` → 2 عند فشل العامل؛ `ec check` fail-closed) |
| `ec-app` | تكامل النظام بالكامل | I/O (Sandbox Mode قابل للاختيار فعليًا) |

---

## 5. أوامر الصيانة القياسية (بترتيب CI)

~~~bash
cargo fmt --all -- --check
cargo clippy --workspace --tests --locked -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --locked --no-deps
cargo build --workspace --locked
cargo test --workspace --locked --no-fail-fast

# اختبارات Docker المعزولة
cargo test -p ec-sandbox --locked --no-fail-fast --features docker_tests -- --test-threads=1
cargo test -p ec-app --locked --no-fail-fast --features docker_tests -- --test-threads=1

# تدقيق التبعيات وفحص المشروع الذاتي
cargo audit
cargo run --bin ec -- check .
~~~

---

## 6. كيف تُحدَّث هذه الوثيقة

أعد قياس كل رقم على التزام محدد واذكر الالتزام. لا تنقل أرقامًا من سجلات أو محادثات بلا hash. الرقم التاريخي يبقى مع إصداره، ولا يُمحى.

نهاية الوثيقة المرجعية — Engineering Civilization v1.9.6 (93950e8، 2026-09-25)
