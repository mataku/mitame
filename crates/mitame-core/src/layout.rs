use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use mitame_contract::Identity;
use walkdir::WalkDir;

use crate::error::{Error, Result};

#[derive(Debug, Clone)]
pub struct Layout {
    pub root: PathBuf,
    pub profile: String,
}

impl Layout {
    pub fn new(root: impl Into<PathBuf>, profile: impl Into<String>) -> Self {
        Layout {
            root: root.into(),
            profile: profile.into(),
        }
    }

    pub fn baseline_dir(&self) -> PathBuf {
        self.root.join("baseline").join(&self.profile)
    }

    pub fn current_dir(&self) -> PathBuf {
        self.root.join("current").join(&self.profile)
    }

    pub fn report_dir(&self) -> PathBuf {
        self.root.join("report")
    }

    pub fn diff_dir(&self) -> PathBuf {
        self.report_dir().join("diff")
    }

    pub fn result_path(&self) -> PathBuf {
        self.report_dir().join("result.json")
    }

    pub fn baseline_png(&self, id: &Identity) -> PathBuf {
        self.baseline_dir().join(id.png_path())
    }

    pub fn baseline_sidecar(&self, id: &Identity) -> PathBuf {
        self.baseline_dir().join(id.sidecar_path())
    }

    pub fn current_png(&self, id: &Identity) -> PathBuf {
        self.current_dir().join(id.png_path())
    }

    pub fn current_sidecar(&self, id: &Identity) -> PathBuf {
        self.current_dir().join(id.sidecar_path())
    }

    pub fn diff_png(&self, id: &Identity) -> PathBuf {
        self.diff_dir().join(id.png_path())
    }

    pub fn relative(&self, path: &Path) -> String {
        path.strip_prefix(&self.root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/")
    }

    pub fn identities(&self) -> Result<BTreeSet<Identity>> {
        let mut set = BTreeSet::new();
        for dir in [self.baseline_dir(), self.current_dir()] {
            for id in collect_identities(&dir)? {
                set.insert(id);
            }
        }
        Ok(set)
    }
}

pub fn collect_identities(dir: &Path) -> Result<Vec<Identity>> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in WalkDir::new(dir).min_depth(1) {
        let entry = entry.map_err(|e| Error::io(dir, e.into()))?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("png") {
            continue;
        }
        let rel = path
            .strip_prefix(dir)
            .map_err(|_| Error::io(path, std::io::Error::other("path outside layout")))?;
        let rel = rel.to_string_lossy().replace('\\', "/");
        out.push(Identity::from_relative_png(&rel)?);
    }
    Ok(out)
}
