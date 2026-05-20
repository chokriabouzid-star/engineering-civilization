# ADR-010: Docker Execution Strategy

**Status:** Accepted
**Date:** 2025-01-15
**Context:** Week 14 — Real sandbox execution

## Decision

نستخدم Docker containers لتنفيذ الأكواد بأمان:

### Architecture
```
SandboxExecutor
  └─> RustSandboxCompiler
        └─> DockerRunner
              └─> docker run --network none --cap-drop ALL --tmpfs /workspace
```

### Key Decisions

1. **Tmpfs workspace** (لا bind mounts)
   - الكود يُحقن عبر shell: `printf '%s' "$CODE" > /workspace/main.rs`
   - أسرع + أبسط + لا shared filesystem

2. **Image:** `rust:1.75-slim` (788MB)
   - يمكن تصغيرها لـ `rust:alpine` لاحقاً

3. **Security constraints:**
   - `--network none` — لا وصول للشبكة
   - `--cap-drop ALL` — لا capabilities
   - `--memory 512m` — حد الذاكرة
   - `--cpus 0.5` — حد CPU

4. **Escape vectors tested:**
   - `/proc/sysrq-trigger` ✅ blocked
   - `mount` syscall ✅ blocked
   - `ptrace` ✅ blocked
   - `/dev/mem` ✅ blocked
   - fork bomb ✅ contained

### Performance
- Single execution: 70-150ms
- 3-run reproducibility: ~250ms
- 20 executions: ~125s

## Consequences

✅ Real execution (لا simulation)
✅ Reproducibility من hash comparison
✅ Latency measurement حقيقي
⚠️ Container runs as root (seccomp في Week 16)
⚠️ No `--read-only` filesystem (Week 16)

## Implementation

- `docker.rs` — CLI wrapper
- `compiler.rs` — Rust compilation pipeline
- `metrics.rs` — reproducibility + latency extraction
