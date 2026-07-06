#!/usr/bin/env bash
# phase0_verify.sh — Engineering Civilization: التحقق الآلي من المرحلة 0
#
# الغرض: تشغيل بناء + اختبار + clippy بشكل نظيف تمامًا (بلا كاش يخفي الأخطاء)،
# وتوليد تقرير واحد (PHASE0_REPORT.md) يحتوي أرقامًا مقاسة فعليًا فقط،
# لا أرقامًا مكتوبة يدويًا.
#
# الاستخدام:
#   1. ضع هذا الملف و phase0_fix.patch في جذر مستودع Engineering Civilization
#   2. chmod +x phase0_verify.sh
#   3. ./phase0_verify.sh
#
# يجب أن يُشغَّل من جذر المستودع (بجانب Cargo.toml الرئيسي للـ workspace).

set -uo pipefail

TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
OUT_DIR="phase0_artifacts"
REPORT="PHASE0_REPORT.md"
fail=0
docker_available=0

echo "=== Phase 0 Verification — $TS ==="

# --- 0. sanity check: هل نحن في المكان الصحيح؟ ---
if [ ! -f "Cargo.toml" ] || ! grep -q "\[workspace\]" Cargo.toml; then
    echo "خطأ: شغّل هذا السكريبت من جذر مستودع Engineering Civilization (بجانب Cargo.toml)." >&2
    exit 2
fi

mkdir -p "$OUT_DIR"

# --- 1. التحقق من تطبيق إصلاح week6_meta.rs (شرط لا يُتجاوز) ---
if grep -q "HighRejectionRate(1.0)" crates/ec-constitutional/tests/week6_meta.rs 2>/dev/null; then
    echo "❌ لم يُطبَّق إصلاح week6_meta.rs بعد." >&2
    echo "طبّقه أولًا: git apply phase0_fix.patch" >&2
    exit 3
fi
FIX_STATUS="✅ مطبَّق"

# --- 2. معلومات البيئة (لإثبات إمكانية إعادة الإنتاج) ---
{
    echo "# تقرير التحقق — المرحلة 0"
    echo ""
    echo "## بيئة التشغيل"
    echo '```'
    echo "التاريخ (UTC): $TS"
    echo "rustc:         $(rustc --version 2>&1)"
    echo "cargo:         $(cargo --version 2>&1)"
    echo "cargo-clippy:  $(cargo clippy --version 2>&1)"
    echo "OS:            $(uname -a)"
    echo "git commit:    $(git rev-parse HEAD 2>&1)"
    echo "git status:"
    git status --porcelain=v1 2>&1 || echo "(ليس مستودع git)"
    echo '```'
    echo ""
    echo "## فحص إصلاح week6_meta.rs"
    echo "$FIX_STATUS"
} > "$REPORT"

# --- 3. تنظيف كامل (بلا كاش) ---
echo "تنظيف target/ ..."
rm -rf target/

# --- 4. البناء ---
echo "cargo build --workspace --locked ..."
if cargo build --workspace --locked > "$OUT_DIR/build.log" 2>&1; then
    BUILD_STATUS="✅ نجح"
else
    BUILD_STATUS="❌ فشل (راجع $OUT_DIR/build.log)"
    fail=1
fi
tail -40 "$OUT_DIR/build.log"

# --- 5. الاختبارات (بلا توقف عند أول فشل) ---
echo "cargo test --workspace --no-fail-fast --locked ..."
cargo test --workspace --no-fail-fast --locked > "$OUT_DIR/test.log" 2>&1
tail -60 "$OUT_DIR/test.log"

TOTAL_PASSED=$(grep -oE "[0-9]+ passed; [0-9]+ failed" "$OUT_DIR/test.log" | awk '{sum+=$1} END{print sum+0}')
TOTAL_FAILED=$(grep -oE "[0-9]+ passed; [0-9]+ failed" "$OUT_DIR/test.log" | awk '{sum+=$3} END{print sum+0}')
FAILED_TEST_NAMES=$(grep -E "^test .* FAILED$" "$OUT_DIR/test.log" | sed -E 's/^test (.*) \.\.\. FAILED$/\1/')

# --- 6. توفر Docker (حقيقة تُذكر بجانب الفشل، بلا أي تخمين آلي لسبب الفشل) ---
# ملاحظة مهمة: لا يوجد تصنيف تلقائي موثوق لـ"هل هذا الفشل بسبب Docker أم لا"
# بالاعتماد على اسم الاختبار فقط (مثال حقيقي وقع أثناء اختبار هذا السكريبت:
# compiler::tests::compiles_hello_world يفشل بسبب غياب Docker لكن اسمه لا يحتوي
# كلمة docker إطلاقًا). لذلك أي تخمين آلي هنا كان سيكون ادّعاءً كاذبًا بحد ذاته.
# القرار: كل فشل يُدرَج كاملاً، ويُراجَع يدويًا بمطابقة اسم الاختبار مع سبب
# الفشل الفعلي المطبوع في $OUT_DIR/test.log (ابحث عن السطر الذي يسبق اسم كل
# اختبار فاشل لمعرفة رسالة الخطأ الحقيقية).
if command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then
    docker_available=1
fi

# --- 7. clippy ---
echo "cargo clippy --workspace --tests --locked -- -D warnings ..."
if cargo clippy --workspace --tests --locked -- -D warnings > "$OUT_DIR/clippy.log" 2>&1; then
    CLIPPY_STATUS="✅ 0 تحذيرات"
