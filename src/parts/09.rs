#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectIdentity {
    project_id: String,
    created_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeneratedDataOwnership {
    format: u8,
    owner: String,
    kind: String,
    skill: String,
}

fn project_identity_path(map_dir: &Path) -> PathBuf {
    map_dir.join("project.json")
}

fn generated_data_ownership_path(map_dir: &Path) -> PathBuf {
    map_dir.join(".jls-owned.json")
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

fn ensure_generated_data_ownership(map_dir: &Path) -> Result<()> {
    let path = generated_data_ownership_path(map_dir);
    if path.exists() {
        if !path.is_file() {
            bail!("Map generated-data ownership marker {} is not a file", path.display());
        }
        let marker: GeneratedDataOwnership = serde_json::from_str(
            &fs::read_to_string(&path)
                .with_context(|| format!("reading Map generated-data ownership marker {}", path.display()))?,
        )
        .with_context(|| format!("parsing Map generated-data ownership marker {}", path.display()))?;
        if marker.format != 1
            || marker.owner != "jls"
            || marker.kind != "generated-data"
            || marker.skill != "map"
        {
            bail!("Map generated-data ownership marker is invalid");
        }
        return Ok(());
    }

    let marker = GeneratedDataOwnership {
        format: 1,
        owner: "jls".to_string(),
        kind: "generated-data".to_string(),
        skill: "map".to_string(),
    };
    let tmp = map_dir.join(format!(
        ".jls-owned-{}-{}.tmp",
        std::process::id(),
        now_ms()
    ));
    fs::write(&tmp, format!("{}\n", serde_json::to_string_pretty(&marker)?))
        .with_context(|| format!("writing Map generated-data ownership marker {}", tmp.display()))?;
    if let Err(error) = fs::rename(&tmp, &path) {
        let _ = fs::remove_file(&tmp);
        return Err(error).with_context(|| {
            format!(
                "committing Map generated-data ownership marker {}",
                path.display()
            )
        });
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
    ensure_generated_data_ownership(map_dir)?;
    Ok(identity)
}

fn ensure_project_identity(map_dir: &Path) -> Result<ProjectIdentity> {
    let identity = read_project_identity(map_dir)?.ok_or_else(|| {
        anyhow!(
            "Map project identity {} is missing",
            project_identity_path(map_dir).display()
        )
    })?;
    ensure_generated_data_ownership(map_dir)?;
    Ok(identity)
}
