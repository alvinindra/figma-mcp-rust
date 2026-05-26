//! Node-ID format validation and per-tool RPC parameter validation.
//!
//! Mirrors the Go `internal/schema.go` behaviour so that follower → leader RPCs
//! get the same diagnostics as direct tool calls.

use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::{Map, Value};

static NODE_ID_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^I?\d+:\d+(;\d+:\d+)*$").expect("hard-coded regex"));

/// Convert hyphen-format node IDs (e.g. "4029-12345") that LLMs sometimes emit
/// into Figma's canonical colon format ("4029:12345"). No-op for anything else.
pub fn normalize_node_id(s: &str) -> String {
    if s.contains('-') && !s.contains(':') {
        let candidate = s.replace('-', ":");
        if NODE_ID_RE.is_match(&candidate) {
            return candidate;
        }
    }
    s.to_string()
}

pub fn valid_node_id(s: &str) -> bool {
    NODE_ID_RE.is_match(s)
}

fn ensure_valid_node_id(id: &str) -> Option<String> {
    if valid_node_id(id) {
        None
    } else {
        Some(format!(
            "invalid nodeId: {} — must use colon format e.g. 4029:12345",
            id
        ))
    }
}

fn ensure_all_valid(ids: &[String]) -> Option<String> {
    for id in ids {
        if let Some(msg) = ensure_valid_node_id(id) {
            return Some(msg);
        }
    }
    None
}

fn str_param<'a>(params: &'a Map<String, Value>, key: &str) -> Option<&'a str> {
    params.get(key).and_then(|v| v.as_str())
}

fn f64_param(params: &Map<String, Value>, key: &str) -> Option<f64> {
    params.get(key).and_then(|v| v.as_f64())
}

fn bool_param(params: &Map<String, Value>, key: &str) -> Option<bool> {
    params.get(key).and_then(|v| v.as_bool())
}

fn array_param<'a>(params: &'a Map<String, Value>, key: &str) -> Option<&'a Vec<Value>> {
    params.get(key).and_then(|v| v.as_array())
}

