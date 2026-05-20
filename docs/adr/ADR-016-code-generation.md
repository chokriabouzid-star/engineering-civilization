# ADR-016: Code Generation

## Status
Accepted

## Context
Phase 3 requires automated code generation from high-level specifications. The system must produce valid Rust code that passes constitutional evaluation.

## Decision
- Template-based code generation via `ec-codegen` crate
- Three templates: RustPureTemplate, RustFunctionTemplate, RustStructTemplate
- Priority-based matching: pure → function → struct
- GenerationSpec as input, GenerationResult as output
- No LLM dependency — templates are deterministic

## Consequences
- Predictable, reproducible output
- Limited to template-covered patterns
- Extensible: new templates can be added without changing the generator
