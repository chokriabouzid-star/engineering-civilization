# ADR-014: Static Code Analysis — Week 19

## Status
✅ Accepted — 2024

## Context

Phase 2 اكتمل بـ 230 test، لكن مع hack خطير:

```rust
// ec-app/src/pipeline.rs (Week 17)
pub fn estimate_fitness_from_reality(reality: &RealityVector) -> FitnessVector {
    // تقدير من نتيجة التنفيذ — ليس صحيحاً
    let base = if reality.is_correct() { 0.8 } else { 0.3 };
    // ...
}
المشكلة:

يخترق Truth ≠ Fitness إذا لم يُزال
FitnessVector يجب أن يُقاس من الكود، لا من نتيجة التنفيذ
كان placeholder لـ Phase 2، لا يصلح لـ Phase 3
الحل:
Static code analysis حقيقي يُنتج FitnessVector من الكود مباشرة.

Decision
إنشاء ec-analysis
text

ec-analysis/
  ├── analyzer.rs      ← analyze_code() → FitnessVector
  ├── security.rs      ← unsafe, unwrap, panic detection
  ├── coverage.rs      ← #[test] function counting
  ├── complexity.rs    ← cyclomatic complexity
  ├── reversibility.rs ← side-effect detection
  └── metrics.rs       ← shared utilities
API الرئيسي
Rust

pub fn analyze_code(code: &str) -> FitnessVector
Design Rule: Truth ≠ Fitness محفوظ:

RealityVector = نتيجة التنفيذ (ec-sandbox)
FitnessVector = خصائص الكود (ec-analysis)
لا أحدهما يُشتق من الآخر
Implementation
1. Security Metrics
Rust

pub struct SecurityMetrics {
    pub unsafe_blocks: usize,
    pub unwrap_calls: usize,
    pub expect_calls: usize,
    pub panic_calls: usize,
}

impl SecurityMetrics {
    pub fn from_code(code: &str) -> Self
    pub fn score(&self) -> f64  // 1.0 - penalties
    pub fn is_safe(&self) -> bool
}
Scoring:

1.0 = no issues
unsafe: -0.3 each
unwrap/expect: -0.05 each
panic!: -0.1 each
Floor: 0.0
2. Coverage Metrics
Rust

pub struct CoverageMetrics {
    pub total_functions: usize,
    pub test_functions: usize,
}

impl CoverageMetrics {
    pub fn score(&self) -> f64  // test_functions / total_functions
}
Scoring:

No functions → 0.5 (neutral)
Functions without tests → 0.0
All tested → 1.0
3. Complexity Metrics
Rust

pub struct ComplexityMetrics {
    pub cyclomatic: usize,
    pub function_count: usize,
}

impl ComplexityMetrics {
    pub fn avg_complexity(&self) -> f64
    pub fn maintainability_score(&self) -> f64
}
Cyclomatic complexity:

Count: if, match, &&, ||, while, for, loop, ?
Score: 1.0 / (1.0 + (avg - 1) * 0.1)
4. Reversibility Metrics
Rust

pub struct ReversibilityMetrics {
    pub side_effects: usize,
}

impl ReversibilityMetrics {
    pub fn score(&self) -> f64
    pub fn is_pure(&self) -> bool
}
Side-effect detection:

println!, print!, eprintln!
fs::, File::, Command::
std::net::, TcpStream::, UdpSocket::
Score: 1.0 - (side_effects * 0.05), floor 0.2
5. Performance Score
Rust

fn performance_score(code: &str) -> f64 {
    let allocations = count(String::, Vec::, Box::, HashMap::,
                           clone(), to_string(), format!);
    (1.0 - allocations * 0.03).clamp(0.3, 1.0)
}
6. Architectural Stability
Rust

fn architectural_score(code: &str) -> f64 {
    let uses = count("use ");
    (1.0 - uses * 0.02).clamp(0.4, 1.0)
}
Integration with Pipeline
Rust

// ec-app/src/pipeline.rs — Week 19 update

pub fn run(&mut self, artifact_id: &str, code: &str) -> PipelineResult {
    // ...
    
    // ─── Step 2: Code → Fitness (static analysis) ──────────────
    // Week 19: FitnessVector من تحليل الكود، لا من RealityVector
    let fitness = analyze_code(code);  // ← من ec-analysis
    
    // ─── Step 3: Reality → Epistemic ────────────────────────────
    let epistemic = match &execution.reality {
        Some(reality) => build_epistemic_from_reality(reality),  // ← منفصل
        None => default_epistemic(),
    };
    
    // ...
}
الفصل الواضح:

analyze_code(code) → FitnessVector (خصائص الكود)
build_epistemic_from_reality(reality) → EpistemicState (نتيجة التنفيذ)
لا علاقة بينهما
Limitations
1. Pattern Matching
Rust

count_pattern(code, "unsafe")  // بدائي
Known Issues:

String literals: "unsafe block" يُعد unsafe
Comments: // unsafe code يُعد unsafe
False positives ممكنة
Mitigation:

التحليل conservative: يُفضّل false positive على false negative
الـ penalty صغير (-0.3) لا كارثي
Week 20+ سيُحسّن بـ syn crate (AST parsing)
2. Test Coverage
Rust

count("#[test]")  // لا يعرف code coverage حقيقي
Approximation:

عدد #[test] functions / إجمالي functions
لا يعرف أي functions مُختبرة
يُحفّز كتابة اختبارات لكن لا يضمن جودتها
3. Cyclomatic Complexity
Rust

count("if ") + count("match ")  // تقريبي
Approximation:

يعد branches لكن لا يحسب paths
nested complexity لا يُوزن أكثر
كافٍ لـ Phase 3، يُحسّن لاحقاً
Testing
Coverage
text

ec-analysis:
  unit tests:  30 ✅
  gate tests:  20 ✅
  ────────────────
  total:       50 tests
  
Scenarios tested:
  ✓ Empty code
  ✓ Safe code
  ✓ Unsafe code
  ✓ Complex code
  ✓ Code with I/O
  ✓ Code with tests
  ✓ Code with allocations
  ✓ Code with many dependencies
  ✓ Consistency (same code → same result)
  ✓ All dimensions in [0.0, 1.0]
Gate Criteria
text

✅ analyze_code() returns valid FitnessVector
✅ security: unsafe → penalty
✅ coverage: #[test] counted
✅ complexity: branches increase complexity
✅ reversibility: side effects lower score
✅ performance: allocations lower score
✅ architectural: use statements lower score
✅ FitnessVector::validate() passes
✅ estimate_fitness_from_reality() deleted
✅ Pipeline uses analyze_code()
✅ Week 18 gate still passes
Consequences
✅ Gains
Truth ≠ Fitness enforced:

Compile-time separation
RealityVector ≠ FitnessVector
No conversion possible
Real measurement:

FitnessVector from code properties
Not estimated from execution result
Deterministic (same code → same fitness)
Testable:

50 tests
All edge cases covered
Consistent results
Foundation for Phase 3:

Code generation يُمكن قياسه قبل التنفيذ
Multi-iteration pipeline يُمكنه الفلترة مبكراً
Memory system لديها قرارات حقيقية لتخزينها
⚠️ Limitations
Pattern matching:

False positives possible
يُحسّن بـ syn crate لاحقاً
Test coverage:

Approximation only
لا code coverage حقيقي
Performance:

String matching: O(n * m)
كافٍ لـ Phase 3 (< 1ms per analysis)
🔴 Future Work
Week 20: AST parsing (syn crate)
Week 22: Semantic analysis
Phase 4: LLVM IR analysis for performance
Phase 5: Formal verification integration
Alternatives Considered
1. syn crate (AST parsing)
Pro: Accurate, no false positives
Con: Complex, 2-3 weeks work
Decision: Defer to Week 20 — pattern matching sufficient for Week 19

2. clippy integration
Pro: Industry-standard lints
Con: External tool, hard to integrate
Decision: Use clippy insights but not the tool itself

3. Keep estimate_fitness_from_reality()
Pro: Already works
Con: Violates Truth ≠ Fitness
Decision: ❌ Rejected — architectural principle more important

References
ADR-013: Integration Pipeline (Week 17-18)
Truth ≠ Fitness: Core invariant (ADR-001)
McCabe Cyclomatic Complexity: Wikipedia
Week 19 Gate: ✅ PASSED (280 tests, 0 failures)
