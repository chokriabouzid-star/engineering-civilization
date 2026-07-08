# Engineering Civilization — الوثيقة المرجعية
## v1.8 · Phase 1 Verified (CI) · 2026-07-07

---

## ⚠️ حالة التحقق (Phase 1 — CI)

CI عامل فعليًا على GitHub Actions منذ commit `b7ee05d`، تأكّد أخضرًا عبر 3 تشغيلات متتالية، آخرها commit `6351891` (2m 23s، 0 تحذيرات annotations).

### ما تحقق فعليًا في Phase 1 (لا ادّعاءات)

| البند | الحالة | الدليل |
|---|---|---|
| CI يعمل (4 jobs: lint, build, test-fast, test-docker) | ✅ | 3 تشغيلات خضراء متتالية على GitHub Actions |
| ثغرة أمنية حقيقية (fork bomb، غياب `--pids-limit`) | ✅ اكتُشفت وأُصلحت | commit `b7ee05d` — إصلاح في مسارين: `docker.rs` (المسار الإنتاجي الفعلي) و`hardened.rs` |
| اختبار جديد للثغرة | ✅ | `docker_pids_limit_blocks_fork_bomb` — يتحقق من `DockerRunner` مباشرة، لا `HardenedDockerRunner` فقط |
| تحذيرات Node.js 20 (4×) | ✅ أُزيلت | commit `6351891` — `actions/checkout@v7.0.0` (مُتحقَّق ببحث فعلي، ليس v5 كما ظُنّ أولًا)، `Swatinem/rust-cache@v2.9.1` |
| توليكشين مثبَّت | ✅ | `dtolnay/rust-toolchain@1.96.0` + `rust-toolchain.toml` محليًا |
| فروقات `cargo fmt` | ✅ أُصلحت | `cargo fmt --check` بوابة صلبة في job "Format & Clippy"، أخضر في كل التشغيلات |
| اختبارات محلية بعد إصلاح الثغرة | ✅ | 127 passed, 0 failed, 15 ignored (تحقّق يدوي مطابق تمامًا لرسالة commit `b7ee05d`) |

### بيئة CI (للمرجعية)
```
runner:  ubuntu-24.04.4 LTS, kernel 6.17.0-1018-azure, 4 CPU, 15.61GiB
rustc:   1.96.0 (مثبَّت عبر dtolnay/rust-toolchain@1.96.0)
commits: b7ee05d (إصلاح الثغرة + بنية CI) → 6351891 (تصحيح إصدارات actions)
```

### ⚠️ ادّعاءات موروثة من v1.6 — ما زالت غير مفحوصة
لم تتغيّر منذ v1.7؛ لا تُقرأ كحقائق حالية:

| الادّعاء | الحالة |
|---|---|
| `ec check: 71/129 files passing · score 0.874` | غير مُعاد فحصه؛ تحفّظ منهجي قائم حول تصميم `test_coverage` |
| `10 design invariants · 12 ADRs` | لم يُعَد عدّه فعليًا |
| `pre-commit hook نشط على sel-agent-v4` | لم يُعَد التحقق منه |

---

## 1. نظرة عامة (الحقائق المُتحقَّقة فقط)
11 crates · ~20,635 سطر Rust · 690 اختبارًا مكتشفًا آخر مرة تحقّق كامل — 674 ناجح + 16 ignored = 690 ✓ (في تشغيل Phase 0 المرجعي كان 673 passed + 1 failed + 16 ignored؛ الاختبار الفاشل `gate_network_isolation` ينجح دائمًا عند التشغيل المنفرد، لذا يُحسب في هذا الجدول ضمن الناجحين. راجع Phase 0 للتفصيل)
⚠️ **هذا الرقم (690) قديم نسبيًا:** أُضيف اختبار جديد واحد (`docker_pids_limit_blocks_fork_bomb`) أثناء Phase 1 ولم يُشغَّل تحقق كامل من الصفر بعده بعد — العدد الحقيقي الحالي على الأرجح **691**، لكن هذا **غير مؤكَّد** حتى تشغيل `phase0_verify.sh` كاملًا من جديد (أول خطوة في Phase 2، انظر أدناه).
0 تحذيرات clippy · 0 تحذيرات CI · 2 unwrap() موثَّقان في كود الإنتاج · CI أخضر منذ 3 تشغيلات

للادّعاءات غير المُعاد فحصها، راجع القسم أعلاه صراحةً.

---

## 2. ما الجديد في v1.8 (Phase 1)

### الاكتشاف الأهم: ثغرة أمنية حقيقية، لا خللًا في اختبار

