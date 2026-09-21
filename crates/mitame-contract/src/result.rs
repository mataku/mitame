use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Changed,
    Added,
    Removed,
    Mismatch,
    Error,
    Unchanged,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Summary {
    pub unchanged: u32,
    pub changed: u32,
    pub added: u32,
    pub removed: u32,
    pub mismatch: u32,
    pub error: u32,
}

impl Summary {
    pub fn count(&mut self, status: Status) {
        match status {
            Status::Unchanged => self.unchanged += 1,
            Status::Changed => self.changed += 1,
            Status::Added => self.added += 1,
            Status::Removed => self.removed += 1,
            Status::Mismatch => self.mismatch += 1,
            Status::Error => self.error += 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Entry {
    pub id: String,
    pub status: Status,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diff_ratio: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diff_pixels: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diff: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub captured_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResultFile {
    pub schema_version: u32,
    pub profile: String,
    pub summary: Summary,
    pub results: Vec<Entry>,
}
