use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_map")
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("skills directory")
        .parent()
        .expect("repository root")
        .to_path_buf()
}

fn scratch(name: &str) -> PathBuf {
    let path = repo_root().join("test").join("map-legacy-marker").join(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create scratch root");
    path
}

fn schema() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("schema.surql")
}

fn run(root: &Path, args: &[&str]) {
    let output = Command::new(bin())
        .arg("--path")
        .arg(root)
        .args(args)
        .output()
        .expect("run map");
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

fn initialized(name: &str) -> PathBuf {
    let root = scratch(name);
    let schema = schema().to_string_lossy().into_owned();
    run(&root, &["init", "--schema", &schema]);
    root
}

#[test]
fn valid_legacy_generated_data_marker_is_removed() {
    let root = initialized("valid");
    let marker = root.join(".map").join(".jls-owned.json");
    fs::write(
        &marker,
        r#"{"format":1,"owner":"jls","kind":"generated-data","skill":"map"}"#,
    )
    .unwrap();

    run(&root, &["status"]);
    assert!(!marker.exists());
}

#[test]
fn coincidentally_named_non_marker_file_is_preserved() {
    let root = initialized("foreign");
    let marker = root.join(".map").join(".jls-owned.json");
    let foreign = r#"{"format":1,"owner":"someone-else","kind":"generated-data","skill":"map"}"#;
    fs::write(&marker, foreign).unwrap();

    run(&root, &["status"]);
    assert_eq!(fs::read_to_string(marker).unwrap(), foreign);
}

#[test]
fn legacy_marker_with_extra_fields_is_preserved() {
    let root = initialized("extra-field");
    let marker = root.join(".map").join(".jls-owned.json");
    let foreign = r#"{"format":1,"owner":"jls","kind":"generated-data","skill":"map","note":"not the legacy schema"}"#;
    fs::write(&marker, foreign).unwrap();

    run(&root, &["status"]);
    assert_eq!(fs::read_to_string(marker).unwrap(), foreign);
}
