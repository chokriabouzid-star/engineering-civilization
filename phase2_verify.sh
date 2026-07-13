#!/usr/bin/env bash
# phase2_verify.sh — التحقق الآلي من سياسة docker_tests/slow_tests (ADR-021)

set -uo pipefail

TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
OUT_DIR="phase2_artifacts"
REPORT="PHASE2_REPORT.md"
fail=0

mkdir -p "$OUT_DIR" || { echo "خطأ: لا يمكن إنشاء $OUT_DIR" >&2; exit 2; }

echo "=== Phase 2 Verification — $TS ==="

if [ ! -f "Cargo.toml" ] || ! grep -q "\[workspace\]" Cargo.toml; then
    echo "خطأ: شغّل من جذر المستودع." >&2
    exit 2
fi

{
    echo "# تقرير التحقق — Phase 2 (سياسة docker_tests/slow_tests)"
    echo ""
    echo "## بيئة التشغيل"
    echo '```'
    echo "التاريخ (UTC): $TS"
    echo "rustc: $(rustc --version 2>&1)"
    echo "git commit: $(git rev-parse HEAD 2>&1)"
    echo '```'
} > "$REPORT"

# دالة مساعدة: تفحص أن cargo test لم ينهار قبل تنفيذ أي اختبار.
# false-negative guard: passed+failed+ignored > 0 يعني أن اختبارات فُحصت فعلًا.
sanity_check_ran() {
    local label="$1" total="$2"
    if [ "$total" -eq 0 ]; then
        echo "❌ $label: لم يُنفَّذ أي اختبار (compile fail؟ Docker down؟ راجع السجل)"
        fail=1
    fi
}

# --- 1. بلا features ---
echo "1/3: تشغيل بلا features ..."
cargo test --workspace --locked --no-fail-fast > "$OUT_DIR/no_features.log" 2>&1
NF_EXIT=$?
NF_PASSED=$(grep -oE "[0-9]+ passed" "$OUT_DIR/no_features.log" | awk '{s+=$1} END{print s+0}')
NF_FAILED=$(grep -oE "[0-9]+ failed" "$OUT_DIR/no_features.log" | awk '{s+=$1} END{print s+0}')
NF_IGNORED=$(grep -oE "[0-9]+ ignored" "$OUT_DIR/no_features.log" | awk '{s+=$1} END{print s+0}')
NF_TOTAL=$((NF_PASSED + NF_FAILED + NF_IGNORED))

sanity_check_ran "بلا features" "$NF_TOTAL"
if [ "$NF_FAILED" -ne 0 ]; then
    echo "❌ بلا features: $NF_FAILED فشل غير متوقَّع"
    fail=1
fi
if [ "$NF_EXIT" -ne 0 ] && [ "$NF_FAILED" -eq 0 ]; then
    echo "❌ بلا features: cargo exit=$NF_EXIT بلا failed مرصود — انهيار خارج الاختبارات"
    fail=1
fi

# قائمة الـ44 الصريحة (نفس الأصلية، دون تغيير)
DOCKER_TEST_NAMES=(
    hardened_compiles_hello_world hardened_blocks_proc_sysrq hardened_blocks_dev_mem
    hardened_blocks_ptrace hardened_blocks_mount hardened_contains_fork_bomb
    docker_available docker_runs_echo docker_network_is_isolated
    docker_workspace_tmpfs_is_writable docker_compiles_hello_world docker_pids_limit_blocks_fork_bomb
    docker_compiles_and_runs_hello_world docker_fails_on_invalid_code
    docker_measures_real_latency docker_reproducibility_from_real_runs
    gate_docker_compiles_and_runs_real_rust gate_reality_vector_from_real_execution
    gate_correct_program_produces_trustworthy_reality gate_compilation_failure_handled
    gate_real_latency_measured gate_reproducibility_from_hash_comparison
    gate_empirical_confidence_from_runs gate_escape_vector_1_proc_sysrq
    gate_escape_vector_2_mount_syscall gate_escape_vector_3_ptrace
    gate_escape_vector_4_dev_mem gate_escape_vector_5_fork_bomb
    gate_zero_escapes_in_20_executions week14_gate_complete
    gate_hardened_compiles_and_runs gate_hardened_runs_as_non_root
    gate_read_only_filesystem_prevents_writes gate_workspace_tmpfs_writable
    gate_escape_vector_1_proc_sysrq_blocked gate_escape_vector_2_dev_mem_blocked
    gate_escape_vector_3_ptrace_proc_mem_blocked gate_escape_vector_4_mount_blocked
    gate_escape_vector_5_fork_bomb_contained gate_network_remains_isolated_in_hardened_mode
    week16_gate_complete gate_hardened_escape_vectors_contained
    gate_network_isolation phase2_gate_complete
)
DOCKER_LEAK=""
for name in "${DOCKER_TEST_NAMES[@]}"; do
    hit=$(grep -E "^test [a-zA-Z0-9_:]*\b${name}\b .* ok$" "$OUT_DIR/no_features.log" || true)
    if [ -n "$hit" ]; then
        DOCKER_LEAK="${DOCKER_LEAK}${hit}"$'\n'
    fi
