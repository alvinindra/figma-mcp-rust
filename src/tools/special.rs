//! Tools that need server-side filesystem access: `save_screenshots` and `export_frames_to_pdf`.

use std::path::{Path, PathBuf};

use base64::{engine::general_purpose::STANDARD as B64, Engine};
use futures::future::BoxFuture;
use futures::FutureExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::handler::HandlerArc;
use crate::pdf;

#[derive(Debug, Deserialize)]
struct SaveItem {
    #[serde(rename = "nodeId")]
    node_id: String,
    #[serde(rename = "outputPath")]
    output_path: String,
    #[serde(default)]
    format: Option<String>,
    #[serde(default)]
    scale: Option<f64>,
}

#[derive(Debug, Serialize, Default)]
struct SaveResult {
    index: usize,
    #[serde(rename = "nodeId", skip_serializing_if = "String::is_empty")]
    node_id: String,
    #[serde(rename = "nodeName", skip_serializing_if = "String::is_empty")]
    node_name: String,
    #[serde(rename = "outputPath", skip_serializing_if = "String::is_empty")]
    output_path: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    format: String,
    #[serde(skip_serializing_if = "is_zero_f64")]
    width: f64,
    #[serde(skip_serializing_if = "is_zero_f64")]
    height: f64,
    #[serde(rename = "bytesWritten", skip_serializing_if = "is_zero")]
    bytes_written: usize,
    success: bool,
    #[serde(skip_serializing_if = "String::is_empty")]
    error: String,
}

fn is_zero_f64(n: &f64) -> bool {
    *n == 0.0
}
fn is_zero(n: &usize) -> bool {
    *n == 0
}

pub fn save_screenshots(
    handler: HandlerArc,
    args: Map<String, Value>,
) -> BoxFuture<'static, Result<String, String>> {
    async move {
        let raw_items = args
            .get("items")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let default_format = args
            .get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let default_scale = args.get("scale").and_then(|v| v.as_f64()).unwrap_or(0.0);

        let work_dir = std::env::current_dir().map_err(|e| format!("getwd: {e}"))?;

        let mut results: Vec<SaveResult> = Vec::with_capacity(raw_items.len());
        let mut succeeded = 0;
        let mut failed = 0;

        for (i, raw) in raw_items.into_iter().enumerate() {
            let item: SaveItem = match serde_json::from_value(raw) {
                Ok(it) => it,
                Err(e) => {
                    results.push(SaveResult {
                        index: i,
                        error: e.to_string(),
                        ..Default::default()
                    });
                    failed += 1;
                    continue;
                }
            };
            let r = save_one(
                &handler,
                &item,
                i,
                &work_dir,
                &default_format,
                default_scale,
            )
            .await;
            if r.success {
                succeeded += 1;
            } else {
                failed += 1;
            }
            results.push(r);
        }

        let out = json!({
            "total": results.len(),
            "succeeded": succeeded,
            "failed": failed,
            "hasErrors": failed > 0,
            "results": results,
        });
        Ok(out.to_string())
    }
    .boxed()
}

#[derive(Debug, Deserialize)]
struct ScreenshotExport {
    #[serde(rename = "nodeId", default)]
    node_id: String,
    #[serde(rename = "nodeName", default)]
    node_name: String,
    #[serde(default)]
    base64: String,
    #[serde(default)]
    width: f64,
    #[serde(default)]
    height: f64,
}

async fn save_one(
    handler: &HandlerArc,
    item: &SaveItem,
    index: usize,
    work_dir: &Path,
    default_format: &str,
    default_scale: f64,
) -> SaveResult {
    let resolved = match resolve_output_path(&item.output_path, work_dir) {
        Ok(p) => p,
        Err(e) => {
            return SaveResult {
                index,
                node_id: item.node_id.clone(),
                output_path: item.output_path.clone(),
                error: e,
                ..Default::default()
            }
        }
    };

    let mut format = item
        .format
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| default_format.to_string());
    let inferred = infer_format(&resolved);
    if format.is_empty() {
        format = inferred.clone();
    }
    if format.is_empty() {
        format = "PNG".into();
    }
    if !inferred.is_empty() && format != inferred {
        return SaveResult {
            index,
            node_id: item.node_id.clone(),
            output_path: resolved.display().to_string(),
            error: format!(
                "format {} conflicts with file extension {}",
                format, inferred
            ),
            ..Default::default()
        };
    }

    let scale = item.scale.filter(|s| *s > 0.0).unwrap_or(default_scale);
    let mut params: Map<String, Value> = Map::new();
    params.insert("format".into(), Value::String(format.clone()));
    if scale > 0.0 {
        params.insert("scale".into(), json!(scale));
    }

    let resp = match handler
        .node
        .send("get_screenshot", vec![item.node_id.clone()], params)
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return SaveResult {
                index,
                node_id: item.node_id.clone(),
                output_path: resolved.display().to_string(),
                error: e.to_string(),
                ..Default::default()
            }
        }
    };
    if !resp.error.is_empty() {
        return SaveResult {
            index,
            node_id: item.node_id.clone(),
            output_path: resolved.display().to_string(),
            error: resp.error,
            ..Default::default()
        };
    }

    let export = match extract_screenshot(resp.data.unwrap_or(Value::Null)) {
        Ok(e) => e,
        Err(e) => {
            return SaveResult {
                index,
                node_id: item.node_id.clone(),
                output_path: resolved.display().to_string(),
                error: e,
                ..Default::default()
            }
        }
    };

    match write_base64(&export.base64, &resolved).await {
        Ok(n) => SaveResult {
            index,
            node_id: export.node_id,
            node_name: export.node_name,
            output_path: resolved.display().to_string(),
            format,
            width: export.width,
            height: export.height,
            bytes_written: n,
            success: true,
            error: String::new(),
        },
        Err(e) => SaveResult {
            index,
            node_id: item.node_id.clone(),
            output_path: resolved.display().to_string(),
            error: e,
            ..Default::default()
        },
    }
}

