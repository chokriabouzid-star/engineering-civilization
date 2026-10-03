# Engineering Civilization — الوثيقة المرجعية

## v1.9.8 · تحديث الحالة بعد PR #13 · 2026-10-03

هذه الوثيقة مرجع حالة مضغوط. **أرقام الحالة الحالية مقاسة بتشغيل فعلي على الالتزام المرجعي أدناه**؛ وتبقى الأرقام التاريخية منسوبة إلى الالتزامات المذكورة معها. أي رقم بلا التزام مرجعي لا يُعتمد.

**الالتزام المرجعي:** `803a00dc5bd5c33058c696a3164674bf48cc6275` على `master` (شجرته = شجرة رأس PR `dfa6eac`؛ قِيس محليًا في 2026-10-02/03؛ وفُحصت وظائف CI على رأس PR وعلى التزام الدمج عبر GitHub API).

---

## ⚠️ حالة التحقق الحالية

| البند | الحالة | الدليل التنفيذي الفعلي |
|---|---|---|
| وظائف CI (Format & Clippy · Build · Tests بلا Docker · Tests Docker · Rustdoc · Security audit · Constitutional Check) | ✅ | 7/7 check-runs مكتملة بنتيجة `success` على رأس PR `dfa6eac` وعلى التزام الدمج `803a00d` عبر GitHub API (6 مطلوبة + 1 استشارية). وظيفة `Constitutional Check` **استشارية** (`continue-on-error`)؛ خضرتها لا تثبت أن `ec check .` اجتاز. `cargo audit` ناجح محليًا وعلى CI؛ `cargo doc -D warnings` ناجح محليًا وعلى CI |
| `cargo test --workspace --locked --no-fail-fast` | ✅ | **721 passed / 0 failed / 61 ignored** على `803a00d` محليًا (51 السابقة + 10 لاختبارات `sandbox_truth_gate`). العدد قد يختلف بين البيئات؛ الرقم صالح لهذا الالتزام وهذه البيئة فقط |
| `cargo test -p ec-sandbox --locked --no-fail-fast --features docker_tests -- --test-threads=1` | ✅ | **156 passed / 0 failed / 7 ignored** على `803a00d` محليًا (152 السابقة + 10 للبوابة الجديدة − 6 معزولة صراحة لـSBX-03؛ صورة البصمة، ADR-031/ADR-032)؛ صفر حاويات `ec-sbx-*` قبل السويتات وبعدها. رقم "713" القديم لم يُشغَّل وقت كتابته ويبقى مسحوبًا |
| `cargo test -p ec-app --locked --no-fail-fast --features docker_tests -- --test-threads=1` | ✅ | **108 passed / 0 failed / 1 ignored** على `803a00d` بنفس الظروف |
| `cargo run --locked --bin ec -- check .` | ⚠️ | **144 scanned / 139 passed / 5 failed / score=0.911**، رمز الخروج **1**، على `803a00d` (ملف البوابة الجديد ضمن الممسوح؛ نفس الانتهاكات الخمسة). التشغيل اكتمل، لكن المشروع **لا يجتاز حكم `ec check`**. الانتهاكات: `ec-cli/src/main.rs` (reversibility 0.00)، `ec-analysis/src/depth_guard.rs` (maintainability 0.00)، `ec-api/src/handlers.rs` (architectural_stability 0.28)، `ec-sandbox/tests/seccomp_policy_gate.rs` (maintainability 0.16)، `ec-memory/src/types.rs` (architectural_stability 0.43). العتبات والقيم غير معايرة؛ لا تُقرأ كحكم مطلق على جودة هذه الملفات |
| المصادقة على `ec-api` | ✅ (في الحالات المغطاة) | `X-API-Key` مطلوب في الحالات التي يغطيها `week50_gate` ضمن سويت workspace. تأكيد `curl` اليدوي (401/401/200/200) يعود إلى v1.9.5. سلوك الخادم عند ضبط `EC_API_KEY` على قيمة فارغة **لم يُقَس على `803a00d`** |
| وضع الـSandbox الحقيقي في الـpipelines | ✅ (قابلية الاختيار فقط) | ADR-025 G1 — قابل للاختيار عبر `new_docker`/`with_sandbox_config`. صدق `success`/`correctness` لأعطال التنفيذ (panic/exit/OOM/فشل تصريف يحوي العلامة) **مقاس ومُصلَح على `803a00d`** (بوابة `sandbox_truth_gate` 10/10، ADR-032). كشف الانتهاكات (SBX-03) **ما زال مفتوحًا** |
| seccomp في مسار الإنتاج | ✅ | الجذر المقاس في ADR-026 هو `statfs`/`fstatfs`، وهو يحل محل فرضية `clone3` الواردة في ADR-025 G2. تكافؤ مقاس (ADR-028)؛ تقييد `clone`/`clone3` (ADR-029) مع `seccomp_policy_gate` (11 اختبارًا) و`least_privilege_compat_gate` (3)؛ `seccomp_parity_gate` ناجح على `803a00d` منفردًا وضمن السويتة؛ مهمة Docker في CI ناجحة |
| صورة الـSandbox | ✅ | `rust@sha256:31ee7fc65186be7e0e0ccb3f2ca305f14e4739e7642a1ae65753aa5d7b874523` — الصورة المحلية المحلولة من هذه البصمة هي `linux/amd64` (rustc 1.96.1، glibc 2.41، Debian 13؛ ADR-031)؛ نطاق منصات البصمة في السجل لم يُعَد قياسه على `803a00d` |
| بقاء الخادم/CLI أمام انهيار المحلل | ✅ | ADR-030: التحليل في عملية فرعية؛ `f1_server_survival` و`f1_cli_survival` بوابتان دائمتان |
| نقاء الـ Kernel | ✅ | `ec-constitutional` خالٍ من `tokio`/`async` — مفروض آليًا بـ`crates/ec-constitutional/tests/kernel_purity.rs` مع تحكّم سلبي (ADR-027) |
| عدد وثائق الـ ADRs | ✅ (العدد والعناوين فقط) | **27** ملف Markdown مباشرة في `docs/adr` على `803a00d` (الجديد ADR-032) (`find docs/adr -maxdepth 1 -name '*.md' \| wc -l`)، ولا مجلدات فرعية. الترقيم غير متصل: 006–008 و011 و020 غير موجودة. الملفات 001–003 بلا بادئة `ADR-` في الاسم، لكن عناوينها مطابقة. بوابة PR #10 تطابق رقم اسم الملف مع أول عنوان غير فارغ وتشترط 26 ملفًا على الأقل (27 يحقّق الشرط)؛ لا تكشف تكرار الأرقام، ولا تفحص المجلدات الفرعية، ولا تثبت صدق المحتوى |

