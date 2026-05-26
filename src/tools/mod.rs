//! Declarative tool table. Each entry describes:
//! - its MCP-facing name, description, and JSON Schema
//! - how to extract `nodeIds` from the call args (most tools share a single rule)
//! - special handlers for the two tools that touch the local filesystem
//!
//! Compared to the Go version (one hand-rolled handler per tool), this collapses
//! ~700 lines of repetitive plumbing into a single data table plus a generic dispatcher.

pub mod definitions;
pub mod special;

use serde_json::{Map, Value};

/// How a tool extracts node IDs from its raw call arguments.
#[derive(Debug, Clone, Copy)]
pub enum NodeIds {
    /// No node IDs are passed; everything goes into `params`.
    None,
    /// Pull `args["nodeId"]` (string) into `nodeIds[0]`; everything else into `params`.
    SingleField,
    /// Pull `args["nodeId"]` if present (optional); used by find_replace_text.
    SingleOptional,
    /// Pull `args["nodeIds"]` (array of strings) into `nodeIds`; rest into `params`.
    Multi,
}

/// Custom-handler hook for tools whose Rust-side behaviour differs from the plain forward
/// (file IO for save_screenshots, PDF merging for export_frames_to_pdf).
pub type SpecialFn = fn(
    crate::handler::HandlerArc,
    Map<String, Value>,
) -> futures::future::BoxFuture<'static, Result<String, String>>;

pub struct ToolDef {
    pub name: &'static str,
    pub description: &'static str,
    /// JSON Schema for `inputSchema` — pre-built `serde_json::Value::Object`.
    pub input_schema: fn() -> Value,
    pub node_ids: NodeIds,
    /// Optional custom handler that overrides the default "validate → forward" pipeline.
    pub special: Option<SpecialFn>,
}

pub fn all() -> &'static [ToolDef] {
    definitions::TOOLS
}

pub fn find(name: &str) -> Option<&'static ToolDef> {
    definitions::TOOLS.iter().find(|t| t.name == name)
}

/// Split call args into (nodeIDs, params) according to a tool's node-id rule.
pub fn extract_node_ids(
    rule: NodeIds,
    mut args: Map<String, Value>,
) -> (Vec<String>, Map<String, Value>) {
    match rule {
        NodeIds::None => (Vec::new(), args),
        NodeIds::SingleField => {
            let id = args
                .remove("nodeId")
                .and_then(|v| match v {
                    Value::String(s) => Some(s),
                    _ => None,
                })
                .unwrap_or_default();
            (vec![id], args)
        }
        NodeIds::SingleOptional => {
            let id = args.remove("nodeId").and_then(|v| match v {
                Value::String(s) if !s.is_empty() => Some(s),
                _ => None,
            });
            (id.into_iter().collect(), args)
        }
        NodeIds::Multi => {
            let ids = args
                .remove("nodeIds")
                .and_then(|v| match v {
                    Value::Array(arr) => Some(
                        arr.into_iter()
                            .filter_map(|x| match x {
                                Value::String(s) => Some(s),
                                _ => None,
                            })
                            .collect::<Vec<_>>(),
                    ),
                    _ => None,
                })
                .unwrap_or_default();
            (ids, args)
        }
    }
}
