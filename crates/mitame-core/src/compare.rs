use std::fs;
use std::path::Path;

use image::RgbaImage;
use mitame_contract::{Entry, Identity, ResultFile, Sidecar, Status, Summary, SCHEMA_VERSION};
use rayon::prelude::*;

use crate::config::{Config, Severity};
use crate::diff::diff_images;
use crate::error::{Error, Result};
use crate::layout::Layout;

#[derive(Debug)]
pub struct CompareOutcome {
    pub result: ResultFile,
    pub failed: bool,
    pub errored: bool,
}

pub fn compare(config: &Config, layout: &Layout) -> Result<CompareOutcome> {
    let matchers = config.matchers()?;
    let identities: Vec<Identity> = layout.identities()?.into_iter().collect();
    let diff_dir = layout.diff_dir();
    if diff_dir.exists() {
        fs::remove_dir_all(&diff_dir).map_err(|e| Error::io(&diff_dir, e))?;
    }
    fs::create_dir_all(layout.report_dir()).map_err(|e| Error::io(layout.report_dir(), e))?;

    let entries: Vec<Entry> = identities
        .par_iter()
        .map(|id| {
            let effective = config.effective(&matchers, &id.id());
            compare_one(layout, id, effective.threshold, effective.pixel_tolerance)
        })
        .collect();

    let mut summary = Summary::default();
    let mut failed = false;
    let mut errored = false;
    for entry in &entries {
        summary.count(entry.status);
        failed |= is_failure(config, entry.status);
        errored |= entry.status == Status::Error;
    }

    let result = ResultFile {
        schema_version: SCHEMA_VERSION,
        profile: layout.profile.clone(),
        summary,
        results: entries,
    };
    let json = serde_json::to_string_pretty(&result).map_err(|e| Error::Json {
        path: layout.result_path(),
        source: e,
    })?;
    fs::write(layout.result_path(), json).map_err(|e| Error::io(layout.result_path(), e))?;
    Ok(CompareOutcome {
        result,
        failed,
        errored,
    })
}

fn is_failure(config: &Config, status: Status) -> bool {
    match status {
        Status::Unchanged | Status::Error => false,
        Status::Changed => true,
        Status::Added => config.policy.added == Severity::Fail,
        Status::Removed => config.policy.removed == Severity::Fail,
        Status::Mismatch => config.policy.mismatch == Severity::Fail,
    }
}

fn compare_one(layout: &Layout, id: &Identity, threshold: f64, pixel_tolerance: f64) -> Entry {
    let baseline = layout.baseline_png(id);
    let current = layout.current_png(id);
    let mut entry = Entry {
        id: id.id(),
        status: Status::Error,
        diff_ratio: None,
        diff_pixels: None,
        baseline: baseline.exists().then(|| layout.relative(&baseline)),
        current: current.exists().then(|| layout.relative(&current)),
        diff: None,
        message: None,
    };
    match (baseline.exists(), current.exists()) {
        (false, false) => {
            entry.message = Some("neither baseline nor current exists".to_string());
            return entry;
        }
        (true, false) => {
            entry.status = Status::Removed;
            return entry;
        }
        (false, true) => {
            entry.status = Status::Added;
            return entry;
        }
        (true, true) => {}
    }
    match compare_pair(layout, id, &baseline, &current, threshold, pixel_tolerance) {
        Ok(filled) => filled(entry),
        Err(e) => {
            entry.status = Status::Error;
            entry.message = Some(e.to_string());
            entry
        }
    }
}

type Fill = Box<dyn FnOnce(Entry) -> Entry>;

fn compare_pair(
    layout: &Layout,
    id: &Identity,
    baseline: &Path,
    current: &Path,
    threshold: f64,
    pixel_tolerance: f64,
) -> Result<Fill> {
    let baseline_bytes = fs::read(baseline).map_err(|e| Error::io(baseline, e))?;
    let current_bytes = fs::read(current).map_err(|e| Error::io(current, e))?;
    if baseline_bytes == current_bytes {
        return Ok(Box::new(|mut e| {
            e.status = Status::Unchanged;
            e.diff_ratio = Some(0.0);
            e.diff_pixels = Some(0);
            e
        }));
    }

    let baseline_img = decode(baseline, &baseline_bytes)?;
    let current_img = decode(current, &current_bytes)?;
    if baseline_img.dimensions() != current_img.dimensions() {
        let msg = format!(
            "dimensions differ: baseline {}x{}, current {}x{}",
            baseline_img.width(),
            baseline_img.height(),
            current_img.width(),
            current_img.height()
        );
        return Ok(Box::new(move |mut e| {
            e.status = Status::Mismatch;
            e.message = Some(msg);
            e
        }));
    }
    let baseline_sidecar = read_sidecar(&layout.baseline_sidecar(id))?;
    let current_sidecar = read_sidecar(&layout.current_sidecar(id))?;
    for (side, sidecar) in [
        ("baseline", &baseline_sidecar),
        ("current", &current_sidecar),
    ] {
        if let Some(sc) = sidecar {
            if sc.id != id.id() {
                let msg = format!(
                    "{side} sidecar id `{}` does not match path identity `{}`",
                    sc.id,
                    id.id()
                );
                return Ok(Box::new(move |mut e| {
                    e.status = Status::Error;
                    e.message = Some(msg);
                    e
                }));
            }
        }
    }
    if let (Some(b), Some(c)) = (&baseline_sidecar, &current_sidecar) {
        if (b.image.scale - c.image.scale).abs() > f64::EPSILON {
            let msg = format!(
                "scale differs: baseline {}, current {}",
                b.image.scale, c.image.scale
            );
            return Ok(Box::new(move |mut e| {
                e.status = Status::Mismatch;
                e.message = Some(msg);
                e
            }));
        }
    }

    let result = diff_images(&baseline_img, &current_img, pixel_tolerance);
    let ratio = result.ratio();
    let diff_path = layout.diff_png(id);
    let relative_diff = layout.relative(&diff_path);
    let changed = ratio > threshold;
    if changed {
        if let Some(parent) = diff_path.parent() {
            fs::create_dir_all(parent).map_err(|e| Error::io(parent, e))?;
        }
        result.image.save(&diff_path).map_err(|e| Error::Image {
            path: diff_path.clone(),
            source: e,
        })?;
    }
    Ok(Box::new(move |mut e| {
        e.status = if changed {
            Status::Changed
        } else {
            Status::Unchanged
        };
        e.diff_ratio = Some(ratio);
        e.diff_pixels = Some(result.diff_pixels);
        if changed {
            e.diff = Some(relative_diff);
        }
        e
    }))
}

fn decode(path: &Path, bytes: &[u8]) -> Result<RgbaImage> {
    image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
        .map(|img| img.to_rgba8())
        .map_err(|e| Error::Image {
            path: path.to_path_buf(),
            source: e,
        })
}

fn read_sidecar(path: &Path) -> Result<Option<Sidecar>> {
    if !path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(path).map_err(|e| Error::io(path, e))?;
    serde_json::from_str(&text)
        .map(Some)
        .map_err(|e| Error::Json {
            path: path.to_path_buf(),
            source: e,
        })
}
