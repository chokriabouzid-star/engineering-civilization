# ADR-021: سياسة موحّدة لتصنيف اختبارات Docker والاختبارات البطيئة

**الحالة:** مقترح → جاهز للاعتماد بعد مراجعة بشرية نهائية  
**التاريخ:** 2026-07-10  
**السياق:** Phase 2 — بعد اكتشاف عدم اتساق آليات `#[ignore]`/`cfg_attr` عبر `ec-sandbox` و`ec-app` أثناء العمل على استقرار CI (Phase 1).

---

## 1. المشكلة (موثَّقة بالأدلة، لا بالانطباع)

بفحص فعلي لكل دالة `#[test]` عبر 6 ملفات (`hardened.rs`, `docker.rs`, `executor.rs`, `week14_gate.rs`, `week16_gate.rs`, `week18_phase2_gate.rs`) — 67 دالة إجمالاً — تبيّن:

| الآلية المستخدمة | أين | العدد |
|---|---|---|
| `#[cfg_attr(not(feature="slow_tests"), ignore)]` | `week14_gate.rs` فقط | 14 اختبارًا — **كلها Docker فعليًا، رغم التسمية "slow"** |
| `#[ignore = "..."]` ثابت (لا feature) | `week16_gate.rs`, `week18_phase2_gate.rs` | 2 اختبارًا (100-تكرار لكل منهما) |
| بلا أي حماية — يعمل تلقائيًا كلما توفّر Docker | `hardened.rs`, `docker.rs`, `executor.rs`, معظم `week16`/`week18` | **30 اختبارًا** |

**النتيجة:** من أصل 46 اختبارًا يستدعي Docker فعليًا في جسمه، 30 منها تعمل بلا أي حماية في أي بيئة، وهذا كان سبب غياب سياسة تشغيل موحّدة.

كذلك، تسمية `slow_tests` في `week14_gate.rs` مضلِّلة: لا علاقة لها بالبطء الزمني، هي فعليًا "يحتاج Docker" بمصادفة تاريخية.

## 2. القرار

نفصل بين بُعدين مختلفين منطقيًا:

- **"يحتاج Docker؟"** — هل الدالة تستدعي `DockerRunner`/`HardenedDockerRunner` فعليًا؟
- **"بطيء زمنيًا؟"** — هل يستغرق دقائق، لا ثوانٍ؟

لكن على مستوى **كل اختبار بعينه**، يكون التصنيف **حصريًا**: كل اختبار يأخذ **وسمًا واحدًا فقط** من الفئات التالية:

| الفئة | الآلية | الشرط |
|---|---|---|
| **A. سريع، لا Docker** | بلا أي وسم — يعمل دومًا | لا يستدعي Docker إطلاقًا |
| **B. `docker_tests`** | `#[cfg_attr(not(feature = "docker_tests"), ignore = "requires --features docker_tests")]` | يستدعي Docker وينتهي خلال ثوانٍ |
| **C. `slow_tests`** | `#[cfg_attr(not(feature = "slow_tests"), ignore = "requires --features slow_tests")]` | اختبار stress طويل (~10 دقائق) — **حصري** |

**لماذا الحصرية بين B وC؟**  
لو أخذ اختبار الـ100-تكرار كلا الوسمين، فسيتطلب تفعيل الميزتين معًا. هذا يعقّد التشغيل ويكسر المعنى العملي لـ`docker_tests`. لذلك جُعلت `slow_tests` فئة مستقلة حصرية.

### التصنيف النهائي المُطبَّق

- **21 دالة** فئة A
- **44 دالة** فئة B (`docker_tests`)
- **2 دالة** فئة C (`slow_tests`)

### سلوك CI الناتج

```bash
cargo test --workspace --locked                         # test-fast: فئة A تعمل، وDocker tests تُتجاهل تلقائيًا
cargo test -p ec-sandbox --features docker_tests ...   # Docker tests في CI
cargo test -p ec-app     --features docker_tests ...   # Docker tests في CI
cargo test --workspace --features slow_tests           # يدوي فقط قبل الإصدارات
```

## 3. البدائل المرفوضة

- **دمج `docker_tests` و`slow_tests` كوسمين متراكبين:** مرفوض — يعقّد التشغيل بلا فائدة عملية.
- **الإبقاء على `slow_tests` كاسم وحيد لكل اختبارات Docker:** مرفوض — هذا هو الوضع الحالي المضلِّل.
- **`#[cfg(feature = "...")] mod` على مستوى الوحدة:** مرفوض — يُخفي الكود عن الترجمة ويزيد خطر التعفّن الصامت.

## 4. الأثر

- `Cargo.toml` في `ec-sandbox` و`ec-app`: إضافة `docker_tests = []` و`slow_tests = []`.
- `ci.yml`: تشغيل `--features docker_tests` صراحة في job `test-docker`.
- `test-fast`: أُزيل `--exclude ec-sandbox --exclude ec-app` نهائيًا. اختبارات فئة A داخل هاتين الحزمتين تعمل الآن ضمن `test-fast` مباشرة؛ اختبارات Docker تُتجاهَل تلقائيًا فيه، وتعمل حصريًا في `test-docker`.
- لا تغيير في منطق الاختبارات، فقط في سياسة التشغيل.

---
*هذه الوثيقة توثّق قرار التصنيف فقط؛ أرقام التحقق التشغيلية النهائية تُثبَّت من مخرجات raw بعد التطبيق، لا من هذه الوثيقة نفسها.*
