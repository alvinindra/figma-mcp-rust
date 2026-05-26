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