/// Validate an RPC for a given tool. Returns `None` on success, or an error string.
///
/// This is a 1:1 port of the Go `ValidateRPC` switch — every branch mirrors the
/// original behaviour so existing follower clients see identical error text.
pub fn validate_rpc(
    tool: &str,
    node_ids: &[String],
    params: &Map<String, Value>,
) -> Option<String> {
    match tool {
        // ─── Read tools ─────────────────────────────────────────────
        "get_node" | "get_reactions" | "create_component" => require_single_node_id(node_ids),

        "get_nodes_info" | "export_frames_to_pdf" => {
            if node_ids.is_empty() {
                return Some("nodeIds is required and must not be empty".into());
            }
            ensure_all_valid(node_ids)
        }

        "get_screenshot" => {
            if let Some(msg) = ensure_all_valid(node_ids) {
                return Some(msg);
            }
            if let Some(fmt) = str_param(params, "format") {
                if !is_valid_export_format(fmt) {
                    return Some(format!(
                        "format must be PNG, SVG, JPG, or PDF, got: {}",
                        fmt
                    ));
                }
            }
            None
        }

        "save_screenshots" => validate_save_screenshots(params),

        "get_design_context" => {
            if let Some(depth) = f64_param(params, "depth") {
                if depth < 0.0 {
                    return Some("depth must be a non-negative number".into());
                }
            }
            if let Some(detail) = str_param(params, "detail") {
                if !detail.is_empty() && !matches!(detail, "minimal" | "compact" | "full") {
                    return Some(format!(
                        "detail must be minimal, compact, or full, got: {}",
                        detail
                    ));
                }
            }
            None
        }

        "search_nodes" => {
            let query = str_param(params, "query").unwrap_or("");
            if query.is_empty() {
                return Some("query is required".into());
            }
            if let Some(id) = str_param(params, "nodeId") {
                if !id.is_empty() && !valid_node_id(id) {
                    return Some(format!(
                        "nodeId must use colon format e.g. 4029:12345, got: {}",
                        id
                    ));
                }
            }
            if let Some(limit) = f64_param(params, "limit") {
                if limit <= 0.0 {
                    return Some("limit must be a positive number".into());
                }
            }
            None
        }

        "scan_text_nodes" | "scan_nodes_by_types" => {
            let node_id = str_param(params, "nodeId").unwrap_or("");
            if node_id.is_empty() {
                return Some("nodeId is required".into());
            }
            if !valid_node_id(node_id) {
                return Some(format!(
                    "nodeId must use colon format e.g. 4029:12345, got: {}",
                    node_id
                ));
            }
            if tool == "scan_nodes_by_types" {
                let types = array_param(params, "types");
                if types.map_or(true, |a| a.is_empty()) {
                    return Some("types must be a non-empty array".into());
                }
            }
            None
        }

        // ─── Write modify ───────────────────────────────────────────
        "set_opacity" => {
            if let Some(m) = require_node_ids(node_ids) {
                return Some(m);
            }
            let op = match f64_param(params, "opacity") {
                Some(v) => v,
                None => return Some("opacity is required".into()),
            };
            if !(0.0..=1.0).contains(&op) {
                return Some("opacity must be between 0 and 1".into());
            }
            None
        }

        "set_corner_radius" => {
            if let Some(m) = require_node_ids(node_ids) {
                return Some(m);
            }
            if ![
                "cornerRadius",
                "topLeftRadius",
                "topRightRadius",
                "bottomLeftRadius",
                "bottomRightRadius",
            ]
            .iter()
            .any(|k| params.contains_key(*k))
            {
                return Some(
                    "at least one of cornerRadius, topLeftRadius, topRightRadius, bottomLeftRadius, or bottomRightRadius is required".into(),
                );
            }
            None
        }

        "group_nodes" => {
            if node_ids.len() < 2 {
                return Some("nodeIds must contain at least 2 nodes to group".into());
            }
            ensure_all_valid(node_ids)
        }

        "ungroup_nodes" | "delete_nodes" | "detach_instance" => {
            if node_ids.is_empty() {
                return Some("nodeIds is required and must not be empty".into());
            }
            ensure_all_valid(node_ids)
        }

        "navigate_to_page" => {
            let id = str_param(params, "pageId").unwrap_or("");
            let name = str_param(params, "pageName").unwrap_or("");
            if id.is_empty() && name.is_empty() {
                return Some("pageId or pageName is required".into());
            }
            None
        }

        "export_tokens" => {
            if let Some(fmt) = str_param(params, "format") {
                if !fmt.is_empty() && !matches!(fmt, "json" | "css") {
                    return Some(format!("format must be json or css, got: {}", fmt));
                }
            }
            None
        }

        "create_frame" => {
            if let Some(w) = f64_param(params, "width") {
                if w <= 0.0 {
                    return Some("width must be positive".into());
                }
            }
            if let Some(h) = f64_param(params, "height") {
                if h <= 0.0 {
                    return Some("height must be positive".into());
                }
            }
            if let Some(pid) = str_param(params, "parentId") {
                if !pid.is_empty() && !valid_node_id(pid) {
                    return Some(format!(
                        "parentId must use colon format e.g. 4029:12345, got: {}",
                        pid
                    ));
                }
            }
            validate_auto_layout(params)
        }

        "set_auto_layout" => {
            if let Some(m) = require_single_node_id(node_ids) {
                return Some(m);
            }
            validate_auto_layout(params)
        }

        "create_rectangle" | "create_ellipse" => {
            if let Some(w) = f64_param(params, "width") {
                if w <= 0.0 {
                    return Some("width must be positive".into());
                }
            }
            if let Some(h) = f64_param(params, "height") {
                if h <= 0.0 {
                    return Some("height must be positive".into());
                }
            }
            if let Some(pid) = str_param(params, "parentId") {
                if !pid.is_empty() && !valid_node_id(pid) {
                    return Some(format!(
                        "parentId must use colon format e.g. 4029:12345, got: {}",
                        pid
                    ));
                }
            }
            None
        }

        "create_text" => {
            let text = str_param(params, "text").unwrap_or("");
            if text.is_empty() {
                return Some("text is required".into());
            }
            if let Some(pid) = str_param(params, "parentId") {
                if !pid.is_empty() && !valid_node_id(pid) {
                    return Some(format!(
                        "parentId must use colon format e.g. 4029:12345, got: {}",
                        pid
                    ));
                }
            }
            None
        }

        "set_text" => {
            if let Some(m) = require_single_node_id(node_ids) {
                return Some(m);
            }
            if !params.get("text").is_some_and(|v| v.is_string()) {
                return Some("text is required".into());
            }
            None
        }

        "set_fills" | "set_strokes" => {
            if let Some(m) = require_single_node_id(node_ids) {
                return Some(m);
            }
            let color = str_param(params, "color").unwrap_or("");
            if color.is_empty() {
                return Some("color is required (hex string e.g. #FF5733)".into());
            }
            if let Some(mode) = str_param(params, "mode") {
                if !matches!(mode, "replace" | "append") {
                    return Some("mode must be 'replace' or 'append'".into());
                }
            }
            None
        }

        "move_nodes" => {
            if let Some(m) = require_node_ids(node_ids) {
                return Some(m);
            }
            if !params.contains_key("x") && !params.contains_key("y") {
                return Some("at least one of x or y is required".into());
            }
            None
        }

        "resize_nodes" => {
            if let Some(m) = require_node_ids(node_ids) {
                return Some(m);
            }
            if !params.contains_key("width") && !params.contains_key("height") {
                return Some("at least one of width or height is required".into());
            }
            None
        }

        "rename_node" => {
            if let Some(m) = require_single_node_id(node_ids) {
                return Some(m);
            }
            if str_param(params, "name").unwrap_or("").is_empty() {
                return Some("name is required".into());
            }
            None
        }

        "clone_node" => {
            if let Some(m) = require_single_node_id(node_ids) {
                return Some(m);
            }
            if let Some(pid) = str_param(params, "parentId") {
                if !pid.is_empty() && !valid_node_id(pid) {
                    return Some(format!(
                        "parentId must use colon format e.g. 4029:12345, got: {}",
                        pid
                    ));
                }
            }
            None
        }

        "import_image" => {
            if str_param(params, "imageData").unwrap_or("").is_empty() {
                return Some("imageData (base64) is required".into());
            }
            if let Some(sm) = str_param(params, "scaleMode") {
                if !sm.is_empty() && !matches!(sm, "FILL" | "FIT" | "CROP" | "TILE") {
                    return Some(format!(
                        "scaleMode must be FILL, FIT, CROP, or TILE, got: {}",
                        sm
                    ));
                }
            }
            if let Some(pid) = str_param(params, "parentId") {
                if !pid.is_empty() && !valid_node_id(pid) {
                    return Some(format!(
                        "parentId must use colon format e.g. 4029:12345, got: {}",
                        pid
                    ));
                }
            }
            None
        }

        // ─── Style tools ────────────────────────────────────────────
        "create_paint_style" => {
            if str_param(params, "name").unwrap_or("").is_empty() {
                return Some("name is required".into());
            }
            if str_param(params, "color").unwrap_or("").is_empty() {
                return Some("color is required (hex string e.g. #FF5733)".into());
            }
            None
        }

        "create_text_style" => {
            if str_param(params, "name").unwrap_or("").is_empty() {
                return Some("name is required".into());
            }
            if let Some(td) = str_param(params, "textDecoration") {
                if !td.is_empty() && !matches!(td, "NONE" | "UNDERLINE" | "STRIKETHROUGH") {
                    return Some(format!(
                        "textDecoration must be NONE, UNDERLINE, or STRIKETHROUGH, got: {}",
                        td
                    ));
                }
            }
            for k in ["lineHeightUnit", "letterSpacingUnit"] {
                if let Some(u) = str_param(params, k) {
                    if !u.is_empty() && !matches!(u, "PIXELS" | "PERCENT") {
                        return Some(format!("{} must be PIXELS or PERCENT, got: {}", k, u));
                    }
                }
            }
            None
        }

        "create_effect_style" => {
            if str_param(params, "name").unwrap_or("").is_empty() {
                return Some("name is required".into());
            }
            if let Some(t) = str_param(params, "type") {
                if !t.is_empty()
                    && !matches!(
                        t,
                        "DROP_SHADOW" | "INNER_SHADOW" | "LAYER_BLUR" | "BACKGROUND_BLUR"
                    )
                {
                    return Some(format!(
                        "type must be DROP_SHADOW, INNER_SHADOW, LAYER_BLUR, or BACKGROUND_BLUR, got: {}",
                        t
                    ));
                }
            }
            None
        }

        "create_grid_style" => {
            if str_param(params, "name").unwrap_or("").is_empty() {
                return Some("name is required".into());
            }
            if let Some(p) = str_param(params, "pattern") {
                if !p.is_empty() && !matches!(p, "GRID" | "COLUMNS" | "ROWS") {
                    return Some(format!(
                        "pattern must be GRID, COLUMNS, or ROWS, got: {}",
                        p
                    ));
                }
            }
            if let Some(a) = str_param(params, "alignment") {
                if !a.is_empty() && !matches!(a, "STRETCH" | "CENTER" | "MIN" | "MAX") {
                    return Some(format!(
                        "alignment must be STRETCH, CENTER, MIN, or MAX, got: {}",
                        a
                    ));
                }
            }
            None
        }

        "update_paint_style" => {
            if str_param(params, "styleId").unwrap_or("").is_empty() {
                return Some("styleId is required".into());
            }
            if !["name", "color", "description"]
                .iter()
                .any(|k| params.contains_key(*k))
            {
                return Some("at least one of name, color, or description is required".into());
            }
            None
        }

        "delete_style" => {
            if str_param(params, "styleId").unwrap_or("").is_empty() {
                return Some("styleId is required".into());
            }
            None
        }

        // ─── Variables ─────────────────────────────────────────────
        "create_variable_collection" => {
            if str_param(params, "name").unwrap_or("").is_empty() {
                return Some("name is required".into());
            }
            None
        }

        "add_variable_mode" => {
            if str_param(params, "collectionId").unwrap_or("").is_empty() {
                return Some("collectionId is required".into());
            }
            if str_param(params, "modeName").unwrap_or("").is_empty() {
                return Some("modeName is required".into());
            }
            None
        }

        "create_variable" => {
            if str_param(params, "name").unwrap_or("").is_empty() {
                return Some("name is required".into());
            }
            if str_param(params, "collectionId").unwrap_or("").is_empty() {
                return Some("collectionId is required".into());
            }
            let vt = str_param(params, "type").unwrap_or("");
            if !matches!(vt, "COLOR" | "FLOAT" | "STRING" | "BOOLEAN") {
                return Some(format!(
                    "type must be COLOR, FLOAT, STRING, or BOOLEAN, got: {}",
                    vt
                ));
            }
            None
        }

        "set_variable_value" => {
            if str_param(params, "variableId").unwrap_or("").is_empty() {
                return Some("variableId is required".into());
            }
            if str_param(params, "modeId").unwrap_or("").is_empty() {
                return Some("modeId is required".into());
            }
            if !params.contains_key("value") {
                return Some("value is required".into());
            }
            None
        }

        "delete_variable" => {
            let vid = str_param(params, "variableId").unwrap_or("");
            let cid = str_param(params, "collectionId").unwrap_or("");
            if vid.is_empty() && cid.is_empty() {
                return Some("variableId or collectionId is required".into());
            }
            None
        }

        // ─── Linked ───────────────────────────────────────────────
        "apply_style_to_node" => {
            if let Some(m) = require_single_node_id(node_ids) {
                return Some(m);
            }
            if str_param(params, "styleId").unwrap_or("").is_empty() {
                return Some("styleId is required".into());
            }
            if let Some(target) = str_param(params, "target") {
                if !target.is_empty() && !matches!(target, "fill" | "stroke") {
                    return Some(format!("target must be fill or stroke, got: {}", target));
                }
            }
            None
        }

        "bind_variable_to_node" => {
            if let Some(m) = require_single_node_id(node_ids) {
                return Some(m);
            }
            if str_param(params, "variableId").unwrap_or("").is_empty() {
                return Some("variableId is required".into());
            }
            if str_param(params, "field").unwrap_or("").is_empty() {
                return Some("field is required".into());
            }
            None
        }

        "swap_component" => {
            if let Some(m) = require_single_node_id(node_ids) {
                return Some(m);
            }
            let cid = str_param(params, "componentId").unwrap_or("");
            if cid.is_empty() {
                return Some("componentId is required".into());
            }
            if !valid_node_id(cid) {
                return Some(format!(
                    "componentId must use colon format e.g. 4029:12345, got: {}",
                    cid
                ));
            }
            None
        }

        // ─── Prototype ─────────────────────────────────────────────
        "set_reactions" => validate_set_reactions(node_ids, params),

        "remove_reactions" => {
            if let Some(m) = require_single_node_id(node_ids) {
                return Some(m);
            }
            if let Some(arr) = array_param(params, "indices") {
                for (i, v) in arr.iter().enumerate() {
                    if !v.is_number() {
                        return Some(format!("indices[{}] must be a number", i));
                    }
                }
            }
            None
        }

        // ─── Node control ─────────────────────────────────────────
        "set_visible" => {
            if let Some(m) = require_node_ids(node_ids) {
                return Some(m);
            }
            if bool_param(params, "visible").is_none() {
                return Some("visible (boolean) is required".into());
            }
            None
        }

        "lock_nodes" | "unlock_nodes" => require_node_ids(node_ids),

        "rotate_nodes" => {
            if let Some(m) = require_node_ids(node_ids) {
                return Some(m);
            }
            if f64_param(params, "rotation").is_none() {
                return Some("rotation (degrees) is required".into());
            }
            None
        }

        "reorder_nodes" => {
            if let Some(m) = require_node_ids(node_ids) {
                return Some(m);
            }
            let order = str_param(params, "order").unwrap_or("");
            if !matches!(
                order,
                "bringToFront" | "sendToBack" | "bringForward" | "sendBackward"
            ) {
                return Some(format!(
                    "order must be bringToFront, sendToBack, bringForward, or sendBackward, got: {}",
                    order
                ));
            }
            None
        }

        "set_blend_mode" => {
            if let Some(m) = require_node_ids(node_ids) {
                return Some(m);
            }
            let blend = str_param(params, "blendMode").unwrap_or("");
            if blend.is_empty() {
                return Some("blendMode is required".into());
            }
            if !is_valid_blend_mode(blend) {
                return Some(format!(
                    "blendMode \"{}\" is not a valid Figma blend mode",
                    blend
                ));
            }
            None
        }

        "set_constraints" => {
            if let Some(m) = require_node_ids(node_ids) {
                return Some(m);
            }
            if !params.contains_key("horizontal") && !params.contains_key("vertical") {
                return Some("at least one of horizontal or vertical is required".into());
            }
            for (k, label) in [("horizontal", "horizontal"), ("vertical", "vertical")] {
                if let Some(v) = str_param(params, k) {
                    if !v.is_empty() && !matches!(v, "MIN" | "MAX" | "CENTER" | "STRETCH" | "SCALE")
                    {
                        return Some(format!(
                            "{} must be MIN, MAX, CENTER, STRETCH, or SCALE, got: {}",
                            label, v
                        ));
                    }
                }
            }
            None
        }

        "reparent_nodes" => {
            if let Some(m) = require_node_ids(node_ids) {
                return Some(m);
            }
            let pid = str_param(params, "parentId").unwrap_or("");
            if pid.is_empty() {
                return Some("parentId is required".into());
            }
            if !valid_node_id(pid) {
                return Some(format!(
                    "parentId must use colon format e.g. 4029:12345, got: {}",
                    pid
                ));
            }
            None
        }

        "batch_rename_nodes" => {
            if let Some(m) = require_node_ids(node_ids) {
                return Some(m);
            }
            let has_find = params.contains_key("find");
            let has_replace = params.contains_key("replace");
            let has_prefix = params.contains_key("prefix");
            let has_suffix = params.contains_key("suffix");
            if !has_find && !has_replace && !has_prefix && !has_suffix {
                return Some("at least one of find/replace, prefix, or suffix is required".into());
            }
            if has_find && !has_replace {
                return Some("replace is required when find is provided".into());
            }
            None
        }

        "find_replace_text" => {
            let find = str_param(params, "find").unwrap_or("");
            if find.is_empty() {
                return Some("find is required".into());
            }
            if !params.contains_key("replace") {
                return Some("replace is required".into());
            }
            if let Some(id) = str_param(params, "nodeId") {
                if !id.is_empty() && !valid_node_id(id) {
                    return Some(format!(
                        "nodeId must use colon format e.g. 4029:12345, got: {}",
                        id
                    ));
                }
            }
            if let Some(first) = node_ids.first() {
                if !first.is_empty() && !valid_node_id(first) {
                    return Some(format!(
                        "nodeId must use colon format e.g. 4029:12345, got: {}",
                        first
                    ));
                }
            }
            None
        }

        // ─── Pages ────────────────────────────────────────────────
        "add_page" => {
            if let Some(idx) = f64_param(params, "index") {
                if idx < 0.0 {
                    return Some("index must be non-negative".into());
                }
            }
            None
        }

        "delete_page" | "rename_page" => {
            let id = str_param(params, "pageId").unwrap_or("");
            let name = str_param(params, "pageName").unwrap_or("");
            if id.is_empty() && name.is_empty() {
                return Some("pageId or pageName is required".into());
            }
            if tool == "rename_page" && str_param(params, "newName").unwrap_or("").is_empty() {
                return Some("newName is required".into());
            }
            None
        }

        "set_effects" => {
            if let Some(m) = require_single_node_id(node_ids) {
                return Some(m);
            }
            let arr = match array_param(params, "effects") {
                Some(a) => a,
                None => {
                    return Some(match params.get("effects") {
                        Some(_) => "effects must be an array".into(),
                        None => "effects array is required".into(),
                    })
                }
            };
            for (i, e) in arr.iter().enumerate() {
                let obj = match e.as_object() {
                    Some(o) => o,
                    None => return Some(format!("effects[{}] must be an object", i)),
                };
                let t = obj.get("type").and_then(|v| v.as_str()).unwrap_or("");
                if !matches!(
                    t,
                    "DROP_SHADOW" | "INNER_SHADOW" | "LAYER_BLUR" | "BACKGROUND_BLUR"
                ) {
                    return Some(format!(
                        "effects[{}].type must be DROP_SHADOW, INNER_SHADOW, LAYER_BLUR, or BACKGROUND_BLUR, got: {}",
                        i, t
                    ));
                }
            }
            None
        }

        "create_section" => {
            if let Some(w) = f64_param(params, "width") {
                if w <= 0.0 {
                    return Some("width must be positive".into());
                }
            }
            if let Some(h) = f64_param(params, "height") {
                if h <= 0.0 {
                    return Some("height must be positive".into());
                }
            }
            None
        }

        // Unknown tool — pass through (allows the plugin to error if not supported)
        _ => None,
    }
}

