use std::fs;
use std::path::Path;

use image::{Rgba, RgbaImage};
use mitame_contract::{DiffBounds, Status};
use mitame_core::{compare, update_baseline, Config, Layout};

fn write_png(path: &Path, width: u32, height: u32, color: [u8; 4], marks: &[(u32, u32)]) {
    let mut img = RgbaImage::from_pixel(width, height, Rgba(color));
    for &(x, y) in marks {
        img.put_pixel(x, y, Rgba([255, 0, 0, 255]));
    }
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    img.save(path).unwrap();
}

fn status_of(layout: &Layout, config: &Config, id: &str) -> Status {
    let outcome = compare(config, layout).unwrap();
    outcome
        .result
        .results
        .iter()
        .find(|e| e.id == id)
        .unwrap_or_else(|| panic!("no entry for {id}"))
        .status
}

fn sidecar_json(id: &str, schema_version: u32) -> String {
    format!(
        r#"{{"schema_version":{schema_version},"id":"{id}","platform":"ios","capture":"widget","group":"","name":"x","variant":{{}},"image":{{"width":4,"height":4,"scale":1.0}}}}"#
    )
}

#[test]
fn compare_classifies_and_approve_promotes() {
    let dir = tempfile::tempdir().unwrap();
    let layout = Layout::new(dir.path().join(".mitame"), "default");
    let config = Config::default();

    let png = |side: &str, id: &str| {
        layout
            .root
            .join(side)
            .join("default")
            .join(format!("{id}.png"))
    };

    write_png(
        &png("baseline", "flutter/a/same"),
        20,
        20,
        [10, 20, 30, 255],
        &[],
    );
    write_png(
        &png("current", "flutter/a/same"),
        20,
        20,
        [10, 20, 30, 255],
        &[],
    );
    write_png(
        &png("baseline", "flutter/a/tiny__theme=dark"),
        100,
        100,
        [0, 0, 0, 255],
        &[],
    );
    let twenty: Vec<(u32, u32)> = (0..20).map(|i| (i, 0)).collect();
    write_png(
        &png("current", "flutter/a/tiny__theme=dark"),
        100,
        100,
        [0, 0, 0, 255],
        &twenty,
    );
    write_png(
        &png("baseline", "flutter/a/big"),
        10,
        10,
        [0, 0, 0, 255],
        &[],
    );
    write_png(
        &png("current", "flutter/a/big"),
        10,
        10,
        [255, 255, 255, 255],
        &[],
    );
    write_png(
        &png("baseline", "flutter/a/resized"),
        10,
        10,
        [0, 0, 0, 255],
        &[],
    );
    write_png(
        &png("current", "flutter/a/resized"),
        12,
        10,
        [0, 0, 0, 255],
        &[],
    );
    write_png(&png("current", "flutter/a/new"), 5, 5, [0, 0, 0, 255], &[]);
    write_png(
        &png("baseline", "flutter/a/gone"),
        5,
        5,
        [0, 0, 0, 255],
        &[],
    );

    let outcome = compare(&config, &layout).unwrap();
    assert!(outcome.failed);
    let s = &outcome.result.summary;
    assert_eq!(
        (
            s.unchanged,
            s.changed,
            s.added,
            s.removed,
            s.mismatch,
            s.error
        ),
        (1, 2, 1, 1, 1, 0)
    );
    assert_eq!(
        status_of(&layout, &config, "flutter/a/same"),
        Status::Unchanged
    );
    assert_eq!(
        status_of(&layout, &config, "flutter/a/tiny__theme=dark"),
        Status::Changed
    );
    assert_eq!(
        status_of(&layout, &config, "flutter/a/big"),
        Status::Changed
    );
    assert_eq!(
        status_of(&layout, &config, "flutter/a/resized"),
        Status::Mismatch
    );
    assert_eq!(status_of(&layout, &config, "flutter/a/new"), Status::Added);
    assert_eq!(
        status_of(&layout, &config, "flutter/a/gone"),
        Status::Removed
    );
    assert!(layout.root.join("report/diff/flutter/a/big.png").exists());
    assert!(layout.root.join("report/result.json").exists());

    let lenient: Config = toml::from_str(
        r#"
        [[rules]]
        match = "flutter/**/*__*theme=dark*"
        threshold = 0.01
        "#,
    )
    .unwrap();
    assert_eq!(
        status_of(&layout, &lenient, "flutter/a/tiny__theme=dark"),
        Status::Unchanged
    );
    assert!(layout
        .root
        .join("report/diff/flutter/a/tiny__theme=dark.png")
        .exists());
    assert!(layout
        .root
        .join("report/current/flutter/a/tiny__theme=dark.png")
        .exists());
    assert!(!layout
        .root
        .join("report/current/flutter/a/same.png")
        .exists());
    let html = fs::read_to_string(layout.root.join("report/index.html")).unwrap();
    assert!(html.contains("0.200% · 20 px"));

    let updated = update_baseline(&layout, &outcome.result, false).unwrap();
    assert_eq!(updated.updated.len() + updated.added.len(), 4);
    assert!(updated.deleted.is_empty());
    assert!(layout
        .root
        .join("baseline/default/flutter/a/gone.png")
        .exists());
    let outcome = compare(&config, &layout).unwrap();
    assert!(!outcome.failed);
    assert_eq!(outcome.result.summary.unchanged, 5);
    assert_eq!(outcome.result.summary.removed, 1);
    let pruned = update_baseline(&layout, &outcome.result, true).unwrap();
    assert_eq!(pruned.deleted, vec!["flutter/a/gone".to_string()]);
    assert!(!layout
        .root
        .join("baseline/default/flutter/a/gone.png")
        .exists());
    let outcome = compare(&config, &layout).unwrap();
    assert_eq!(outcome.result.results.len(), 5);
}

