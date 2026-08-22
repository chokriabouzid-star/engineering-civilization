# Engineering Civilization — الوثيقة المرجعية

## v1.9.4 · Phase 4 CLOSED (Audit Hardening) · 2026-08-22

هذه الوثيقة مرجع حالة مضغوط ودقيق. لا تحتوي أرقامًا إلا إذا كانت متحققة مباشرة من تشغيل فعلي.

---

## ⚠️ حالة التحقق الحالية (Phase 4 Mapped)

| البند | الحالة | الدليل التنفيذي الفعلي |
|---|---|---|
| `cargo build --workspace --locked` | ✅ | تشغيل فعلي ناجح (صفر أخطاء) |
| `cargo clippy --workspace --tests --locked -- -D warnings` | ✅ | صفر تحذيرات |
| `cargo fmt --all -- --check` | ✅ | نظيف بالكامل |
| `cargo test --workspace --locked --no-fail-fast` | ✅ | **666 passed / 0 failed / 46 ignored** |
| `cargo test -p ec-sandbox --features docker_tests` | ✅ | **713 passed / 0 failed / 1 ignored** (إجمالي الاختبارات المكتشفة مع Docker) |
| `cargo run --bin ec -- check .` | ✅ | **130 scanned / 128 passed / 2 failed / score=0.916** (Exit Code = 1) |
| المصادقة والأمان في API | ✅ | `X-API-Key` إجباري على كل المسارات المحدثة |
| حصانة الـ Sandbox | ✅ | `HardenedDockerRunner` إلزامي ومباشر في الإنتاج |
| نقاء الـ Kernel | ✅ | `ec-constitutional` نقي 100% وخالٍ من `tokio` و `async` |
| عدد وثائق الـ ADRs | ✅ | **18 ADR** رسمية ومستقلة |

---

## 1. ما الجديد في v1.9.4 (Phase 4)

تمت معالجة وإغلاق كافة الفجوات الفنية والأمنية الـ 10 التي كشفها التدقيق العدائي المستقل:
1. **F1**: إلزامية الحصانة لـ Docker في مسار الإنتاج وإلغاء المسار غير المقوّى.
2. **F2**: إغلاق الوصول المفتوح لـ API عبر فرض مصادقة `X-API-Key`.
3. **F3**: تعديل `ec check` ليخرج بـ `Exit Code = 1` عند الانتهاكات، وتفعيله استشارياً في CI.
4. **F4**: ربط تغطية الـ Crate بوجود `assert_count > 0` لمنع التغطية الشكليّة الفارغة.
5. **F5**: استعادة نقاء الـ Kernel في `ec-constitutional` بحذف `tokio`.
6. **F6**: استخراج `ADR-005` لملف مستقل.
7. **F7**: تنظيف شامل لملفات النتائج والبقايا وتوسيع `.gitignore`.
8. **F9**: دعم التوافق مع Windows باستخدام `tempfile`.
9. **F10**: توضيح التسمية الإحصائية لـ `BayesianTracker`.

تفاصيل القرارات المعمارية كاملة في **`docs/adr/ADR-024-multi-model-audit-resolutions.md`**.

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
| `ec-app` | تكامل النظام بالكامل | I/O |

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
نهاية الوثيقة المرجعية — Engineering Civilization v1.9.4 (2026-08-22)
