# Engineering Civilization — الوثيقة المرجعية
## v1.9 · Phase 2 CLOSED (Test Classification Policy) · 2026-07-13

---

## ⚠️ حالة التحقق (Phase 2 — سياسة تصنيف الاختبارات)

Phase 2 مغلقة بأدلة خام مباشرة. CI أخضر على commit `f1cb6bd` (2m 44s، 4 jobs).

### ما تحقق فعليًا في Phase 2 (لا ادّعاءات)

| البند | الحالة | الدليل |
|---|---|---|
| ADR-021 مكتوبة ومعتمدة | ✅ | `docs/adr/ADR-021-slow-tests-policy.md`، commit `f1cb6bd` |
| 46 اختبار Docker مصنَّف بدقة | ✅ | 44 `docker_tests` + 2 `slow_tests`، تقاطع خماسي (كود + CI + محلي + phase2_verify.sh + مراجعة يدوية مستقلة) |
| Cargo features منفصلة | ✅ | `ec-sandbox/Cargo.toml` و`ec-app/Cargo.toml`، commit `430e6ce` |
| test-fast يعمل بلا `--exclude` | ✅ | CI test-fast: 645 passed / 0 failed / 46 ignored / 691 discovered |
| test-docker مع `--features docker_tests` | ✅ | ec-sandbox: 141/0/1، ec-app: 101/0/1 (CI ومحلي متطابقان) |
| اختباران `slow_tests` معزولان | ✅ | `gate_zero_escapes_in_100_executions` في week16 وweek18 — يعمل يدويًا فقط، لا في CI |
| phase2_verify.sh آلي | ✅ | يفحص التسرّب بقائمة 44 اسم صريح، exit=0 |

### بيئة التحقق النهائية
runner: ubuntu-24.04 (GitHub Actions)
rustc: 1.96.0
commits: 430e6ce (كود + CI) → f1cb6bd (ADR-021)
CI run: #6 على master، Success في 2m 44s

text


### ⚠️ ادّعاءات موروثة من v1.6 — ما زالت غير مفحوصة
لا تغيير عن v1.8:

| الادّعاء | الحالة |
|---|---|
| `ec check: 71/129 files passing · score 0.874` | غير مُعاد فحصه |
| `10 design invariants · 13 ADRs (بعد إضافة ADR-021)` | لم يُعَد عدّه فعليًا |
| `pre-commit hook نشط على sel-agent-v4` | لم يُعَد التحقق منه |

---

## 1. نظرة عامة (الحقائق المُتحقَّقة فقط)

11 crates · ~20,635 سطر Rust · **691 اختبارًا مكتشفًا** (645 ناجح + 0 فاشل + 46 مصنَّف بحسب ADR-021).
0 تحذيرات clippy (مُتحقَّق `f1cb6bd`) · 0 تحذيرات CI annotations (آخر تحقق مباشر بالصورة/PDF لـ`6351891`؛ الافتراض المعقول لـ`f1cb6bd`: لم تتغير إصدارات الـactions، لكن Annotations لم تُفحَص خاماً بعد) · 2 unwrap() موثَّقان في كود الإنتاج · CI أخضر عبر 4 تشغيلات متتالية (`b7ee05d`، `6351891`، `430e6ce`، `f1cb6bd`).

للادّعاءات غير المُعاد فحصها، راجع القسم أعلاه صراحةً.

---

## 2. ما الجديد في v1.9 (Phase 2)

### إنجاز رئيسي: توحيد سياسة تصنيف الاختبارات

قبل Phase 2، كانت اختبارات Docker موزعة على 3 آليات مختلفة:
- 14 اختبار في `week14_gate.rs` تحت `slow_tests` (تسمية مضلِّلة تاريخيًا)
- 2 اختبار في `week16`/`week18` تحت `#[ignore]` ثابت
- **30 اختبار Docker يعمل بلا أي حماية** — السبب الجذري لتذبذب CI السابق

### الإصلاح (commits `430e6ce` + `f1cb6bd`)

**التصنيف الجديد الحصري:**
| الفئة | الآلية | العدد |
|---|---|---|
| A. سريع، لا Docker | بلا وسم — يعمل دومًا | 21 اختبار |
| B. `docker_tests` | `cfg_attr(not(feature))` مع feature منفصل | 44 اختبار |
| C. `slow_tests` | `cfg_attr` منفصل، حصري (لا يتراكم مع B) | 2 اختبار |