#[test]
fn update_copies_added_entries_with_their_sidecars() {
    let dir = tempfile::tempdir().unwrap();
    let layout = Layout::new(dir.path().join(".mitame"), "linux");
    let config = Config::default();
    let current = |id: &str| layout.root.join("current/linux").join(format!("{id}.png"));
    write_png(&current("ios/x"), 4, 4, [0, 0, 0, 255], &[]);
    write_png(&current("ios/y"), 4, 4, [0, 0, 0, 255], &[]);
    fs::write(
        layout.root.join("current/linux/ios/x.json"),
        sidecar_json("ios/x", 1),
    )
    .unwrap();

    let outcome = compare(&config, &layout).unwrap();
    let updated = update_baseline(&layout, &outcome.result, false).unwrap();
    assert_eq!(
        updated.added,
        vec!["ios/x".to_string(), "ios/y".to_string()]
    );
    assert!(layout.root.join("baseline/linux/ios/x.png").exists());
    assert!(layout.root.join("baseline/linux/ios/x.json").exists());
    assert!(layout.root.join("baseline/linux/ios/y.png").exists());
    assert!(!layout.root.join("baseline/linux/ios/y.json").exists());
}

#[test]
fn corrupt_png_is_error_not_failure() {
    let dir = tempfile::tempdir().unwrap();
    let layout = Layout::new(dir.path().join(".mitame"), "default");
    let config = Config::default();
    write_png(
        &layout.root.join("baseline/default/flutter/a/x.png"),
        4,
        4,
        [0, 0, 0, 255],
        &[],
    );
    let bad = layout.root.join("current/default/flutter/a/x.png");
    fs::create_dir_all(bad.parent().unwrap()).unwrap();
    fs::write(&bad, b"not a png").unwrap();

    let outcome = compare(&config, &layout).unwrap();
    assert!(outcome.errored);
    assert!(!outcome.failed);
    assert_eq!(outcome.result.summary.error, 1);
    assert_eq!(outcome.result.results[0].status, Status::Error);
}

#[test]
fn sidecar_id_mismatch_is_error() {
    let dir = tempfile::tempdir().unwrap();
    let layout = Layout::new(dir.path().join(".mitame"), "default");
    let config = Config::default();
    write_png(
        &layout.root.join("baseline/default/flutter/a/x.png"),
        4,
        4,
        [0, 0, 0, 255],
        &[],
    );
    write_png(
        &layout.root.join("current/default/flutter/a/x.png"),
        4,
        4,
        [1, 0, 0, 255],
        &[],
    );
    let sidecar = r#"{"schema_version":1,"id":"flutter/a/other","platform":"flutter","capture":"widget","group":"a","name":"other","variant":{},"image":{"width":4,"height":4,"scale":1.0}}"#;
    fs::write(
        layout.root.join("current/default/flutter/a/x.json"),
        sidecar,
    )
    .unwrap();

    let outcome = compare(&config, &layout).unwrap();
    assert!(outcome.errored);
    let entry = &outcome.result.results[0];
    assert_eq!(entry.status, Status::Error);
    assert!(entry.message.as_deref().unwrap().contains("does not match"));
}