else
    CLIPPY_STATUS="❌ فشل / توجد تحذيرات (راجع $OUT_DIR/clippy.log)"
    fail=1
fi
tail -60 "$OUT_DIR/clippy.log"

# --- 8. fmt (إعلامي فقط، غير حاسم) ---
if cargo fmt --all -- --check > "$OUT_DIR/fmt.log" 2>&1; then
    FMT_STATUS="✅ متوافق"
else
    FMT_STATUS="⚠️ يوجد فروقات تنسيق (راجع $OUT_DIR/fmt.log)"
fi

# --- 9. unwrap() في كود الإنتاج فقط (تقريب: يستبعد ما بعد أول #[cfg(test)] بكل ملف) ---
UNWRAP_REPORT=$(python3 - <<'PYEOF' 2>/dev/null || echo "TOTAL=N/A (python3 غير متاح)"
import glob
files = glob.glob("crates/*/src/**/*.rs", recursive=True)
hits = []
for f in files:
    with open(f, encoding="utf-8", errors="replace") as fh:
        lines = fh.readlines()
    test_mod_start = None
    for i, l in enumerate(lines):
        if "#[cfg(test)]" in l:
            test_mod_start = i
            break
    for i, l in enumerate(lines):
        stripped = l.strip()
        if stripped.startswith("//"):
            continue
        if ".unwrap()" in l and "unwrap_calls" not in l and '".unwrap()"' not in l:
            if test_mod_start is not None and i >= test_mod_start:
                continue
            hits.append(f"{f}:{i+1}: {l.strip()}")
for h in hits:
    print(h)
print(f"TOTAL={len(hits)}")
PYEOF
)
UNWRAP_TOTAL=$(echo "$UNWRAP_REPORT" | grep "^TOTAL=" | cut -d= -f2)

# --- 10. مقاييس عامة ---
LOC=$(find crates -name "*.rs" 2>/dev/null | xargs wc -l 2>/dev/null | tail -1 | awk '{print $1}')
TEST_ATTR_COUNT=$(grep -rn '#\[test\]' crates --include=*.rs 2>/dev/null | wc -l)

# --- كتابة التقرير النهائي ---
{
echo ""
echo "## نتائج القياس الفعلي (وليست ادّعاءات)"
echo ""
echo "| البند | القيمة المقاسة |"
echo "|---|---|"
echo "| البناء (cargo build --workspace) | $BUILD_STATUS |"
echo "| الاختبارات: نجح | $TOTAL_PASSED |"
echo "| الاختبارات: فشل | $TOTAL_FAILED |"
echo "| Docker متاح في هذه البيئة | $([ $docker_available -eq 1 ] && echo "نعم" || echo "لا") |"
echo "| clippy (-D warnings, --tests) | $CLIPPY_STATUS |"
echo "| cargo fmt --check | $FMT_STATUS |"
echo "| unwrap() في كود إنتاج (تقريبي) | $UNWRAP_TOTAL |"
echo "| إجمالي أسطر Rust (crates/**/*.rs) | $LOC |"
echo "| عدد #[test] الفعلي | $TEST_ATTR_COUNT |"
echo ""
if [ -n "$FAILED_TEST_NAMES" ]; then
    echo "### الاختبارات الفاشلة (الاسم الكامل) — راجعها يدويًا واحدًا واحدًا"
    echo "لا تصنيف آلي هنا لأنه غير موثوق (انظر الملاحظة في السكريبت). لكل اسم أدناه:"
    echo "ابحث عنه في $OUT_DIR/test.log وتحقق هل رسالة الخطأ الفعلية مرتبطة بـ Docker أم لا."
    echo '```'
    echo "$FAILED_TEST_NAMES"
    echo '```'
fi
echo ""
echo "### تفاصيل unwrap() في كود الإنتاج"
echo '```'
echo "$UNWRAP_REPORT"
echo '```'
echo ""
echo "## الحكم النهائي"
if [ "$fail" -eq 0 ] && [ "$TOTAL_FAILED" -eq 0 ]; then
    echo "✅ المرحلة 0 مكتملة تلقائيًا: البناء نظيف، clippy بلا تحذيرات، 0 فشل اختبارات."
elif [ "$fail" -eq 0 ] && [ "$TOTAL_FAILED" -gt 0 ]; then
    echo "⚠️ لا حكم تلقائي ممكن: البناء وclippy ناجحان، لكن $TOTAL_FAILED اختبارًا فشل."
    echo "راجع كل اسم في قسم \"الاختبارات الفاشلة\" أعلاه يدويًا مقابل \$OUT_DIR/test.log."
    echo "لا تُغلَق هذه المرحلة إلا بعد أن تحدّد بنفسك (أو ترسل القائمة لتُراجَع) أن كل"
    echo "فشل مردّه فعليًا لغياب Docker (Docker متاح هنا: $([ $docker_available -eq 1 ] && echo نعم || echo لا)) وليس خللًا حقيقيًا آخر."
else
    echo "❌ المرحلة 0 غير مكتملة: فشل في البناء أو clippy. راجع السجلات في $OUT_DIR/."
fi
} >> "$REPORT"

echo ""
echo "=== انتهى. التقرير في: $REPORT | السجلات الخام في: $OUT_DIR/ ==="
