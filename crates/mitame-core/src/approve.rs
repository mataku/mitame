use std::fs;
use std::path::Path;

use mitame_contract::Identity;

use crate::error::{Error, Result};
use crate::layout::Layout;

#[derive(Debug, Default)]
pub struct ApproveOutcome {
    pub copied: Vec<String>,
    pub deleted: Vec<String>,
}

pub fn approve(layout: &Layout, ids: &[String]) -> Result<ApproveOutcome> {
    let selected: Vec<Identity> = if ids.is_empty() {
        layout.identities()?.into_iter().collect()
    } else {
        ids.iter()
            .map(|s| Identity::parse(s))
            .collect::<std::result::Result<_, _>>()?
    };

    let mut outcome = ApproveOutcome::default();
    for id in &selected {
        let src_png = layout.current_png(id);
        let dst_png = layout.baseline_png(id);
        let dst_sidecar = layout.baseline_sidecar(id);
        if !src_png.exists() {
            if dst_png.exists() {
                remove(&dst_png)?;
                remove_if_exists(&dst_sidecar)?;
                outcome.deleted.push(id.id());
            }
            continue;
        }
        copy(&src_png, &dst_png)?;
        let src_sidecar = layout.current_sidecar(id);
        if src_sidecar.exists() {
            copy(&src_sidecar, &dst_sidecar)?;
        } else {
            remove_if_exists(&dst_sidecar)?;
        }
        outcome.copied.push(id.id());
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

fn remove(path: &Path) -> Result<()> {
    fs::remove_file(path).map_err(|e| Error::io(path, e))
}

fn remove_if_exists(path: &Path) -> Result<()> {
    if path.exists() {
        remove(path)?;
    }
    Ok(())
}
