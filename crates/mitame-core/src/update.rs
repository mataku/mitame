use std::fs;
use std::path::Path;

use mitame_contract::{Identity, ResultFile, Status};

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
                    copy(&src_sidecar, &dst_sidecar)?;
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

fn remove_if_exists(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_file(path).map_err(|e| Error::io(path, e))?;
    }
    Ok(())
}
