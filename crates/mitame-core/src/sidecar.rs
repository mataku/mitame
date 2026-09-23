use std::fs;
use std::path::Path;

use mitame_contract::{Identity, SCHEMA_VERSION};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub(crate) struct SidecarView {
    pub id: String,
    pub image: ImageView,
    #[serde(default)]
    pub captured_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ImageView {
    pub scale: f64,
}

pub(crate) fn read(side: &str, path: &Path, id: &Identity) -> Result<Option<SidecarView>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(path).map_err(|e| {
        format!(
            "{side} sidecar could not be read at {}: {e}",
            path.display()
        )
    })?;
    let value: Value = serde_json::from_str(&text).map_err(|e| {
        format!(
            "{side} sidecar is not valid JSON at {}: {e}",
            path.display()
        )
    })?;
    let version = value
        .get("schema_version")
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            format!(
                "{side} sidecar has no integer schema_version at {}",
                path.display()
            )
        })?;
    if version > u64::from(SCHEMA_VERSION) {
        return Err(format!(
            "{side} sidecar has schema_version {version} but this mitame {} reads schema_version {SCHEMA_VERSION} and older; update the binary",
            env!("CARGO_PKG_VERSION")
        ));
    }
    let view: SidecarView = serde_json::from_value(value).map_err(|e| {
        format!(
            "{side} sidecar is missing a field mitame reads at {}: {e}",
            path.display()
        )
    })?;
    if view.id != id.id() {
        return Err(format!(
            "{side} sidecar id `{}` does not match path identity `{}`",
            view.id,
            id.id()
        ));
    }
    Ok(Some(view))
}