fn require_single_node_id(ids: &[String]) -> Option<String> {
    let first = ids.first().map(String::as_str).unwrap_or("");
    if first.is_empty() {
        return Some("nodeId is required".into());
    }
    if !valid_node_id(first) {
        return Some(format!(
            "nodeId must use colon format e.g. 4029:12345, got: {}",
            first
        ));
    }
    None
}

fn require_node_ids(ids: &[String]) -> Option<String> {
    if ids.is_empty() {
        return Some("nodeIds is required".into());
    }
    ensure_all_valid(ids)
}

fn validate_save_screenshots(params: &Map<String, Value>) -> Option<String> {
    let items = match params.get("items") {
        Some(v) => v,
        None => return Some("items is required".into()),
    };
    let arr = match items.as_array() {
        Some(a) if !a.is_empty() => a,
        _ => return Some("items must be a non-empty array".into()),
    };
    for (i, raw) in arr.iter().enumerate() {
        let obj = match raw.as_object() {
            Some(o) => o,
            None => return Some(format!("items[{}] must be an object", i)),
        };
        let node_id = obj.get("nodeId").and_then(|v| v.as_str()).unwrap_or("");
        if !valid_node_id(node_id) {
            return Some(format!(
                "items[{}].nodeId must use colon format e.g. 4029:12345",
                i
            ));
        }
        let output = obj.get("outputPath").and_then(|v| v.as_str()).unwrap_or("");
        if output.is_empty() {
            return Some(format!("items[{}].outputPath is required", i));
        }
    }
    None
}

