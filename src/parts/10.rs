#[derive(Args, Debug)]
struct ExportArgs {
    #[arg(short = 'f', long, value_enum, default_value = "json")]
    format: ExportFormat,
    #[arg(long)]
    include_history: bool,
    #[arg(long)]
    include_abandoned: bool,
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ExportFormat {
    Json,
    Yaml,
    Toml,
}

fn preflight_export_output(path: &Path) -> Result<()> {
    let target = expand_tilde(path);
    if target.exists() {
        if !target.is_file() {
            bail!("export output {} is not a file", target.display());
        }
        fs::OpenOptions::new()
            .write(true)
            .open(&target)
            .with_context(|| format!("export output {} is not writable", target.display()))?;
        return Ok(());
    }

    let parent = target
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if !parent.is_dir() {
        bail!("export output parent {} does not exist", parent.display());
    }

    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&target)
        .with_context(|| format!("export output {} is not writable", target.display()))?;
    drop(file);
    fs::remove_file(&target)
        .with_context(|| format!("removing export output preflight file {}", target.display()))?;
    Ok(())
}

async fn export_command(store: &Store, args: ExportArgs) -> Result<()> {
    let graph = store.graph().await?;
    let identity = ensure_project_identity(&store.map_dir)?;
    let errors = validate_graph_semantics(&graph);
    let historical = graph.historical_ids();

    let mut current_nodes: Vec<&DbNode> = graph
        .nodes
        .iter()
        .filter(|(id, node)| {
            !historical.contains(*id) && (args.include_abandoned || !node.data.abandoned)
        })
        .map(|(_, node)| node)
        .collect();
    current_nodes.sort_by_key(|node| node.key());
    let nodes: Vec<Value> = current_nodes
        .into_iter()
        .map(|node| node_output(&graph, node))
        .collect();

    let relationships: Vec<Value> = graph
        .normalized_edges(args.include_abandoned)
        .into_iter()
        .map(|edge| {
            json!({
                "kind": edge.kind.table(),
                "source": edge.source,
                "target": edge.target,
            })
        })
        .collect();

    let mut payload = json!({
        "map": {
            "projectId": identity.project_id,
            "depth": graph.meta.depth,
            "stance": graph.meta.stance,
        },
        "validation": {
            "ok": errors.is_empty(),
            "errors": errors,
        },
        "nodes": nodes,
        "relationships": relationships,
    });

    if args.include_history {
        let mut history_nodes: Vec<&DbNode> = historical
            .iter()
            .filter_map(|id| graph.nodes.get(id))
            .filter(|node| args.include_abandoned || !node.data.abandoned)
            .collect();
        history_nodes.sort_by_key(|node| node.key());
        let history_nodes: Vec<Value> = history_nodes
            .into_iter()
            .map(stored_node_output)
            .collect();

        let mut replacements = graph.replacements.clone();
        replacements.sort_by(|a, b| {
            (a.created_at_ms, &a.old_id, &a.new_id)
                .cmp(&(b.created_at_ms, &b.old_id, &b.new_id))
        });

        payload["history"] = json!({
            "nodes": history_nodes,
            "replacements": replacements,
        });
    }

    let rendered = render_export(&payload, args.format)?;
    if let Some(path) = args.output {
        let target = expand_tilde(&path);
        fs::write(&target, rendered)
            .with_context(|| format!("writing export output {}", target.display()))?;
    } else {
        print!("{rendered}");
    }
    Ok(())
}

fn stored_node_output(node: &DbNode) -> Value {
    let mut value = serde_json::to_value(&node.data).expect("serializable node");
    value
        .as_object_mut()
        .expect("node serializes as object")
        .insert("id".to_string(), json!(node.key()));
    value
}

fn render_export(value: &Value, format: ExportFormat) -> Result<String> {
    let mut rendered = match format {
        ExportFormat::Json => serde_json::to_string_pretty(value)?,
        ExportFormat::Yaml => render_yaml(value),
        ExportFormat::Toml => toml::to_string_pretty(value).context("serializing export as TOML")?,
    };
    if !rendered.ends_with('\n') {
        rendered.push('\n');
    }
    Ok(rendered)
}

fn render_yaml(value: &Value) -> String {
    let mut out = String::new();
    write_yaml(value, 0, &mut out);
    out
}

fn write_yaml(value: &Value, indent: usize, out: &mut String) {
    match value {
        Value::Object(map) if map.is_empty() => {
            out.push_str(&" ".repeat(indent));
            out.push_str("{}\n");
        }
        Value::Object(map) => {
            for (key, value) in map {
                out.push_str(&" ".repeat(indent));
                out.push_str(&yaml_key(key));
                out.push(':');
                if let Some(scalar) = yaml_scalar(value) {
                    out.push(' ');
                    out.push_str(&scalar);
                    out.push('\n');
                } else if is_empty_collection(value) {
                    out.push(' ');
                    out.push_str(if value.is_array() { "[]" } else { "{}" });
                    out.push('\n');
                } else {
                    out.push('\n');
                    write_yaml(value, indent + 2, out);
                }
            }
        }
        Value::Array(items) if items.is_empty() => {
            out.push_str(&" ".repeat(indent));
            out.push_str("[]\n");
        }
        Value::Array(items) => {
            for value in items {
                out.push_str(&" ".repeat(indent));
                out.push('-');
                if let Some(scalar) = yaml_scalar(value) {
                    out.push(' ');
                    out.push_str(&scalar);
                    out.push('\n');
                } else if is_empty_collection(value) {
                    out.push(' ');
                    out.push_str(if value.is_array() { "[]" } else { "{}" });
                    out.push('\n');
                } else {
                    out.push('\n');
                    write_yaml(value, indent + 2, out);
                }
            }
        }
        _ => {
            out.push_str(&" ".repeat(indent));
            out.push_str(&yaml_scalar(value).unwrap_or_else(|| "null".to_string()));
            out.push('\n');
        }
    }
}

fn yaml_scalar(value: &Value) -> Option<String> {
    match value {
        Value::Null => Some("null".to_string()),
        Value::Bool(value) => Some(value.to_string()),
        Value::Number(value) => Some(value.to_string()),
        Value::String(value) => Some(serde_json::to_string(value).expect("serializable string")),
        Value::Array(_) | Value::Object(_) => None,
    }
}

fn yaml_key(key: &str) -> String {
    if !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        && !key.chars().next().unwrap().is_ascii_digit()
    {
        key.to_string()
    } else {
        serde_json::to_string(key).expect("serializable key")
    }
}

fn is_empty_collection(value: &Value) -> bool {
    matches!(value, Value::Array(items) if items.is_empty())
        || matches!(value, Value::Object(map) if map.is_empty())
}
