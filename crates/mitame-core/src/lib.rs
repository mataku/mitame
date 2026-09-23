mod compare;
mod config;
mod diff;
mod error;
mod layout;
mod report;
mod sidecar;
mod update;

pub use compare::{compare, CompareOutcome};
pub use config::{Config, Policy, Rule, Severity, CONFIG_TEMPLATE};
pub use diff::{color_delta, diff_images, DiffOptions, DiffResult};
pub use error::Error;
pub use layout::Layout;
pub use report::{render as render_html, write_html};
pub use update::{update_baseline, UpdateOutcome};
