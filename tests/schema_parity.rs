//! Parity tests vs the Go `internal/schema_test.go` — ported case-by-case to
//! ensure the Rust validator produces equivalent results.

use figma_mcp_rust::schema::{normalize_node_id, valid_node_id, validate_rpc};
use serde_json::{json, Map, Value};

fn obj(v: Value) -> Map<String, Value> {
    v.as_object().cloned().unwrap_or_default()
}

#[test]
fn valid_node_id_examples() {
    for id in [
        "4029:12345",
        "0:1",
        "1:1",
        "I44:9;44:3",
        "I2167:9091;186:1579;186:1745",
    ] {
        assert!(valid_node_id(id), "{id} should be valid");
    }
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
fn normalize_node_id_examples() {
    assert_eq!(normalize_node_id("4029-12345"), "4029:12345");
    assert_eq!(normalize_node_id("4029:12345"), "4029:12345");
    assert_eq!(normalize_node_id("not-a-node-id"), "not-a-node-id");
    assert_eq!(normalize_node_id(""), "");
}

#[test]
fn get_node() {
    assert!(validate_rpc("get_node", &[], &Map::new()).is_some());
    assert!(validate_rpc("get_node", &["4029-12345".into()], &Map::new()).is_some());
    assert!(validate_rpc("get_node", &["4029:12345".into()], &Map::new()).is_none());
}

#[test]
fn get_nodes_info_requires_non_empty() {
    assert!(validate_rpc("get_nodes_info", &[], &Map::new()).is_some());
    assert!(validate_rpc("get_nodes_info", &["bad".into()], &Map::new()).is_some());
    assert!(validate_rpc("get_nodes_info", &["1:1".into(), "2:2".into()], &Map::new()).is_none());
}

#[test]
fn get_screenshot_format() {
    assert!(validate_rpc(
        "get_screenshot",
        &["1:1".into()],
        &obj(json!({"format": "GIF"}))
    )
    .is_some());
    for f in ["PNG", "SVG", "JPG", "PDF"] {
        assert!(validate_rpc(
            "get_screenshot",
            &["1:1".into()],
            &obj(json!({ "format": f }))
        )
        .is_none());
    }
}

#[test]
fn save_screenshots_items() {
    assert!(validate_rpc("save_screenshots", &[], &Map::new()).is_some());
    assert!(validate_rpc("save_screenshots", &[], &obj(json!({ "items": [] }))).is_some());
    assert!(validate_rpc(
        "save_screenshots",
        &[],
        &obj(json!({ "items": [{ "nodeId": "bad", "outputPath": "out.png" }] }))
    )
    .is_some());
    assert!(validate_rpc(
        "save_screenshots",
        &[],
        &obj(json!({ "items": [{ "nodeId": "1:1" }] }))
    )
    .is_some());
    assert!(validate_rpc(
        "save_screenshots",
        &[],
        &obj(json!({ "items": [{ "nodeId": "1:1", "outputPath": "out.png" }] }))
    )
    .is_none());
}

#[test]
fn get_design_context_bounds() {
    assert!(validate_rpc("get_design_context", &[], &obj(json!({ "depth": -1.0 }))).is_some());
    assert!(validate_rpc("get_design_context", &[], &obj(json!({ "detail": "huge" }))).is_some());
    for d in ["minimal", "compact", "full"] {
        assert!(validate_rpc("get_design_context", &[], &obj(json!({ "detail": d }))).is_none());
    }
}

#[test]
fn set_opacity_bounds() {
    assert!(validate_rpc(
        "set_opacity",
        &["1:1".into()],
        &obj(json!({ "opacity": 1.5 }))
    )
    .is_some());
    assert!(validate_rpc(
        "set_opacity",
        &["1:1".into()],
        &obj(json!({ "opacity": -0.1 }))
    )
    .is_some());
    for v in [0.0, 0.5, 1.0] {
        assert!(
            validate_rpc(
                "set_opacity",
                &["1:1".into()],
                &obj(json!({ "opacity": v }))
            )
            .is_none(),
            "{v} should be valid"
        );
    }
}

#[test]
fn create_variable_types() {
    assert!(validate_rpc(
        "create_variable",
        &[],
        &obj(json!({"name": "x", "collectionId": "c1", "type": "NUMBER"}))
    )
    .is_some());
    for t in ["COLOR", "FLOAT", "STRING", "BOOLEAN"] {
        assert!(validate_rpc(
            "create_variable",
            &[],
            &obj(json!({"name": "x", "collectionId": "c1", "type": t}))
        )
        .is_none());
    }
}

#[test]
fn swap_component_id_format() {
    assert!(validate_rpc(
        "swap_component",
        &["1:1".into()],
        &obj(json!({ "componentId": "bad-format" }))
    )
    .is_some());
    assert!(validate_rpc(
        "swap_component",
        &["1:1".into()],
        &obj(json!({ "componentId": "2:2" }))
    )
    .is_none());
}

#[test]
fn unknown_tool_passes_through() {
    assert!(validate_rpc("totally_made_up_tool", &[], &Map::new()).is_none());
}

#[test]
fn set_reactions_after_timeout_requires_timeout() {
    assert!(validate_rpc(
        "set_reactions",
        &["1:2".into()],
        &obj(json!({
            "reactions": [{
                "trigger": {"type": "AFTER_TIMEOUT"},
                "action": {"type": "BACK"}
            }]
        }))
    )
    .is_some());

    assert!(validate_rpc(
        "set_reactions",
        &["1:2".into()],
        &obj(json!({
            "reactions": [{
                "trigger": {"type": "AFTER_TIMEOUT", "timeout": 3000.0},
                "action": {"type": "BACK"}
            }]
        }))
    )
    .is_none());
}

#[test]
fn group_nodes_minimum() {
    assert!(validate_rpc("group_nodes", &[], &Map::new()).is_some());
    assert!(validate_rpc("group_nodes", &["1:1".into()], &Map::new()).is_some());
    assert!(validate_rpc("group_nodes", &["1:1".into(), "2:2".into()], &Map::new()).is_none());
}

#[test]
fn reorder_nodes_order_values() {
    assert!(validate_rpc(
        "reorder_nodes",
        &["1:1".into()],
        &obj(json!({ "order": "up" }))
    )
    .is_some());
    for order in ["bringToFront", "sendToBack", "bringForward", "sendBackward"] {
        assert!(validate_rpc(
            "reorder_nodes",
            &["1:1".into()],
            &obj(json!({ "order": order }))
        )
        .is_none());
    }
}

#[test]
fn batch_rename_requires_some_op() {
    assert!(validate_rpc("batch_rename_nodes", &["1:1".into()], &Map::new()).is_some());
    assert!(validate_rpc(
        "batch_rename_nodes",
        &["1:1".into()],
        &obj(json!({ "find": "Btn" }))
    )
    .is_some());
    assert!(validate_rpc(
        "batch_rename_nodes",
        &["1:1".into()],
        &obj(json!({ "prefix": "UI/" }))
    )
    .is_none());
}

#[test]
fn auto_layout_invalid_values() {
    for (k, v) in [
        ("primaryAxisAlignItems", "LEFT"),
        ("counterAxisAlignItems", "TOP"),
        ("primaryAxisSizingMode", "SHRINK"),
        ("layoutWrap", "FLEX_WRAP"),
    ] {
        assert!(
            validate_rpc("create_frame", &[], &obj(json!({ k: v }))).is_some(),
            "{k}={v} should be invalid"
        );
    }
}

#[test]
fn export_tokens_format() {
    for f in ["json", "css"] {
        assert!(validate_rpc("export_tokens", &[], &obj(json!({ "format": f }))).is_none());
    }
    assert!(validate_rpc("export_tokens", &[], &obj(json!({ "format": "yaml" }))).is_some());
}

#[test]
fn set_effects_array_required() {
    assert!(validate_rpc("set_effects", &["1:1".into()], &Map::new()).is_some());
    assert!(validate_rpc(
        "set_effects",
        &["1:1".into()],
        &obj(json!({"effects": "shadow"}))
    )
    .is_some());
    assert!(validate_rpc(
        "set_effects",
        &["1:1".into()],
        &obj(json!({"effects": [{"type": "GLOW"}]}))
    )
    .is_some());
    assert!(validate_rpc(
        "set_effects",
        &["1:1".into()],
        &obj(json!({"effects": [{"type": "DROP_SHADOW"}]}))
    )
    .is_none());
}