fn validate_set_reactions(node_ids: &[String], params: &Map<String, Value>) -> Option<String> {
    if let Some(m) = require_single_node_id(node_ids) {
        return Some(m);
    }
    let reactions = match params.get("reactions") {
        Some(v) => v,
        None => return Some("reactions is required".into()),
    };
    let arr = match reactions.as_array() {
        Some(a) => a,
        None => return Some("reactions must be an array".into()),
    };
    if let Some(mode) = str_param(params, "mode") {
        if !mode.is_empty() && mode != "replace" && mode != "append" {
            return Some(format!("mode must be 'replace' or 'append', got: {}", mode));
        }
    }
    for (i, raw) in arr.iter().enumerate() {
        let obj = match raw.as_object() {
            Some(o) => o,
            None => return Some(format!("reactions[{}] must be an object", i)),
        };
        if let Some(trigger) = obj.get("trigger").and_then(|v| v.as_object()) {
            let t = trigger.get("type").and_then(|v| v.as_str()).unwrap_or("");
            if !t.is_empty() && !is_valid_trigger_type(t) {
                return Some(format!("reactions[{}].trigger.type is invalid: {}", i, t));
            }
            if t == "AFTER_TIMEOUT" && !trigger.get("timeout").is_some_and(|v| v.is_number()) {
                return Some(format!(
                    "reactions[{}].trigger.timeout is required for AFTER_TIMEOUT and must be a number (milliseconds)",
                    i
                ));
            }
        }
        if let Some(action) = obj.get("action").and_then(|v| v.as_object()) {
            let t = action.get("type").and_then(|v| v.as_str()).unwrap_or("");
            if !t.is_empty() && !is_valid_action_type(t) {
                return Some(format!("reactions[{}].action.type is invalid: {}", i, t));
            }
            match t {
                "NODE" => {
                    let nav = action
                        .get("navigation")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if nav.is_empty() {
                        return Some(format!(
                            "reactions[{}].action.navigation is required for NODE (e.g. NAVIGATE, OVERLAY, SCROLL_TO, SWAP, CHANGE_TO)",
                            i
                        ));
                    }
                }
                "URL" => {
                    let url = action.get("url").and_then(|v| v.as_str()).unwrap_or("");
                    if url.is_empty() {
                        return Some(format!("reactions[{}].action.url is required for URL", i));
                    }
                }
                _ => {}
            }
        }
    }
    None
}

