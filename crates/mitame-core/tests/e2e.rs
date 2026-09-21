use std::fs;
use std::path::Path;

use image::{Rgba, RgbaImage};
use mitame_contract::Status;
use mitame_core::{approve, compare, Config, Layout};

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

    let approved = approve(&layout, &[]).unwrap();
    assert_eq!(approved.copied.len(), 4);
    assert_eq!(approved.unchanged, vec!["flutter/a/same".to_string()]);
    assert_eq!(approved.deleted, vec!["flutter/a/gone".to_string()]);
    let outcome = compare(&config, &layout).unwrap();
    assert!(!outcome.failed);
    assert_eq!(outcome.result.summary.unchanged, 5);
    assert_eq!(outcome.result.results.len(), 5);
}

#[test]
fn approve_selected_ids_only() {
    let dir = tempfile::tempdir().unwrap();
    let layout = Layout::new(dir.path().join(".mitame"), "linux");
    let current = |id: &str| layout.root.join("current/linux").join(format!("{id}.png"));
    write_png(&current("ios/x"), 4, 4, [0, 0, 0, 255], &[]);
    write_png(&current("ios/y"), 4, 4, [0, 0, 0, 255], &[]);
    fs::write(layout.root.join("current/linux/ios/x.json"), "{}").unwrap();

    let outcome = approve(&layout, &["ios/x".to_string()]).unwrap();
    assert_eq!(outcome.copied, vec!["ios/x".to_string()]);
    assert!(layout.root.join("baseline/linux/ios/x.png").exists());
    assert!(layout.root.join("baseline/linux/ios/x.json").exists());
    assert!(!layout.root.join("baseline/linux/ios/y.png").exists());
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
fn approve_skips_identical_png_and_keeps_baseline_sidecar() {
    let dir = tempfile::tempdir().unwrap();
    let layout = Layout::new(dir.path().join(".mitame"), "default");
    let png = |side: &str| layout.root.join(side).join("default/flutter/a/x.png");
    let json = |side: &str| layout.root.join(side).join("default/flutter/a/x.json");
    write_png(&png("baseline"), 4, 4, [0, 0, 0, 255], &[]);
    write_png(&png("current"), 4, 4, [0, 0, 0, 255], &[]);
    fs::write(json("baseline"), "{\"captured_at\":\"old\"}").unwrap();
    fs::write(json("current"), "{\"captured_at\":\"new\"}").unwrap();

    let outcome = approve(&layout, &[]).unwrap();
    assert_eq!(outcome.unchanged, vec!["flutter/a/x".to_string()]);
    assert!(outcome.copied.is_empty());
    assert_eq!(
        fs::read_to_string(json("baseline")).unwrap(),
        "{\"captured_at\":\"old\"}"
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