**التغييرات الملموسة:**
- إضافة `docker_tests = []` و`slow_tests = []` إلى `ec-sandbox/Cargo.toml` و`ec-app/Cargo.toml`
- تصنيف 46 اختبار Docker في 6 ملفات بعد فحص فردي لجسم كل دالة
- تحديث `ci.yml`: `test-docker` job يستخدم `--features docker_tests` صراحة، و`test-fast` أُزيل منه `--exclude ec-sandbox --exclude ec-app`
- إنشاء `phase2_verify.sh` كحارس آلي ضد أي تراجع مستقبلي
- توثيق القرار في ADR-021

**درس منهجي:** التسمية `slow_tests` القديمة كانت مضلِّلة — الاختبارات ليست بطيئة بحد ذاتها، بل تحتاج Docker. فصل البُعدين ("يحتاج Docker" ≠ "بطيء زمنيًا") جعل السياسة واضحة ومنعت تسرّب اختبارات بلا حماية.

### أرقام التحقق النهائية (كلها من سجل خام مباشر)

| السيناريو | passed | failed | ignored | المصدر |
|---|---:|---:|---:|---|
| بلا features (test-fast) | 645 | 0 | 46 | CI run #6 + phase2_verify.sh محليًا |
| ec-sandbox --features docker_tests | 141 | 0 | 1 | CI test-docker + phase2_verify.sh |
| ec-app --features docker_tests | 101 | 0 | 1 | CI test-docker + phase2_verify.sh |

الـ ignored الوحيد في كل من ec-sandbox وec-app هو `gate_zero_escapes_in_100_executions` (اختبار stress 100 تكرار، مصنَّف `slow_tests`، يُشغَّل يدويًا فقط قبل الإصدارات).

---

## 3. الـ Crates — مرجع بنيوي (لم يتغيّر منذ v1.8)

راجع v1.8 لتفصيل كل crate. Phase 2 لم تُغيّر أي منطق، فقط سياسة تشغيل الاختبارات.

---

## 4. البنود المفتوحة (منقولة صراحة للمراحل القادمة)

| البند | الأولوية | المرحلة المقترحة |
|---|---|---|
| `gate_network_isolation` — لم يتذبذب بعد عزله خلف `docker_tests` + `--test-threads=1` (نجح مرتين: محليًا وعلى CI). دليل داعم للفرضية، ليس إغلاقًا | منخفضة (كانت متوسطة) | مراقبة إضافية عبر 3-5 تشغيلات CI قادمة قبل الإغلاق النهائي |
| 2 `unwrap()` في `ec-cli/src/main.rs` | منخفضة | Phase 3 |
| إعادة فحص `ec check` على `sel-agent-v4` | متوسطة | Phase 4 |
| عدّ ADRs والـ invariants الفعلي | منخفضة | Phase 5 |
| إضافة CI badge إلى README (لا يوجد README أصلًا) | منخفضة | Phase 5 |
| فحص Annotations الخام لـ`f1cb6bd` (تأكيد "0 تحذيرات CI" حديثًا لا موروثًا) | منخفضة | متى تسنّى |

---

## 5. أوامر الصيانة

```bash
# التشغيل السريع (بلا Docker)
cargo build --workspace --locked
cargo clippy --workspace --tests --locked -- -D warnings
cargo test --workspace --no-fail-fast --locked

# اختبارات Docker (تحتاج Docker daemon)
cargo test -p ec-sandbox --locked --features docker_tests -- --test-threads=1
cargo test -p ec-app --locked --features docker_tests -- --test-threads=1

# التحقق الآلي من سياسة Phase 2
./phase2_verify.sh

# اختبارات stress اليدوية (10 دقائق تقريبًا لكل واحد، لا تعمل في CI)
cargo test -p ec-sandbox --locked --features slow_tests gate_zero_escapes_in_100_executions -- --exact
cargo test -p ec-app --locked --features slow_tests gate_zero_escapes_in_100_executions -- --exact
راجع .github/workflows/ci.yml للتشغيل الآلي المطابق.

نهاية الوثيقة المرجعية — Engineering Civilization v1.9 · Phase 2 CLOSED (2026-07-13)
