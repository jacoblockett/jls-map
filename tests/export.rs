use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_map")
}

fn schema() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("schema.surql")
        .to_string_lossy()
        .into_owned()
}

fn scratch(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("jls-map-export-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create export test root");
    root
}

fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(bin())
        .env("USERPROFILE", root)
        .env("HOME", root)
        .arg("--path")
        .arg(root)
        .args(args)
        .output()
        .expect("run map")
}

fn ok_json(root: &Path, args: &[&str]) -> Value {
    let output = run(root, args);
    assert!(
        output.status.success(),
        "map {:?} failed\nstdout:\n{}\nstderr:\n{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("JSON stdout")
}

fn id(value: &Value) -> String {
    value["id"].as_str().expect("id").to_string()
}

fn new_map(name: &str) -> PathBuf {
    let root = scratch(name);
    let schema = schema();
    ok_json(&root, &["init", "--schema", &schema]);
    root
}

#[test]
fn export_defaults_to_current_nonabandoned_json() {
    let root = new_map("default");
    let intent = id(&ok_json(&root, &["create", "intent", "Build app"]));
    let question = id(&ok_json(
        &root,
        &["create", "question", "Which auth model?", "--intent", &intent],
    ));
    let old = id(&ok_json(
        &root,
        &["create", "decision", "Passwords", "--question", &question],
    ));
    let current = id(&ok_json(&root, &["create", "decision", "Passkeys"]));
    ok_json(
        &root,
        &["replace", &old, &current, "--reason", "Updated requirement"],
    );
    let abandoned = id(&ok_json(&root, &["create", "idea", "Legacy mode"]));
    ok_json(
        &root,
        &["abandon", &abandoned, "--by", "user", "--reason", "Dropped"],
    );
    ok_json(&root, &["session", "init"]);

    let export = ok_json(&root, &["export"]);
    assert_eq!(export["validation"]["ok"], true);
    assert!(export.get("history").is_none());
    assert!(export.get("session").is_none());
    assert!(export["map"].get("runtimeVersion").is_none());
    assert!(export["map"].get("schemaVersion").is_none());

    let ids: Vec<&str> = export["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|node| node["id"].as_str().unwrap())
        .collect();
    assert!(ids.contains(&intent.as_str()));
    assert!(ids.contains(&question.as_str()));
    assert!(ids.contains(&current.as_str()));
    assert!(!ids.contains(&old.as_str()));
    assert!(!ids.contains(&abandoned.as_str()));

    assert!(export["relationships"].as_array().unwrap().iter().any(|edge| {
        edge["kind"] == "answers" && edge["source"] == question && edge["target"] == current
    }));
}

#[test]
fn export_history_and_abandoned_are_independent_opt_ins() {
    let root = new_map("flags");
    let old = id(&ok_json(&root, &["create", "intent", "Old intent"]));
    let current = id(&ok_json(&root, &["create", "intent", "Current intent"]));
    ok_json(&root, &["replace", &old, &current, "--reason", "Refined"]);
    let abandoned = id(&ok_json(&root, &["create", "fact", "Obsolete fact"]));
    ok_json(
        &root,
        &["abandon", &abandoned, "--by", "user", "--reason", "Obsolete"],
    );

    let history_only = ok_json(&root, &["export", "--include-history"]);
    assert!(history_only["history"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|node| node["id"] == old));
    assert!(!history_only["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|node| node["id"] == abandoned));

    let full = ok_json(
        &root,
        &["export", "--include-history", "--include-abandoned"],
    );
    assert!(full["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|node| node["id"] == abandoned));
    assert_eq!(full["history"]["replacements"][0]["oldId"], old);
    assert_eq!(full["history"]["replacements"][0]["newId"], current);
}

#[test]
fn export_supports_formats_files_and_early_output_preflight() {
    let root = new_map("formats");
    ok_json(&root, &["create", "intent", "Build app"]);

    let toml_path = root.join("map-export.toml");
    let output = run(
        &root,
        &["export", "--format", "toml", "--output", toml_path.to_str().unwrap()],
    );
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(output.stdout.is_empty());
    let parsed: toml::Value = toml::from_str(&fs::read_to_string(&toml_path).unwrap()).unwrap();
    assert!(parsed["map"]["projectId"].as_str().is_some());

    let yaml = run(&root, &["export", "-f", "yaml"]);
    assert!(yaml.status.success(), "{}", String::from_utf8_lossy(&yaml.stderr));
    let yaml = String::from_utf8(yaml.stdout).unwrap();
    assert!(yaml.starts_with("map:\n"));
    assert!(yaml.contains("nodes:\n"));
    assert!(yaml.contains("relationships:\n"));

    let bad_target = root.join("output-directory");
    fs::create_dir(&bad_target).unwrap();
    let missing_map = root.join("missing-map");
    let failure = Command::new(bin())
        .env("USERPROFILE", &root)
        .env("HOME", &root)
        .arg("--path")
        .arg(&missing_map)
        .arg("export")
        .arg("-o")
        .arg(&bad_target)
        .output()
        .unwrap();
    assert!(!failure.status.success());
    let stderr = String::from_utf8_lossy(&failure.stderr);
    assert!(stderr.contains("not a file"));
    assert!(!stderr.contains("no .map exists"));
}