**سجل v1.9.7 التاريخي (المرجع `505d586`):** workspace **721/0/51**؛ `ec-sandbox` Docker **152/0/1**؛ `ec-app` Docker **108/0/1**؛ `ec check .` **143 scanned / 138 passed / 5 failed / 0.911** مع خروج 1. **سجل v1.9.6 التاريخي (المرجع `93950e8`):** workspace **697/0/51**؛ `ec-sandbox` Docker **152/0/1**؛ `ec-app` Docker **107/0/1**؛ `ec check .` **141 scanned / 136 passed / 5 failed / 0.911** مع خروج 1. هذه أرقام تلك النسخ، وليست إعادة قياس على `803a00d`.

---

## 1. سجل PRs منذ v1.9.5

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
| #9 | `345a385` | تحديث `PROJECT-REFERENCE.md` إلى v1.9.6 بأرقام مرجعها `93950e8` | — |
| #10 | `275d326` | بوابة مطابقة عناوين ADR مع أرقام الملفات، وتصحيح ثلاثة عناوين قديمة | — |
| #11 | `505d586` | إصلاح مسار الاختبارات المولَّدة في `ec-codegen` للمسارات الرقمية المغطاة، وإزالة سقف الأسماء الثمانية في جسمي القالبين، وبوابة انحدار من 23 اختبارًا | — |
| #12 | `b8b9a1e` | تحديث `PROJECT-REFERENCE.md` إلى v1.9.7 بأرقام مرجعها `505d586` | — |
| #13 | `803a00d` | صدق تنفيذ الـsandbox (SBX-01/02): حفظ رمز الخروج الحقيقي وفصل تشخيص `rustc`؛ بوابة `sandbox_truth_gate` 10/10؛ عزل 6 اختبارات Week14 لـSBX-03 المفتوح | ADR-032 |

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
- **F2 (PR #11، `505d586`) — ضمن نطاق البوابة لا أكثر:** كان الاختبار المولَّد يبني الاستدعاء من أسماء المعاملات (`a, b, c, d` مبتورة عند أربعة) فينتج E0425/E0061، وجسمه `let _ = result;` لا يمكن أن يفشل. في المسارات الرقمية المغطاة صار يمرّر قيمًا مميّزة `1..n` ويقارن الناتج بقيمة مشتقة من القالب، وصار جسما القالبين يستعملان المعاملات حتى `z` بدل التوقف عند `h`. البوابة `crates/ec-codegen/tests/generated_code_compiles.rs` (23 اختبارًا) تصرّف الاختبار المولَّد وتشغّله، وتشترط أن يفشل جسم خاطئ عمدًا عند التوكيد لا عند التصريف. القيمة المشتقة تثبت مطابقة الجسم للقالب، **لا** صحة الدالة وفق نية المستخدم.
- **حدود تغطية F2:** البوابة تغطي حالات `i32` و`f32` و`f64` محددة، وأوراكل n=9 في القالبين، وحالات امتناع عن توليد الاختبار. الأنواع الرقمية المدعومة الأخرى، وحدود الدقة العائمة، والامتناع عند `n > 26` وعند pure بـ`n = 0` وللأنواع `isize`/`usize`/`i128`/`u128`، كلها مطبّقة وموثّقة لكنها ليست حالات مباشرة في البوابة.
- **ديون أجسام القوالب (خارج PR #11):** `format_params` يعيد الاسم `x` ابتداءً من المعامل 27 فيتكرر مع المعامل 24؛ القالب pure عند `n = 0` ينتج `42` بلا نوع؛ أجسام `i32 -> String` (E0605) و`String` عند `n = 2` (E0308) و`RustStructTemplate` (E0425) لا تُصرَّف.
- **توصيل L2 غير مكتمل:** `POST /analyze` عديم الحالة (لا جدول `evaluation_runs`)؛ ذاكرة الـAPI بلا تغذية ولا استعادة من القرص؛ انتهاكات مسار Docker الناجح لا تُبنى.
- **SBX-01/02 (PR #13، `803a00d` — مُغلق ضمن نطاق البوابة):** كان `compiler.rs` يجبر رمز الخروج إلى 0 عند وجود `---OUTPUT---`، وتشخيص `rustc` في stdout يسمح بتزوير نجاح التصريف. الفاصل يُطبع بعد نجاح التصريف فقط، ورمز البرنامج الحقيقي محفوظ (101/3/137). البوابة `sandbox_truth_gate` (10 اختبارات، seccomp مفعّل) كانت حمراء قبل الإصلاح (6/4) وخضراء بعده (10/10).
- **SBX-03 (مفتوح، ADR-032):** مسار Docker يبني `violations` فارغة دائمًا فلا يمكن لـ`is_secure()` أن يفشل هناك؛ 6 اختبارات Week14 على مستوى المُنفِّذ معزولة بسبب صريح تحت `docker_tests`. دليل المتجهات المحدود: `seccomp_parity_gate` (5 متجهات خام بعلامات `BLOCKED`/`CONTAINED`)، لا برهان عزل عام. بوابتا الـ100 تنفيذ في `week16_gate` و`week18_phase2_gate` لم تُراجعا هنا.
- **المنصة:** seccomp والصورة مُتحقَّقتان على `x86_64` / `linux/amd64` فقط.
- **`without_seccomp()`** يعني سياسة daemon الافتراضية (`builtin`) لا `unconfined` (ADR-028).
- **G5/G6** من v1.9.5 ما زالا مؤجلين.
- **لا بوابة آلية لصدق هذه الوثيقة بعد (docs-truth)**؛ تُحدَّث يدويًا. بوابة PR #10 (`adr_title_integrity`) تفحص عناوين ADR فقط، لا محتوى هذه الوثيقة.

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

## 5. أوامر التحقق المحلي (تقابل وظائف CI)

~~~bash
cargo fmt --all -- --check
cargo clippy --workspace --tests --locked -- -D warnings
cargo build --workspace --locked
cargo test --workspace --locked --no-fail-fast

# اختبارات Docker المعزولة
cargo test -p ec-sandbox --locked --no-fail-fast --features docker_tests -- --test-threads=1
cargo test -p ec-app --locked --no-fail-fast --features docker_tests -- --test-threads=1

# التوثيق
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --locked --no-deps

# تدقيق التبعيات وفحص المشروع الذاتي
cargo audit
cargo run --bin ec -- check .
~~~

---

## 6. كيف تُحدَّث هذه الوثيقة

أعد قياس كل رقم على التزام محدد واذكر الالتزام. لا تنقل أرقامًا من سجلات أو محادثات بلا hash. الرقم التاريخي يبقى مع إصداره، ولا يُمحى.

نهاية الوثيقة المرجعية — Engineering Civilization v1.9.8 (803a00d، 2026-10-03)
