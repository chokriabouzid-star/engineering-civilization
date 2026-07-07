#![forbid(unsafe_code)]

//! ec-app — Integration Pipeline
//!
//! يدمج كل الطبقات:
//! - ec-analysis (تحليل ثابت)
//! - ec-constitutional (تقييم دستوري)
//! - ec-sandbox (تنفيذ)
//! - ec-sandbox::feedback (تعلم)
//! - ec-codegen (توليد الكود)
//! - ec-memory (ذاكرة سببية)

pub mod pipeline;

pub use pipeline::{
    build_epistemic_from_reality, AttemptRecord, IntegrationPipeline, IterativePipeline,
    IterativePipelineResult, PipelineResult, PipelineVerdict,
};