fn validate_auto_layout(params: &Map<String, Value>) -> Option<String> {
    let cases: &[(&str, &[&str], &str)] = &[
        (
            "layoutMode",
            &["HORIZONTAL", "VERTICAL", "NONE"],
            "HORIZONTAL, VERTICAL, or NONE",
        ),
        (
            "primaryAxisAlignItems",
            &["MIN", "CENTER", "MAX", "SPACE_BETWEEN"],
            "MIN, CENTER, MAX, or SPACE_BETWEEN",
        ),
        (
            "counterAxisAlignItems",
            &["MIN", "CENTER", "MAX", "BASELINE"],
            "MIN, CENTER, MAX, or BASELINE",
        ),
        ("primaryAxisSizingMode", &["FIXED", "AUTO"], "FIXED or AUTO"),
        ("counterAxisSizingMode", &["FIXED", "AUTO"], "FIXED or AUTO"),
        ("layoutWrap", &["NO_WRAP", "WRAP"], "NO_WRAP or WRAP"),
    ];
    for (key, allowed, label) in cases {
        if let Some(v) = str_param(params, key) {
            if !v.is_empty() && !allowed.contains(&v) {
                return Some(format!("{} must be {}, got: {}", key, label, v));
            }
        }
    }
    None
}

fn is_valid_export_format(f: &str) -> bool {
    matches!(f, "PNG" | "SVG" | "JPG" | "PDF")
}

