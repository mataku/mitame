use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Sidecar {
    pub schema_version: u32,
    pub id: String,
    pub platform: String,
    pub capture: String,
    pub group: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default)]
    pub variant: BTreeMap<String, String>,
    pub image: ImageInfo,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<EnvInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub captured_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ext: Option<BTreeMap<String, Value>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ImageInfo {
    pub width: u32,
    pub height: u32,
    pub scale: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct EnvInfo {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub os: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sdk: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub renderer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ci: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SourceInfo {
    pub file: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
}