#[test]
fn sidecar_schema_version_mismatch_is_error() {
    let dir = tempfile::tempdir().unwrap();
    let layout = Layout::new(dir.path().join(".mitame"), "default");
    let config = Config::default();
    write_png(
        &layout.root.join("current/default/flutter/a/x.png"),
        4,
        4,
        [0, 0, 0, 255],
        &[],
    );
    let sidecar = r#"{"schema_version":99,"id":"flutter/a/x","platform":"flutter","capture":"widget","group":"a","name":"x","variant":{},"image":{"width":4,"height":4,"scale":1.0}}"#;
    fs::write(
        layout.root.join("current/default/flutter/a/x.json"),
        sidecar,
    )
    .unwrap();

    let outcome = compare(&config, &layout).unwrap();
    assert!(outcome.errored);
    let entry = &outcome.result.results[0];
    assert_eq!(entry.status, Status::Error);
    assert!(entry
        .message
        .as_deref()
        .unwrap()
        .contains("schema_version 99"));
    assert_eq!(
        outcome.result.mitame_version.as_deref(),
        Some(env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn update_leaves_unchanged_entries_alone_even_when_bytes_differ() {
    let dir = tempfile::tempdir().unwrap();
    let layout = Layout::new(dir.path().join(".mitame"), "default");
    let mut config = Config::default();
    config.compare.max_diff_pixels = 5;
    let png = |side: &str| layout.root.join(side).join("default/flutter/a/x.png");
    let json = |side: &str| layout.root.join(side).join("default/flutter/a/x.json");
    write_png(&png("baseline"), 4, 4, [0, 0, 0, 255], &[]);
    write_png(&png("current"), 4, 4, [0, 0, 0, 255], &[(0, 0)]);
    let sidecar = |stamp: &str| {
        format!(
            r#"{{"schema_version":1,"id":"flutter/a/x","platform":"flutter","capture":"widget","group":"a","name":"x","variant":{{}},"image":{{"width":4,"height":4,"scale":1.0}},"captured_at":"{stamp}"}}"#
        )
    };
    fs::write(json("baseline"), sidecar("old")).unwrap();
    fs::write(json("current"), sidecar("new")).unwrap();

    let before = fs::read(png("baseline")).unwrap();
    let outcome = compare(&config, &layout).unwrap();
    assert_eq!(outcome.result.results[0].status, Status::Unchanged);
    let updated = update_baseline(&layout, &outcome.result, false).unwrap();
    assert!(updated.updated.is_empty());
    assert_eq!(fs::read(png("baseline")).unwrap(), before);
    assert_eq!(
        fs::read_to_string(json("baseline")).unwrap(),
        sidecar("old")
    );
}

#[test]
fn compare_writes_html_report() {
    let dir = tempfile::tempdir().unwrap();
    let layout = Layout::new(dir.path().join(".mitame"), "default");
    write_png(
        &layout.root.join("current/default/flutter/a/x.png"),
        4,
        4,
        [0, 0, 0, 255],
        &[],
    );
    compare(&Config::default(), &layout).unwrap();
    let html = fs::read_to_string(layout.root.join("report/index.html")).unwrap();
    assert!(html.contains("flutter/a/x"));
    assert!(html.contains("src=\"current/flutter/a/x.png\""));
    assert!(layout.root.join("report/current/flutter/a/x.png").exists());
    assert!(!layout.root.join("report/baseline/flutter/a/x.png").exists());
}

#[test]
fn diff_bounds_cover_the_differing_pixels_and_land_in_result_json() {
    let dir = tempfile::tempdir().unwrap();
    let layout = Layout::new(dir.path().join(".mitame"), "default");
    let config = Config::default();
    let png = |side: &str, id: &str| {
        layout
            .root
            .join(side)
            .join("default")
            .join(format!("{id}.png"))
    };
    write_png(&png("baseline", "ios/a/same"), 20, 20, [0, 0, 0, 255], &[]);
    write_png(&png("current", "ios/a/same"), 20, 20, [0, 0, 0, 255], &[]);
    write_png(&png("baseline", "ios/a/moved"), 40, 30, [0, 0, 0, 255], &[]);
    write_png(
        &png("current", "ios/a/moved"),
        40,
        30,
        [0, 0, 0, 255],
        &[(10, 12), (25, 12), (10, 21)],
    );
    let outcome = compare(&config, &layout).unwrap();
    let entry = |id: &str| outcome.result.results.iter().find(|e| e.id == id).unwrap();
    assert_eq!(
        entry("ios/a/moved").diff_bounds,
        Some(DiffBounds {
            x: 10,
            y: 12,
            width: 16,
            height: 10
        })
    );
    assert_eq!(entry("ios/a/same").diff_bounds, None);
    let json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(layout.root.join("report/result.json")).unwrap())
            .unwrap();
    let results = json["results"].as_array().unwrap();
    let moved = results.iter().find(|e| e["id"] == "ios/a/moved").unwrap();
    assert_eq!(
        moved["diff_bounds"],
        serde_json::json!({"x": 10, "y": 12, "width": 16, "height": 10})
    );
    let same = results.iter().find(|e| e["id"] == "ios/a/same").unwrap();
    assert!(same.get("diff_bounds").is_none());
}

#[test]
fn unreadable_current_sidecar_is_error_and_not_copied_on_update() {
    let dir = tempfile::tempdir().unwrap();
    let layout = Layout::new(dir.path().join(".mitame"), "default");
    let config = Config::default();
    write_png(
        &layout.root.join("current/default/ios/x.png"),
        4,
        4,
        [0, 0, 0, 255],
        &[],
    );
    fs::write(layout.root.join("current/default/ios/x.json"), "{}").unwrap();

    let outcome = compare(&config, &layout).unwrap();
    assert!(outcome.errored);
    let entry = &outcome.result.results[0];
    assert_eq!(entry.status, Status::Error);
    assert!(entry
        .message
        .as_deref()
        .unwrap()
        .contains("current sidecar is invalid"));
    let updated = update_baseline(&layout, &outcome.result, false).unwrap();
    assert!(updated.added.is_empty());
    assert!(!layout.root.join("baseline/default/ios/x.png").exists());
}

#[test]
fn baseline_sidecar_problem_is_error_before_dimension_and_byte_checks() {
    let dir = tempfile::tempdir().unwrap();
    let layout = Layout::new(dir.path().join(".mitame"), "default");
    let config = Config::default();
    let png = |side: &str, id: &str| {
        layout
            .root
            .join(side)
            .join("default")
            .join(format!("{id}.png"))
    };
    let json = |id: &str| {
        layout
            .root
            .join("baseline/default")
            .join(format!("{id}.json"))
    };
    write_png(&png("baseline", "ios/resized"), 4, 4, [0, 0, 0, 255], &[]);
    write_png(&png("current", "ios/resized"), 6, 4, [0, 0, 0, 255], &[]);
    fs::write(json("ios/resized"), sidecar_json("ios/resized", 99)).unwrap();
    write_png(&png("baseline", "ios/same"), 4, 4, [0, 0, 0, 255], &[]);
    write_png(&png("current", "ios/same"), 4, 4, [0, 0, 0, 255], &[]);
    fs::write(json("ios/same"), sidecar_json("ios/other", 1)).unwrap();

    let outcome = compare(&config, &layout).unwrap();
    let entry = |id: &str| outcome.result.results.iter().find(|e| e.id == id).unwrap();
    let resized = entry("ios/resized");
    assert_eq!(resized.status, Status::Error);
    assert!(resized
        .message
        .as_deref()
        .unwrap()
        .contains("baseline sidecar has schema_version 99"));
    let same = entry("ios/same");
    assert_eq!(same.status, Status::Error);
    assert!(same.message.as_deref().unwrap().contains("does not match"));
}