fn is_valid_blend_mode(m: &str) -> bool {
    matches!(
        m,
        "NORMAL"
            | "MULTIPLY"
            | "SCREEN"
            | "OVERLAY"
            | "DARKEN"
            | "LIGHTEN"
            | "COLOR_DODGE"
            | "COLOR_BURN"
            | "HARD_LIGHT"
            | "SOFT_LIGHT"
            | "DIFFERENCE"
            | "EXCLUSION"
            | "HUE"
            | "SATURATION"
            | "COLOR"
            | "LUMINOSITY"
            | "PASS_THROUGH"
    )
}

fn is_valid_trigger_type(t: &str) -> bool {
    matches!(
        t,
        "ON_CLICK"
            | "ON_HOVER"
            | "ON_PRESS"
            | "ON_DRAG"
            | "AFTER_TIMEOUT"
            | "MOUSE_ENTER"
            | "MOUSE_LEAVE"
            | "MOUSE_UP"
            | "MOUSE_DOWN"
    )
}

fn is_valid_action_type(t: &str) -> bool {
    matches!(
        t,
        "NODE"
            | "BACK"
            | "CLOSE"
            | "URL"
            | "CONDITIONAL"
            | "SET_VARIABLE"
            | "SET_VARIABLE_MODE"
            | "UPDATE_MEDIA_RUNTIME"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn obj(v: serde_json::Value) -> Map<String, Value> {
        match v {
            serde_json::Value::Object(m) => m,
            _ => panic!("not object"),
        }
    }

    #[test]
    fn valid_node_ids() {
        for id in [
            "4029:12345",
            "0:1",
            "1:1",
            "I44:9;44:3",
            "I2167:9091;186:1579;186:1745",
        ] {
            assert!(valid_node_id(id), "{id} should be valid");
        }
    }

    #[test]
    fn invalid_node_ids() {
        for id in [
            "",
            "4029-12345",
            "4029:12345:6789",
            "abc:def",
            "4029:",
            ":12345",
            "4029",
        ] {
            assert!(!valid_node_id(id), "{id} should be invalid");
        }
    }

    #[test]
    fn normalize_hyphen_to_colon() {
        assert_eq!(normalize_node_id("4029-12345"), "4029:12345");
        assert_eq!(normalize_node_id("4029:12345"), "4029:12345");
        assert_eq!(normalize_node_id("not-a-node-id"), "not-a-node-id");
        assert_eq!(normalize_node_id(""), "");
    }

    #[test]
    fn get_node_validation() {
        assert!(validate_rpc("get_node", &[], &Map::new()).is_some());
        assert!(validate_rpc("get_node", &["4029-12345".into()], &Map::new()).is_some());
        assert!(validate_rpc("get_node", &["4029:12345".into()], &Map::new()).is_none());
    }

    #[test]
    fn set_opacity_bounds() {
        assert!(validate_rpc(
            "set_opacity",
            &["1:1".into()],
            &obj(json!({"opacity": 1.5}))
        )
        .is_some());
        for v in [0.0, 0.5, 1.0] {
            assert!(
                validate_rpc("set_opacity", &["1:1".into()], &obj(json!({"opacity": v}))).is_none(),
                "opacity {v} should be valid"
            );
        }
    }

    #[test]
    fn group_nodes_requires_two() {
        assert!(validate_rpc("group_nodes", &["1:1".into()], &Map::new()).is_some());
        assert!(validate_rpc("group_nodes", &["1:1".into(), "2:2".into()], &Map::new()).is_none());
    }

    #[test]
    fn unknown_tool_passes() {
        assert!(validate_rpc("totally_made_up_tool", &[], &Map::new()).is_none());
    }

    #[test]
    fn set_reactions_after_timeout_requires_number() {
        let r = validate_rpc(
            "set_reactions",
            &["1:2".into()],
            &obj(json!({
                "reactions": [{
                    "trigger": {"type": "AFTER_TIMEOUT"},
                    "action": {"type": "BACK"}
                }]
            })),
        );
        assert!(r.is_some());
    }
}
