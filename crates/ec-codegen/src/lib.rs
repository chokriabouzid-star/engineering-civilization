#![forbid(unsafe_code)]

//! ec-codegen — Template-Based Code Generation
//!
//! Week 21 — Phase 3
//!
//! **Design:**
//! - Templates بسيطة تُنتج كود Rust صالح للتنفيذ
//! - لا ذكاء مصطنع — pattern-based فقط
//! - الكود المُولَّد يمر عبر ec-analysis و ec-constitutional
//! - كل محاولة تُسجل في ec-memory

pub mod generator;
pub mod result;
pub mod rust;
pub mod spec;
pub mod template;

pub use generator::CodeGenerator;
pub use result::{GenerationResult, GenerationSuccess};
pub use spec::GenerationSpec;
