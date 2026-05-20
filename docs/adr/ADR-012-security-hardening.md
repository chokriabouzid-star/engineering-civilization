# ADR-012: Security Hardening — Week 16

## Status
✅ Accepted — 2024

## Context
Week 14 أنجز Docker execution لكن مع ثغرات أمنية:
- Container يعمل كـ root (UID 0)
- Filesystem قابل للكتابة بالكامل
- لا seccomp profile
- لا `no-new-privileges`

بدون hardening، container يمكن أن:
- يكتب على `/proc/sysrq-trigger` → إعادة تشغيل الـ host
- يقرأ `/dev/mem` → الوصول لذاكرة الـ host
- يستخدم `mount` syscall → الهروب من الـ container
- يطلق fork bomb → استنزاف موارد الـ host

## Decision

### طبقات الأمان المُضافة (5 layers):

#### 1. **seccomp profile** (`profiles/rust-sandbox.json`)
- Allowlist لـ ~120 syscall ضرورية لـ Rust runtime
- `defaultAction: SCMP_ACT_ERRNO` — كل syscall آخر يُرفض
- يمنع: `reboot`, `swapon`, `kexec_load`, `bpf`, etc.

#### 2. **Non-root user**
```rust
--user 1000:1000
لا امتيازات root داخل الـ container
لا يستطيع الوصول لـ /proc/sysrq-trigger حتى لو كان موجوداً
3. Read-only root filesystem
Rust

--read-only
/etc, /usr, /bin غير قابلة للكتابة
يمنع persistence attacks
4. tmpfs workspace (200MB)
Rust

--tmpfs /workspace:size=200m,exec
--tmpfs /tmp:size=50m
المكان الوحيد القابل للكتابة
يُحذف بعد انتهاء الـ execution
لا bind mounts من الـ host
5. no-new-privileges
Rust

--security-opt no-new-privileges
يمنع setuid/setgid escalation
Escape Vectors المُختبرة
#	Vector	Attack	Mitigation	Status
1	/proc/sysrq-trigger	Host reboot	--cap-drop ALL + non-root	✅ BLOCKED
2	/dev/mem	Read host memory	--cap-drop ALL	✅ BLOCKED
3	ptrace /proc/1/mem	Trace host processes	--cap-drop ALL (no CAP_SYS_PTRACE)	✅ BLOCKED
4	mount syscall	Mount host filesystem	--cap-drop ALL (no CAP_SYS_ADMIN)	✅ BLOCKED
5	Fork bomb	Resource exhaustion	--memory 512m + timeout	✅ CONTAINED
Implementation
Rust

pub struct HardenedDockerRunner {
    base: DockerRunner,
    hardened: HardenedConfig,
}

impl HardenedDockerRunner {
    pub fn compile_and_run_hardened(&self, source_code: &str) -> Result<DockerOutput>
    pub fn test_proc_escape(&self) -> EscapeTestResult
    pub fn test_dev_mem_escape(&self) -> EscapeTestResult
    pub fn test_ptrace_escape(&self) -> EscapeTestResult
    pub fn test_mount_escape(&self) -> EscapeTestResult
    pub fn test_fork_bomb_escape(&self) -> EscapeTestResult
}
Testing Strategy
text

Unit tests (8):
  - Configuration validation
  - Basic execution (hello world)
  - Each escape vector individually

Gate tests (13):
  - All 5 escape vectors blocked
  - Non-root execution verified
  - Read-only filesystem enforced
  - tmpfs writable
  - Network isolation maintained
  - Final integration test

Stress test (ignored by default):
  - 100 executions — 0 escapes
  - Run with: cargo test --ignored
Consequences
✅ Gains
Defense in depth: 5 independent security layers
Zero escapes: في جميع الاختبارات (100+ executions)
Minimal attack surface: 120 syscalls بدلاً من 300+
Testable: كل vector له test محدد
Independent: HardenedDockerRunner منفصل عن SandboxExecutor
⚠️ Limitations
Docker dependency: لا يزال يعتمد على Docker daemon security
Image size: 788MB (rust:1.75-slim) — سيُعالج في Phase 3
seccomp maintenance: يحتاج update عند إضافة syscalls جديدة
Performance: Docker overhead لا يزال ~100ms per execution
🔴 Known Risks (to be addressed)
Docker daemon vulnerability = host compromise
Kernel vulnerability in allowed syscalls
Time-of-check-to-time-of-use races في --read-only enforcement
(كلها تُعالج في Phase 3 — VM isolation)
Invariants المُضافة
Rust

// من tests/week16_gate.rs
assert!(cfg.user_id != 0, "Must not run as root");
assert!(cfg.read_only_root, "Root filesystem must be read-only");
assert_eq!(escapes_detected, 0, "Zero escapes required");
Alternatives Considered
1. gVisor (rejected for Week 16)
Pro: User-space kernel, stronger isolation
Con: 30% performance overhead, complex setup
Decision: Re-evaluate in Phase 3
2. Firecracker microVMs (deferred to Phase 3)
Pro: Hardware-level isolation, 125ms cold start
Con: Requires KVM, infrastructure complexity
Decision: Phase 3 milestone
3. WASM sandbox (explored in Phase 4)
Pro: No syscalls, deterministic execution
Con: Limited ecosystem, no filesystem access
Decision: Phase 4 exploration
References
Docker Security Best Practices
seccomp in Linux
Container Escape Techniques
ADR-010: Docker Execution Strategy (Week 14)
Week 16 Gate: ✅ PASSED (13/13 tests)
Total Tests: 202 ✅ — 0 failures
Escapes Detected: 0 out of 100+ executions
