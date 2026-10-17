//! hg-report: the 11-section Markdown report (v4 §11), with epistemic
//! tagging ([Measured]/[Estimated]). Depends on hg-schema only.

pub mod writer;

pub use writer::{render_report, write_report, MEASURED, ESTIMATED};