done
if [ -n "$DOCKER_LEAK" ]; then
    echo "❌ تسرّب: اختبار Docker يعمل بلا docker_tests feature:"
    echo "$DOCKER_LEAK"
    fail=1
fi

# --- 2. ec-sandbox --features docker_tests ---
echo "2/3: تشغيل ec-sandbox --features docker_tests ..."
cargo test -p ec-sandbox --locked --no-fail-fast --features docker_tests -- --test-threads=1 \
    > "$OUT_DIR/sandbox_docker.log" 2>&1
SB_EXIT=$?
SB_PASSED=$(grep -oE "[0-9]+ passed" "$OUT_DIR/sandbox_docker.log" | awk '{s+=$1} END{print s+0}')
SB_FAILED=$(grep -oE "[0-9]+ failed" "$OUT_DIR/sandbox_docker.log" | awk '{s+=$1} END{print s+0}')
SB_IGNORED=$(grep -oE "[0-9]+ ignored" "$OUT_DIR/sandbox_docker.log" | awk '{s+=$1} END{print s+0}')
SB_TOTAL=$((SB_PASSED + SB_FAILED + SB_IGNORED))

sanity_check_ran "ec-sandbox --features docker_tests" "$SB_TOTAL"
if [ "$SB_FAILED" -ne 0 ]; then
    echo "❌ ec-sandbox: $SB_FAILED فشل"
    fail=1
fi
if [ "$SB_EXIT" -ne 0 ] && [ "$SB_FAILED" -eq 0 ]; then
    echo "❌ ec-sandbox: cargo exit=$SB_EXIT بلا failed مرصود — انهيار خارج الاختبارات"
    fail=1
fi
if [ "$SB_IGNORED" -ne 1 ]; then
    echo "❌ ec-sandbox: متوقَّع 1 ignored (slow_tests وحده)، الفعلي: $SB_IGNORED"
    fail=1
fi

# --- 3. ec-app --features docker_tests ---
echo "3/3: تشغيل ec-app --features docker_tests ..."
cargo test -p ec-app --locked --no-fail-fast --features docker_tests -- --test-threads=1 \
    > "$OUT_DIR/app_docker.log" 2>&1
APP_EXIT=$?
APP_PASSED=$(grep -oE "[0-9]+ passed" "$OUT_DIR/app_docker.log" | awk '{s+=$1} END{print s+0}')
APP_FAILED=$(grep -oE "[0-9]+ failed" "$OUT_DIR/app_docker.log" | awk '{s+=$1} END{print s+0}')
APP_IGNORED=$(grep -oE "[0-9]+ ignored" "$OUT_DIR/app_docker.log" | awk '{s+=$1} END{print s+0}')
APP_TOTAL=$((APP_PASSED + APP_FAILED + APP_IGNORED))

sanity_check_ran "ec-app --features docker_tests" "$APP_TOTAL"
if [ "$APP_FAILED" -ne 0 ]; then
    echo "❌ ec-app: $APP_FAILED فشل"
    fail=1
fi
if [ "$APP_EXIT" -ne 0 ] && [ "$APP_FAILED" -eq 0 ]; then
    echo "❌ ec-app: cargo exit=$APP_EXIT بلا failed مرصود — انهيار خارج الاختبارات"
    fail=1
fi
if [ "$APP_IGNORED" -ne 1 ]; then
    echo "❌ ec-app: متوقَّع 1 ignored (slow_tests وحده)، الفعلي: $APP_IGNORED"
    fail=1
fi

{
echo ""
echo "## النتائج"
echo ""
echo "| السيناريو | passed | failed | ignored | cargo exit |"
echo "|---|---|---|---|---|"
echo "| بلا features | $NF_PASSED | $NF_FAILED | $NF_IGNORED | $NF_EXIT |"
echo "| ec-sandbox --features docker_tests | $SB_PASSED | $SB_FAILED | $SB_IGNORED | $SB_EXIT |"
echo "| ec-app --features docker_tests | $APP_PASSED | $APP_FAILED | $APP_IGNORED | $APP_EXIT |"
echo ""
echo "**المتوقَّع وفق ADR-021:**"
echo "- بلا features: 645 passed / 0 failed / 46 ignored / exit=0"
echo "- ec-sandbox docker_tests: 141 passed / 0 failed / 1 ignored / exit=0"
echo "- ec-app docker_tests: 101 passed / 0 failed / 1 ignored / exit=0"
if [ -n "$DOCKER_LEAK" ]; then
    echo ""
    echo "## ⚠️ تسرّب اختبار Docker (يعمل بلا حماية)"
    echo '```'
    echo "$DOCKER_LEAK"
    echo '```'
fi
echo ""
echo "## الحكم النهائي"
if [ "$fail" -eq 0 ]; then
    echo "✅ السياسة مطبَّقة وتعمل كما هو متوقَّع — لا تسرّب، لا فشل، لا انهيار."
else
    echo "❌ خلل: راجع الأقسام أعلاه والسجلات الخام في \`$OUT_DIR/\`."
fi
} >> "$REPORT"

echo ""
echo "=== انتهى. التقرير: $REPORT | السجلات: $OUT_DIR/ ==="
exit "$fail"
