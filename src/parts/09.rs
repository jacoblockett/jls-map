#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectIdentity {
    project_id: String,
    created_at_ms: i64,
}

fn project_identity_path(map_dir: &Path) -> PathBuf {
    map_dir.join("project.json")
}

fn remove_legacy_ownership_marker(map_dir: &Path) -> Result<()> {
    let path = map_dir.join(".jls-owned.json");
    if !path.is_file() {
        return Ok(());
    }

    // This is migration cleanup only. The filename is not an ownership signal:
    // delete it only when it exactly matches the marker schema older JLS builds wrote.
    let Ok(text) = fs::read_to_string(&path) else {
        return Ok(());
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
        return Ok(());
    };
    let Some(marker) = value.as_object() else {
        return Ok(());
    };
    let owned = marker.len() == 4
        && marker.get("format") == Some(&serde_json::json!(1))
        && marker.get("owner") == Some(&serde_json::json!("jls"))
        && marker.get("kind") == Some(&serde_json::json!("generated-data"))
        && marker.get("skill") == Some(&serde_json::json!("map"));
    if !owned {
        return Ok(());
    }

    fs::remove_file(&path)
        .with_context(|| format!("removing legacy Map ownership marker {}", path.display()))?;
    Ok(())
}

fn valid_project_id(id: &str) -> bool {
    id.len() == 20
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
}

fn generate_project_id() -> String {
    const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    (0..20)
        .map(|_| ALPHABET[rng.gen_range(0..ALPHABET.len())] as char)
        .collect()
}

fn read_project_identity(map_dir: &Path) -> Result<Option<ProjectIdentity>> {
    let path = project_identity_path(map_dir);
    if !path.exists() {
        return Ok(None);
    }
    if !path.is_file() {
        bail!("Map project identity {} is not a file", path.display());
    }
    let text = fs::read_to_string(&path)
        .with_context(|| format!("reading Map project identity {}", path.display()))?;
    let identity: ProjectIdentity = serde_json::from_str(&text)
        .with_context(|| format!("parsing Map project identity {}", path.display()))?;
    if !valid_project_id(&identity.project_id) {
        bail!("Map project identity contains an invalid project ID");
    }
    Ok(Some(identity))
}

fn write_project_identity(map_dir: &Path, identity: &ProjectIdentity) -> Result<()> {
    let path = project_identity_path(map_dir);
    let tmp = map_dir.join(format!(
        ".project-{}-{}.tmp",
        std::process::id(),
        now_ms()
    ));
    fs::write(&tmp, format!("{}\n", serde_json::to_string_pretty(identity)?))
        .with_context(|| format!("writing Map project identity {}", tmp.display()))?;
    if let Err(error) = fs::rename(&tmp, &path) {
        let _ = fs::remove_file(&tmp);
        return Err(error).with_context(|| format!("committing Map project identity {}", path.display()));
    }
    Ok(())
}

fn create_project_identity(map_dir: &Path) -> Result<ProjectIdentity> {
    let path = project_identity_path(map_dir);
    if path.exists() {
        bail!("Map project identity {} already exists", path.display());
    }
    let identity = ProjectIdentity {
        project_id: generate_project_id(),
        created_at_ms: now_ms(),
    };
    write_project_identity(map_dir, &identity)?;
    remove_legacy_ownership_marker(map_dir)?;
    Ok(identity)
}

fn ensure_project_identity(map_dir: &Path) -> Result<ProjectIdentity> {
    let identity = read_project_identity(map_dir)?.ok_or_else(|| {
        anyhow!(
            "Map project identity {} is missing",
            project_identity_path(map_dir).display()
        )
    })?;
    remove_legacy_ownership_marker(map_dir)?;
    Ok(identity)
}