اختبار `gate_escape_vector_5_fork_bomb_contained` فشل عند التشغيل الأول على GitHub Actions (`ESCAPED: spawned 10000 threads`) بعد أن كان "ينجح" دومًا على WSL2. السبب الجذري: الاحتواء كان يعتمد ضمنيًا على ضعف موارد WSL2، لا على حد صريح — `--memory` وحدها لا تحدّ عدد الـ threads القابلة للإنشاء (`nr_threads`/`pid_max` على مستوى الـ kernel المشترك). على بيئة أقوى (GitHub Actions: 4 CPU)، انكشف غياب الحماية الفعلية.

**الإصلاح (commit `b7ee05d`):**
| # | الملف | التعديل |
|---|-------|---------|
| 1 | `crates/ec-sandbox/src/docker.rs` | إضافة `--pids-limit=256` — **هذا هو المسار الإنتاجي الفعلي** المستخدم لتقييم الكود الحقيقي (لم يكن محميًا إطلاقًا قبل هذا) |
| 2 | `crates/ec-sandbox/src/hardened.rs` | نفس الإصلاح + إعادة تصميم عتبة اختبار fork bomb لتقيس إنفاذ `pids-limit` فعليًا، لا الاعتماد الضمني على الذاكرة |

**درس منهجي:** لولا تشغيل CI على بيئة مختلفة عن جهاز التطوير، هذه الثغرة كانت لتبقى مخفية إلى أجل غير مسمى.

### إصلاحات تبعية (commit `6351891`)
`actions/checkout@v4 → v7.0.0`، `Swatinem/rust-cache@v2 → v2.9.1` — إزالة تحذيرات Node.js 20 الأربعة، مُتحقَّق منها ببحث فعلي (وليس افتراضًا؛ الادّعاء الأول بأن v5 هو الأحدث كان غير دقيقًا — v6 وv7 صدرا لاحقًا).

---

## 3. الـ Crates — مرجع بنيوي (لم يتغيّر محتواه في Phase 1، غير مُعاد تدقيقه هنا)

| Crate | الدور |
|---|---|
| `ec-fitness` | FitnessVector (6 أبعاد) + Pareto + Cosine similarity |
| `ec-epistemic` | EpistemicState + Bayesian calibration |
| `ec-constitutional` | محرك تقييم ضد 8 ثوابت دستورية |
| `ec-sandbox` | تنفيذ Docker + RealityVector — **محمي الآن بـ `--pids-limit=256` في كل من `docker.rs` و`hardened.rs`** |
| `ec-analysis` | 6 AST visitors (syn) — تدعم `Default` |
| `ec-memory` | DAG سببي + SQLite append-only |
| `ec-codegen` | توليد كود بالقوالب |
| `ec-governance` | حوكمة دستورية (proposals/audit) |
| `ec-api` | REST API (axum) — 11 endpoint |
| `ec-cli` | CLI (clap) — يحتوي 2 unwrap() موثَّقان |
| `ec-app` | خط أنابيب التكامل الرئيسي |

Dependency graph وD1-D10 وتفاصيل الـ ADRs لم تتغيّر — راجع `docs/adr/` مباشرة.

---

## 4. البنود المفتوحة (منقولة صراحة للمراحل القادمة)

| البند | الأولوية | المرحلة المقترحة |
|---|---|---|
| `gate_network_isolation` حساس للحمل (WSL2 محليًا؛ لم يتكرر بعد على GitHub Actions حتى الآن) | متوسطة | Phase 2 — مراقبة عبر عدة تشغيلات CI قبل اعتباره محسومًا |
| توحيد سياسة `slow_tests` — بعض اختبارات Docker مُنضبطة بـ feature flag، أخرى تعمل تلقائيًا | متوسطة | Phase 2 |
| 2 `unwrap()` في `ec-cli/src/main.rs` | منخفضة | Phase 3 |
| إعادة فحص `ec check` على `sel-agent-v4` (رقم موروث غير مُعاد قياسه) | متوسطة | Phase 4 |
| عدّ ADRs والـ invariants الفعلي | منخفضة | Phase 5 |
| إضافة CI badge إلى README (لا يوجد README أصلًا بعد) | منخفضة | Phase 5 |

---

## 5. أوامر الصيانة

```bash
cargo build --workspace --locked
cargo clippy --workspace --tests --locked -- -D warnings
cargo test --workspace --no-fail-fast --locked
```
راجع `.github/workflows/ci.yml` للتشغيل الآلي المطابق.

---

*نهاية الوثيقة المرجعية — Engineering Civilization v1.8 · Phase 1 CLOSED (2026-07-07)*
