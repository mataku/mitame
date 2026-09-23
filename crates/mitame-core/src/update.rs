use std::fs;
use std::path::Path;

use mitame_contract::{Identity, ResultFile, Status};
use serde_json::Value;

use crate::error::{Error, Result};
use crate::layout::Layout;

#[derive(Debug, Default)]
pub struct UpdateOutcome {
    pub updated: Vec<String>,
    pub added: Vec<String>,
    pub deleted: Vec<String>,
}

pub fn update_baseline(layout: &Layout, result: &ResultFile, prune: bool) -> Result<UpdateOutcome> {
    let mut outcome = UpdateOutcome::default();
    for entry in &result.results {
        let id = Identity::parse(&entry.id)?;
        match entry.status {
            Status::Changed | Status::Mismatch | Status::Added => {
                copy(&layout.current_png(&id), &layout.baseline_png(&id))?;
                let src_sidecar = layout.current_sidecar(&id);
                let dst_sidecar = layout.baseline_sidecar(&id);
                if src_sidecar.exists() {
                    copy_sidecar(&src_sidecar, &dst_sidecar)?;
                } else {
                    remove_if_exists(&dst_sidecar)?;
                }
                if entry.status != Status::Added {
                    outcome.updated.push(entry.id.clone());
                } else {
                    outcome.added.push(entry.id.clone());
                }
            }
            Status::Removed if prune => {
                remove_if_exists(&layout.baseline_png(&id))?;
                remove_if_exists(&layout.baseline_sidecar(&id))?;
                outcome.deleted.push(entry.id.clone());
            }
            _ => {}
        }
    }
    Ok(outcome)
}

fn copy(src: &Path, dst: &Path) -> Result<()> {
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent).map_err(|e| Error::io(parent, e))?;
    }
    fs::copy(src, dst)
        .map(|_| ())
        .map_err(|e| Error::io(dst, e))
}

fn copy_sidecar(src: &Path, dst: &Path) -> Result<()> {
    let text = fs::read_to_string(src).map_err(|e| Error::io(src, e))?;
    let mut value: Value = serde_json::from_str(&text).map_err(|e| Error::Json {
        path: src.to_path_buf(),
        source: e,
    })?;
    if let Some(object) = value.as_object_mut() {
        object.shift_remove("captured_at");
        if let Some(ext) = object.get_mut("ext").and_then(Value::as_object_mut) {
            for platform in ext.values_mut().filter_map(Value::as_object_mut) {
                platform.shift_remove("run_id");
            }
        }
    }
    let mut json = serde_json::to_string_pretty(&value).map_err(|e| Error::Json {
        path: dst.to_path_buf(),
        source: e,
    })?;
    json.push('\n');
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent).map_err(|e| Error::io(parent, e))?;
    }
    fs::write(dst, json).map_err(|e| Error::io(dst, e))
}

fn remove_if_exists(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_file(path).map_err(|e| Error::io(path, e))?;
    }
    Ok(())
}