fn extract_screenshot(data: Value) -> Result<ScreenshotExport, String> {
    #[derive(Deserialize)]
    struct Wrap {
        exports: Vec<ScreenshotExport>,
    }
    let wrap: Wrap = serde_json::from_value(data).map_err(|e| e.to_string())?;
    wrap.exports
        .into_iter()
        .next()
        .ok_or_else(|| "no screenshot export returned by plugin".into())
}

async fn write_base64(b64: &str, path: &Path) -> Result<usize, String> {
    let data = B64.decode(b64).map_err(|e| format!("base64 decode: {e}"))?;
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("mkdir: {e}"))?;
    }
    // Exclusive create — refuse to overwrite, matching the Go server.
    let mut opts = tokio::fs::OpenOptions::new();
    opts.write(true).create_new(true);
    let mut f = opts.open(path).await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::AlreadyExists {
            format!("file already exists at outputPath: {}", path.display())
        } else {
            e.to_string()
        }
    })?;
    use tokio::io::AsyncWriteExt;
    f.write_all(&data).await.map_err(|e| e.to_string())?;
    f.flush().await.map_err(|e| e.to_string())?;
    Ok(data.len())
}

pub fn export_frames_to_pdf(
    handler: HandlerArc,
    mut args: Map<String, Value>,
) -> BoxFuture<'static, Result<String, String>> {
    async move {
        let node_ids: Vec<String> = args
            .remove("nodeIds")
            .and_then(|v| match v {
                Value::Array(a) => Some(a),
                _ => None,
            })
            .map(|arr| {
                arr.into_iter()
                    .filter_map(|x| match x {
                        Value::String(s) => Some(s),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default();
        let output_path = args
            .remove("outputPath")
            .and_then(|v| match v {
                Value::String(s) => Some(s),
                _ => None,
            })
            .unwrap_or_default();
        if output_path.is_empty() {
            return Err("outputPath is required".into());
        }

        let work_dir = std::env::current_dir().map_err(|e| format!("getwd: {e}"))?;
        let resolved = resolve_output_path(&output_path, &work_dir)?;
        if resolved
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_ascii_lowercase())
            .as_deref()
            != Some("pdf")
        {
            return Err("outputPath must have a .pdf extension".into());
        }

        let resp = handler
            .node
            .send("export_frames_to_pdf", node_ids, Map::new())
            .await
            .map_err(|e| e.to_string())?;
        if !resp.error.is_empty() {
            return Err(resp.error);
        }

        let pages = extract_frame_pdfs(resp.data.unwrap_or(Value::Null))?;
        let merged = pdf::merge_pdfs(&pages).map_err(|e| format!("merge PDFs: {e}"))?;

        if let Some(parent) = resolved.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("mkdir: {e}"))?;
        }
        if tokio::fs::try_exists(&resolved)
            .await
            .map_err(|e| e.to_string())?
        {
            return Err(format!("file already exists: {}", resolved.display()));
        }
        tokio::fs::write(&resolved, &merged)
            .await
            .map_err(|e| format!("write file: {e}"))?;

        Ok(json!({
            "outputPath": resolved.display().to_string(),
            "bytesWritten": merged.len(),
            "pageCount": pages.len(),
            "success": true,
        })
        .to_string())
    }
    .boxed()
}

fn extract_frame_pdfs(data: Value) -> Result<Vec<Vec<u8>>, String> {
    #[derive(Deserialize)]
    struct Frame {
        #[serde(default)]
        base64: String,
    }
    #[derive(Deserialize)]
    struct Wrap {
        frames: Vec<Frame>,
    }
    let wrap: Wrap = serde_json::from_value(data).map_err(|e| e.to_string())?;
    if wrap.frames.is_empty() {
        return Err("no PDF frames returned by plugin".into());
    }
    let mut out = Vec::with_capacity(wrap.frames.len());
    for (i, f) in wrap.frames.into_iter().enumerate() {
        if f.base64.is_empty() {
            return Err(format!("frame {i} has empty base64"));
        }
        out.push(
            B64.decode(f.base64)
                .map_err(|e| format!("frame {i}: base64 decode: {e}"))?,
        );
    }
    Ok(out)
}

fn resolve_output_path(output: &str, work_dir: &Path) -> Result<PathBuf, String> {
    let candidate = if Path::new(output).is_absolute() {
        PathBuf::from(output)
    } else {
        work_dir.join(output)
    };
    let canonical_root = work_dir.to_path_buf();
    // Normalise without requiring the file to exist (would fail on writes).
    let normalised = normalise_path(&candidate);
    if !normalised.starts_with(&canonical_root) {
        return Err(format!(
            "outputPath must be inside the working directory: {}",
            canonical_root.display()
        ));
    }
    Ok(normalised)
}

fn normalise_path(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in p.components() {
        match comp {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            _ => out.push(comp.as_os_str()),
        }
    }
    out
}

fn infer_format(path: &Path) -> String {
    match path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase())
        .as_deref()
    {
        Some("png") => "PNG".into(),
        Some("svg") => "SVG".into(),
        Some("jpg") | Some("jpeg") => "JPG".into(),
        Some("pdf") => "PDF".into(),
        _ => String::new(),
    }
}
