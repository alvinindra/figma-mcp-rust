//! Sanity tests on the tool registry.

use figma_mcp_rust::tools;
use std::collections::HashSet;

#[test]
fn registers_at_least_73_tools() {
    let count = tools::all().len();
    assert!(count >= 73, "expected at least 73 tools, got {count}");
}

#[test]
fn tool_names_are_unique() {
    let mut seen = HashSet::new();
    for def in tools::all() {
        assert!(seen.insert(def.name), "duplicate tool name: {}", def.name);
    }
}

#[test]
fn schema_is_object_with_type() {
    for def in tools::all() {
        let schema = (def.input_schema)();
        let obj = schema
            .as_object()
            .unwrap_or_else(|| panic!("tool {} schema must be an object", def.name));
        let ty = obj.get("type").and_then(|v| v.as_str()).unwrap_or("");
        assert_eq!(
            ty, "object",
            "tool {} schema.type must be 'object'",
            def.name
        );
        assert!(
            obj.contains_key("properties"),
            "tool {} schema must have 'properties'",
            def.name
        );
    }
}

#[test]
fn lookup_by_name_matches_position() {
    for def in tools::all() {
        let found = tools::find(def.name).expect("found");
        assert_eq!(found.name, def.name);
    }
    assert!(tools::find("does_not_exist").is_none());
}

#[test]
fn dedupe_components_reaches_plugin_as_camel_case() {
    use serde_json::{json, Value};
    let mut params = json!({ "depth": 6, "detail": "full", "dedupe_components": true })
        .as_object()
        .cloned()
        .unwrap();
    tools::to_plugin_params("get_design_context", &mut params);
    assert_eq!(params.get("dedupeComponents"), Some(&Value::Bool(true)));
    assert!(!params.contains_key("dedupe_components"));
    assert_eq!(params.get("depth"), Some(&json!(6)));

    // The documented spelling wins over the undocumented one.
    let mut both = json!({ "dedupe_components": true, "dedupeComponents": false })
        .as_object()
        .cloned()
        .unwrap();
    tools::to_plugin_params("get_design_context", &mut both);
    assert_eq!(both.get("dedupeComponents"), Some(&Value::Bool(true)));
}

/// The plugin reads camelCase params, so any snake_case schema property must be
/// renamed on the way out or the plugin silently ignores it (issue #3).
#[test]
fn snake_case_schema_properties_have_plugin_aliases() {
    for def in tools::all() {
        let schema = (def.input_schema)();
        let Some(props) = schema.get("properties").and_then(|p| p.as_object()) else {
            continue;
        };
        for key in props.keys().filter(|k| k.contains('_')) {
            assert!(
                tools::PARAM_ALIASES
                    .iter()
                    .any(|&(t, s, _)| t == def.name && s == key),
                "tool {} property {key} is snake_case but has no entry in PARAM_ALIASES",
                def.name
            );
        }
    }
}
