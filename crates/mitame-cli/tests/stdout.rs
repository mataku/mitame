use std::process::{Command, Stdio};

#[test]
fn compare_survives_a_closed_stdout() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".mitame").join("baseline")).unwrap();
    std::fs::create_dir_all(dir.path().join(".mitame").join("current").join("default")).unwrap();
    let (reader, writer) = std::io::pipe().unwrap();
    drop(reader);
    let output = Command::new(env!("CARGO_BIN_EXE_mitame"))
        .args(["compare", "--update"])
        .current_dir(dir.path())
        .stdout(writer)
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("panicked"), "{stderr}");
    assert_eq!(output.status.code(), Some(0), "{stderr}");
    assert!(dir
        .path()
        .join(".mitame")
        .join("report")
        .join("result.json")
        .is_file());
}
