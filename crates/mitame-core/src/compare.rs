use std::fs;
use std::path::Path;

use image::RgbaImage;
use mitame_contract::{
    DiffBounds, Entry, Identity, ResultFile, Sidecar, Status, Summary, SCHEMA_VERSION,
};
use rayon::prelude::*;

use crate::config::{Config, EffectiveCompare, Severity};
use crate::diff::{diff_images, DiffOptions};
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
    let report_dir = layout.report_dir();
    if report_dir.exists() {
        fs::remove_dir_all(&report_dir).map_err(|e| Error::io(&report_dir, e))?;
    }
    fs::create_dir_all(&report_dir).map_err(|e| Error::io(&report_dir, e))?;

    let entries: Vec<Entry> = identities
        .par_iter()
        .map(|id| {
            let effective = config.effective(&matchers, &id.id());
            compare_one(layout, id, effective)
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
        mitame_version: Some(env!("CARGO_PKG_VERSION").to_string()),
        profile: layout.profile.clone(),
        summary,
        results: entries,
    };
    let json = serde_json::to_string_pretty(&result).map_err(|e| Error::Json {
        path: layout.result_path(),
        source: e,
    })?;
    fs::write(layout.result_path(), json).map_err(|e| Error::io(layout.result_path(), e))?;
    crate::report::write_html(layout, &result)?;
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

fn compare_one(layout: &Layout, id: &Identity, effective: EffectiveCompare) -> Entry {
    let baseline = layout.baseline_png(id);
    let current = layout.current_png(id);
    let mut entry = Entry {
        id: id.id(),
        status: Status::Error,
        diff_ratio: None,
        diff_pixels: None,
        diff_bounds: None,
        baseline: baseline.exists().then(|| layout.relative(&baseline)),
        current: current.exists().then(|| layout.relative(&current)),
        diff: None,
        message: None,
        captured_at: None,
    };
    if current.exists() {
        if let Ok(Some(sc)) = read_sidecar(&layout.current_sidecar(id)) {
            if let Some(msg) = sidecar_problem("current", &sc, id) {
                entry.message = Some(msg);
                return entry;
            }
            entry.captured_at = sc.captured_at;
        }
    }
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
    match compare_pair(layout, id, &baseline, &current, effective) {
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
    effective: EffectiveCompare,
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
            if let Some(msg) = sidecar_problem(side, sc, id) {
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

    let options = DiffOptions {
        pixel_tolerance: effective.pixel_tolerance,
        ignore_anti_aliasing: effective.anti_aliasing,
    };
    let result = diff_images(&baseline_img, &current_img, options);
    let ratio = result.ratio();
    let diff_path = layout.diff_png(id);
    let relative_diff = layout.relative(&diff_path);
    let changed = effective.is_changed(result.diff_pixels, result.total_pixels);
    let has_diff = result.diff_pixels > 0;
    if has_diff {
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
        e.diff_bounds = result.bounds.map(|(x0, y0, x1, y1)| DiffBounds {
            x: x0,
            y: y0,
            width: x1 - x0 + 1,
            height: y1 - y0 + 1,
        });
        if has_diff {
            e.diff = Some(relative_diff);
        }
        e
    }))
}

fn sidecar_problem(side: &str, sc: &Sidecar, id: &Identity) -> Option<String> {
    if sc.schema_version != SCHEMA_VERSION {
        return Some(format!(
            "{side} sidecar has schema_version {} but this mitame {} reads schema_version {SCHEMA_VERSION}; update the binary or the adapter",
            sc.schema_version,
            env!("CARGO_PKG_VERSION")
        ));
    }
    if sc.id != id.id() {
        return Some(format!(
            "{side} sidecar id `{}` does not match path identity `{}`",
            sc.id,
            id.id()
        ));
    }
    None
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
