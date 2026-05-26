//! All 73 tool definitions in one declarative table.
//!
//! Each entry pairs an MCP tool description + JSON Schema with a `NodeIds`
//! rule that determines how to split call args into `(nodeIDs, params)` before
//! handing the request to the Bridge.

use serde_json::{json, Value};

use super::{special, NodeIds, ToolDef};

pub static TOOLS: &[ToolDef] = &[
    // ── Read — Document & Selection ─────────────────────────────────────────
    ToolDef {
        name: "get_document",
        description: "Get the full node tree of the current page (not the whole file — only the active page). Returns all nodes recursively and can be very large. Prefer get_design_context for exploration or when token efficiency matters.",
        input_schema: empty_schema,
        node_ids: NodeIds::None,
        special: None,
    },
    ToolDef {
        name: "get_pages",
        description: "List all pages in the document with their IDs and names. Lightweight alternative to get_document.",
        input_schema: empty_schema,
        node_ids: NodeIds::None,
        special: None,
    },
    ToolDef {
        name: "get_metadata",
        description: "Get metadata about the current Figma document: file name, pages, current page.",
        input_schema: empty_schema,
        node_ids: NodeIds::None,
        special: None,
    },
    ToolDef {
        name: "get_selection",
        description: "Get the nodes currently selected in Figma. Returns an empty array if nothing is selected. Use get_design_context or get_node to retrieve deeper detail about a specific node by ID.",
        input_schema: empty_schema,
        node_ids: NodeIds::None,
        special: None,
    },
    ToolDef {
        name: "get_node",
        description: "Get a single node by ID with full detail. Use get_nodes_info to fetch multiple nodes in one round-trip instead of calling this repeatedly. Node ID must be colon format e.g. '4029:12345', never hyphens.",
        input_schema: get_node_schema,
        node_ids: NodeIds::SingleField,
        special: None,
    },
    ToolDef {
        name: "get_nodes_info",
        description: "Get full details for multiple nodes by ID in one round-trip. Prefer this over calling get_node repeatedly when you need several nodes.",
        input_schema: get_nodes_info_schema,
        node_ids: NodeIds::Multi,
        special: None,
    },
    ToolDef {
        name: "get_design_context",
        description: "Get a depth-limited, token-efficient tree of the current selection or page. Use this instead of get_document when exploring large files. Supports detail levels (minimal/compact/full) and dedupe_components for pages heavy with repeated component instances.",
        input_schema: get_design_context_schema,
        node_ids: NodeIds::None,
        special: None,
    },
    ToolDef {
        name: "search_nodes",
        description: "Search for nodes by name substring and/or type within a subtree. Use this when you know (part of) the node name. Use scan_nodes_by_types when you want all nodes of a type regardless of name.",
        input_schema: search_nodes_schema,
        node_ids: NodeIds::None,
        special: None,
    },
    ToolDef {
        name: "scan_text_nodes",
        description: "Scan all TEXT nodes in a subtree and return their content. Shorthand for scan_nodes_by_types with ['TEXT'] — use when you only need text copy from a component or frame.",
        input_schema: scan_text_nodes_schema,
        node_ids: NodeIds::None,
        special: None,
    },
    ToolDef {
        name: "scan_nodes_by_types",
        description: "Find all nodes of specific types in a subtree, regardless of name. Use search_nodes instead when you need to filter by name.",
        input_schema: scan_nodes_by_types_schema,
        node_ids: NodeIds::None,
        special: None,
    },
    ToolDef {
        name: "get_reactions",
        description: "Get the prototype reactions defined on a node. Returns an array of reaction objects — each has a trigger (e.g. ON_CLICK, ON_HOVER, AFTER_TIMEOUT) and an actions array (navigate to node, open URL, go back, etc.). Use set_reactions to add or replace reactions, remove_reactions to delete them.",
        input_schema: get_node_schema,
        node_ids: NodeIds::SingleField,
        special: None,
    },
    ToolDef {
        name: "get_viewport",
        description: "Get the current Figma viewport: scroll center, zoom level, and visible bounds.",
        input_schema: empty_schema,
        node_ids: NodeIds::None,
        special: None,
    },
    ToolDef {
        name: "get_fonts",
        description: "List all fonts used in the current page, sorted by usage frequency. Useful for understanding typography without scanning all text nodes.",
        input_schema: empty_schema,
        node_ids: NodeIds::None,
        special: None,
    },

    // ── Read — Styles & Variables ────────────────────────────────────────────
    ToolDef {
        name: "get_styles",
        description: "Get all local styles in the document (paint, text, effect, and grid). Returns each style's ID, name, type, and properties. Use the style ID with apply_style_to_node or update_paint_style. For design tokens (variables), use get_variable_defs instead.",
        input_schema: empty_schema,
        node_ids: NodeIds::None,
        special: None,
    },
    ToolDef {
        name: "get_variable_defs",
        description: "Get all local variable definitions: collections, modes, and values. Variables are Figma's design token system.",
        input_schema: empty_schema,
        node_ids: NodeIds::None,
        special: None,
    },
    ToolDef {
        name: "get_local_components",
        description: "Get all components defined in the current Figma file.",
        input_schema: empty_schema,
        node_ids: NodeIds::None,
        special: None,
    },
    ToolDef {
        name: "get_annotations",
        description: "Get dev-mode annotations in the current document or scoped to a specific node. Returns annotation objects with label text, measurement type, and the ID of the annotated node. Omit nodeId to retrieve all annotations on the current page.",
        input_schema: get_annotations_schema,
        node_ids: NodeIds::None,
        special: None,
    },
    ToolDef {
        name: "export_tokens",
        description: "Export all design tokens (variables and paint styles) as JSON or CSS custom properties. Ideal for bridging Figma variables into your codebase.",
        input_schema: export_tokens_schema,
        node_ids: NodeIds::None,
        special: None,
    },

    // ── Read — Export ───────────────────────────────────────────────────────
    ToolDef {
        name: "get_screenshot",
        description: "Export a screenshot of one or more nodes as base64-encoded image data (held in memory). Use save_screenshots instead when you want to write images directly to disk without base64 in the response.",
        input_schema: get_screenshot_schema,
        node_ids: NodeIds::Multi,
        special: None,
    },
    ToolDef {
        name: "export_frames_to_pdf",
        description: "Export multiple frames as a single multi-page PDF file. Each frame becomes one page in order. Ideal for pitch decks, proposals, and slide exports.",
        input_schema: export_frames_to_pdf_schema,
        node_ids: NodeIds::Multi,
        special: Some(special::export_frames_to_pdf),
    },
    ToolDef {
        name: "save_screenshots",
        description: "Export screenshots for multiple nodes and write them to the local filesystem. Returns file metadata (path, size, dimensions) — no base64 in the response. Use get_screenshot instead when you need the image data in memory.",
        input_schema: save_screenshots_schema,
        node_ids: NodeIds::None,
        special: Some(special::save_screenshots),
    },

    // ── Write — Create ──────────────────────────────────────────────────────
    ToolDef {
        name: "create_frame",
        description: "Create a new frame on the current page or inside a parent node.",
        input_schema: create_frame_schema,
        node_ids: NodeIds::None,
        special: None,
    },
    ToolDef {
        name: "create_rectangle",
        description: "Create a new rectangle on the current page or inside a parent node.",
        input_schema: create_rectangle_schema,
        node_ids: NodeIds::None,
        special: None,
    },
    ToolDef {
        name: "create_ellipse",
        description: "Create a new ellipse (circle/oval) on the current page or inside a parent node.",
        input_schema: create_ellipse_schema,
        node_ids: NodeIds::None,
        special: None,
    },
    ToolDef {
        name: "create_text",
        description: "Create a new text node on the current page or inside a parent node. The font is loaded automatically before insertion. Returns the created node ID and bounds. Use set_text to update the content of an existing text node.",
        input_schema: create_text_schema,
        node_ids: NodeIds::None,
        special: None,
    },
    ToolDef {
        name: "import_image",
        description: "Import a base64-encoded image into Figma as a rectangle with an image fill. Use get_screenshot to capture images or provide your own base64 PNG/JPG.",
        input_schema: import_image_schema,
        node_ids: NodeIds::None,
        special: None,
    },
    ToolDef {
        name: "create_component",
        description: "Convert an existing FRAME node into a reusable COMPONENT. The frame is replaced in place by the new component.",
        input_schema: create_component_schema,
        node_ids: NodeIds::SingleField,
        special: None,
    },
    ToolDef {
        name: "create_section",
        description: "Create a Figma Section node on the current page. Sections are the modern way to organize frames and groups on a page.",
        input_schema: create_section_schema,
        node_ids: NodeIds::None,
        special: None,
    },

    // ── Write — Modify ─────────────────────────────────────────────────────
    ToolDef {
        name: "set_text",
        description: "Update the text content of an existing TEXT node.",
        input_schema: set_text_schema,
        node_ids: NodeIds::SingleField,
        special: None,
    },
    ToolDef {
        name: "set_fills",
        description: "Set the fill color on a single node (takes one nodeId, not an array). Use mode='append' to stack a new fill on top of existing fills instead of replacing them.",
        input_schema: set_fills_schema,
        node_ids: NodeIds::SingleField,
        special: None,
    },
    ToolDef {
        name: "set_strokes",
        description: "Set the stroke color and weight on a single node (takes one nodeId, not an array). Use mode='append' to stack a new stroke on top of existing strokes instead of replacing them.",
        input_schema: set_strokes_schema,
        node_ids: NodeIds::SingleField,
        special: None,
    },
    ToolDef {
        name: "move_nodes",
        description: "Move one or more nodes to an absolute canvas position. The same x/y is applied to every node independently (not a relative offset from current position).",
        input_schema: move_nodes_schema,
        node_ids: NodeIds::Multi,
        special: None,
    },
    ToolDef {
        name: "resize_nodes",
        description: "Resize one or more nodes. The same width/height is applied to every node in the list independently. Provide width, height, or both.",
        input_schema: resize_nodes_schema,
        node_ids: NodeIds::Multi,
        special: None,
    },
    ToolDef {
        name: "rename_node",
        description: "Rename a single node by ID. Returns the updated node with its new name. Use batch_rename_nodes to rename multiple nodes at once or to apply find/replace patterns across many nodes.",
        input_schema: rename_node_schema,
        node_ids: NodeIds::SingleField,
        special: None,
    },
    ToolDef {
        name: "clone_node",
        description: "Clone an existing node, optionally repositioning it or placing it in a new parent.",
        input_schema: clone_node_schema,
        node_ids: NodeIds::SingleField,
        special: None,
    },
    ToolDef {
        name: "set_opacity",
        description: "Set the opacity of one or more nodes (0 = fully transparent, 1 = fully opaque).",
        input_schema: set_opacity_schema,
        node_ids: NodeIds::Multi,
        special: None,
    },
    ToolDef {
        name: "set_corner_radius",
        description: "Set corner radius on one or more nodes. Provide a uniform cornerRadius or individual per-corner values.",
        input_schema: set_corner_radius_schema,
        node_ids: NodeIds::Multi,
        special: None,
    },
    ToolDef {
        name: "set_auto_layout",
        description: "Set or update auto-layout (flex) properties on an existing frame.",
        input_schema: set_auto_layout_schema,
        node_ids: NodeIds::SingleField,
        special: None,
    },
    ToolDef {
        name: "delete_nodes",
        description: "Delete one or more nodes. This cannot be undone via MCP — use with care.",
        input_schema: node_ids_array_schema,
        node_ids: NodeIds::Multi,
        special: None,
    },
    ToolDef {
        name: "set_visible",
        description: "Show or hide one or more nodes by setting their visibility.",
        input_schema: set_visible_schema,
        node_ids: NodeIds::Multi,
        special: None,
    },
    ToolDef {
        name: "lock_nodes",
        description: "Lock one or more nodes to prevent accidental edits in Figma.",
        input_schema: node_ids_array_schema,
        node_ids: NodeIds::Multi,
        special: None,
    },
    ToolDef {
        name: "unlock_nodes",
        description: "Unlock one or more nodes, allowing them to be edited again.",
        input_schema: node_ids_array_schema,
        node_ids: NodeIds::Multi,
        special: None,
    },
    ToolDef {
        name: "rotate_nodes",
        description: "Rotate one or more nodes to an absolute angle in degrees.",
        input_schema: rotate_nodes_schema,
        node_ids: NodeIds::Multi,
        special: None,
    },
    ToolDef {
        name: "reorder_nodes",
        description: "Change the z-order (layer stack position) of one or more nodes.",
        input_schema: reorder_nodes_schema,
        node_ids: NodeIds::Multi,
        special: None,
    },
    ToolDef {
        name: "set_blend_mode",
        description: "Set the blend mode of one or more nodes (e.g. MULTIPLY, SCREEN, OVERLAY).",
        input_schema: set_blend_mode_schema,
        node_ids: NodeIds::Multi,
        special: None,
    },
    ToolDef {
        name: "set_constraints",
        description: "Set layout constraints (pinning behaviour) on one or more nodes relative to their parent.",
        input_schema: set_constraints_schema,
        node_ids: NodeIds::Multi,
        special: None,
    },
    ToolDef {
        name: "reparent_nodes",
        description: "Move one or more nodes to a different parent frame, group, or section.",
        input_schema: reparent_nodes_schema,
        node_ids: NodeIds::Multi,
        special: None,
    },
    ToolDef {
        name: "batch_rename_nodes",
        description: "Rename multiple nodes using find/replace, regex substitution, or prefix/suffix addition.",
        input_schema: batch_rename_nodes_schema,
        node_ids: NodeIds::Multi,
        special: None,
    },
    ToolDef {
        name: "find_replace_text",
        description: "Find and replace text content across all TEXT nodes in a subtree. Searches the entire current page if no nodeId is given.",
        input_schema: find_replace_text_schema,
        node_ids: NodeIds::SingleOptional,
        special: None,
    },

    // ── Write — Styles ────────────────────────────────────────────────────
    ToolDef {
        name: "create_paint_style",
        description: "Create a new local paint style with a solid fill color.",
        input_schema: create_paint_style_schema,
        node_ids: NodeIds::None,
        special: None,
    },
    ToolDef {
        name: "create_text_style",
        description: "Create a new local text style (typography preset). Returns the new style's ID. Apply it to nodes with apply_style_to_node. Use get_styles to list existing text styles.",
        input_schema: create_text_style_schema,
        node_ids: NodeIds::None,
        special: None,
    },
    ToolDef {
        name: "create_effect_style",
        description: "Create a new local effect style (drop shadow, inner shadow, or blur).",
        input_schema: create_effect_style_schema,
        node_ids: NodeIds::None,
        special: None,
    },
    ToolDef {
        name: "create_grid_style",
        description: "Create a new local layout grid style.",
        input_schema: create_grid_style_schema,
        node_ids: NodeIds::None,
        special: None,
    },
    ToolDef {
        name: "update_paint_style",
        description: "Update an existing paint style's name, color, or description. Only paint styles support in-place updates — to modify text, effect, or grid styles, use delete_style and recreate them.",
        input_schema: update_paint_style_schema,
        node_ids: NodeIds::None,
        special: None,
    },
    ToolDef {
        name: "delete_style",
        description: "Delete a style (paint, text, effect, or grid) by its ID.",
        input_schema: delete_style_schema,
        node_ids: NodeIds::None,
        special: None,
    },
    ToolDef {
        name: "apply_style_to_node",
        description: "Apply an existing local style (paint, text, effect, or grid) to a node, linking the node to that style.",
        input_schema: apply_style_to_node_schema,
        node_ids: NodeIds::SingleField,
        special: None,
    },
    ToolDef {
        name: "set_effects",
        description: "Apply one or more effects (drop shadow, inner shadow, layer blur, background blur) directly to a node. Replaces all existing effects. Pass an empty array to clear all effects.",
        input_schema: set_effects_schema,
        node_ids: NodeIds::SingleField,
        special: None,
    },
    ToolDef {
        name: "bind_variable_to_node",
        description: "Bind a local variable to a node property so the property is driven by the variable's value. COLOR variables: use fillColor or strokeColor. BOOLEAN variables: use visible. FLOAT variables: use opacity, rotation, width, height, cornerRadius, topLeftRadius, topRightRadius, bottomLeftRadius, bottomRightRadius, strokeWeight, itemSpacing, paddingTop, paddingRight, paddingBottom, paddingLeft.",
        input_schema: bind_variable_to_node_schema,
        node_ids: NodeIds::SingleField,
        special: None,
    },

    // ── Write — Variables ─────────────────────────────────────────────────
    ToolDef {
        name: "create_variable_collection",
        description: "Create a new local variable collection with an optional initial mode name. NOTE — Figma free plan limits each collection to 1 mode. If you need Light/Dark (or any multi-mode) theming and the user is on the free plan, do NOT try to call add_variable_mode; instead use the name-prefix workaround: create all variables in a single collection and prefix each variable name with its mode, e.g. 'light/color-bg' and 'dark/color-bg'. Inform the user of this limitation.",
        input_schema: create_variable_collection_schema,
        node_ids: NodeIds::None,
        special: None,
    },
    ToolDef {
        name: "add_variable_mode",
        description: "Add a new mode to an existing variable collection (e.g. Light/Dark, Desktop/Mobile). IMPORTANT — Figma free plan only allows 1 mode per collection; calling this tool on a free-plan account will return the error 'Limited to 1 modes only'. If that error occurs, stop retrying and switch to the name-prefix workaround: keep the single default mode and create variables prefixed by mode, e.g. 'light/color-bg' and 'dark/color-bg' in the same collection. Tell the user that native multi-mode variables require a paid Figma plan (Professional or above).",
        input_schema: add_variable_mode_schema,
        node_ids: NodeIds::None,
        special: None,
    },
    ToolDef {
        name: "create_variable",
        description: "Create a new variable (design token) inside an existing collection. Returns the new variable's ID. Use get_variable_defs to find collection IDs, set_variable_value to set values per mode, and bind_variable_to_node to apply the variable to a node property.",
        input_schema: create_variable_schema,
        node_ids: NodeIds::None,
        special: None,
    },
    ToolDef {
        name: "set_variable_value",
        description: "Set a variable's value for a specific mode.",
        input_schema: set_variable_value_schema,
        node_ids: NodeIds::None,
        special: None,
    },
    ToolDef {
        name: "delete_variable",
        description: "Delete a single variable (provide variableId) or an entire collection and all its variables (provide collectionId). Provide exactly one of the two — not both.",
        input_schema: delete_variable_schema,
        node_ids: NodeIds::None,
        special: None,
    },

    // ── Write — Pages ─────────────────────────────────────────────────────
    ToolDef {
        name: "add_page",
        description: "Add a new page to the Figma document.",
        input_schema: add_page_schema,
        node_ids: NodeIds::None,
        special: None,
    },
    ToolDef {
        name: "delete_page",
        description: "Delete a page from the Figma document. Cannot delete the only remaining page.",
        input_schema: delete_page_schema,
        node_ids: NodeIds::None,
        special: None,
    },
    ToolDef {
        name: "rename_page",
        description: "Rename an existing page in the Figma document.",
        input_schema: rename_page_schema,
        node_ids: NodeIds::None,
        special: None,
    },

    // ── Write — Components & Navigation ────────────────────────────────────
    ToolDef {
        name: "navigate_to_page",
        description: "Switch the active Figma page. Provide either pageId or pageName.",
        input_schema: navigate_to_page_schema,
        node_ids: NodeIds::None,
        special: None,
    },
    ToolDef {
        name: "group_nodes",
        description: "Group two or more nodes into a GROUP. All nodes must share the same parent.",
        input_schema: group_nodes_schema,
        node_ids: NodeIds::Multi,
        special: None,
    },
    ToolDef {
        name: "ungroup_nodes",
        description: "Ungroup one or more GROUP nodes, moving their children to the parent and removing the group.",
        input_schema: node_ids_array_schema,
        node_ids: NodeIds::Multi,
        special: None,
    },
    ToolDef {
        name: "swap_component",
        description: "Swap the main component of an existing INSTANCE node, replacing it with a different component while keeping position and size.",
        input_schema: swap_component_schema,
        node_ids: NodeIds::SingleField,
        special: None,
    },
    ToolDef {
        name: "detach_instance",
        description: "Detach one or more component instances, converting them to plain frames. The link to the main component is broken; all visual properties are preserved.",
        input_schema: node_ids_array_schema,
        node_ids: NodeIds::Multi,
        special: None,
    },

    // ── Write — Prototype ──────────────────────────────────────────────────
    ToolDef {
        name: "set_reactions",
        description: "Set prototype reactions on a node. Use mode \"replace\" (default) to overwrite all reactions, or \"append\" to add to existing ones.\n\nSupported triggers: ON_CLICK, ON_HOVER, ON_PRESS, ON_DRAG, AFTER_TIMEOUT, MOUSE_ENTER, MOUSE_LEAVE, MOUSE_UP, MOUSE_DOWN\nSupported action types: NODE (navigation), BACK, CLOSE, URL\n  NODE navigation values: NAVIGATE, OVERLAY, SCROLL_TO, SWAP, CHANGE_TO\nTransition types: DISSOLVE, SMART_ANIMATE, MOVE_IN, MOVE_OUT, PUSH, SLIDE_IN, SLIDE_OUT",
        input_schema: set_reactions_schema,
        node_ids: NodeIds::SingleField,
        special: None,
    },
    ToolDef {
        name: "remove_reactions",
        description: "Remove prototype reactions from a node. Omit indices to remove all reactions. Provide a zero-based indices array to remove specific reactions (use get_reactions first to see current indices).",
        input_schema: remove_reactions_schema,
        node_ids: NodeIds::SingleField,
        special: None,
    },
];

// ── Schema builders ────────────────────────────────────────────────────────

fn empty_schema() -> Value {
    json!({ "type": "object", "properties": {} })
}

fn node_id_string(desc: &str) -> Value {
    json!({ "type": "string", "description": desc })
}

fn node_ids_array_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "nodeIds": {
                "type": "array",
                "items": { "type": "string" },
                "description": "Node IDs in colon format e.g. ['4029:12345']"
            }
        },
        "required": ["nodeIds"]
    })
}

fn get_node_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "nodeId": node_id_string("Node ID in colon format e.g. '4029:12345'")
        },
        "required": ["nodeId"]
    })
}

fn get_nodes_info_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "nodeIds": {
                "type": "array",
                "items": { "type": "string" },
                "description": "List of node IDs in colon format e.g. ['4029:12345', '4029:67890']"
            }
        },
        "required": ["nodeIds"]
    })
}

fn get_design_context_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "depth": { "type": "number", "description": "How many levels deep to traverse (default 2)" },
            "detail": { "type": "string", "description": "Property verbosity: minimal (id/name/type/bounds only), compact (+fills/strokes/opacity), full (everything, default)" },
            "dedupe_components": { "type": "boolean", "description": "When true, INSTANCE nodes are serialized compactly (mainComponentId + componentProperties + overrides) and unique component definitions are collected once in a top-level componentDefs map. Highly token-efficient for screens with many repeated component instances." }
        }
    })
}

fn search_nodes_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "query": { "type": "string", "description": "Name substring to match (case-insensitive)" },
            "nodeId": { "type": "string", "description": "Scope search to this subtree (default: current page), colon format e.g. '4029:12345'" },
            "types": { "type": "array", "items": { "type": "string" }, "description": "Filter by Figma node type e.g. ['TEXT', 'FRAME', 'COMPONENT']" },
            "limit": { "type": "number", "description": "Maximum results to return (default: 50)" }
        },
        "required": ["query"]
    })
}

fn scan_text_nodes_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "nodeId": node_id_string("Root node ID to scan from, colon format e.g. '4029:12345'")
        },
        "required": ["nodeId"]
    })
}

fn scan_nodes_by_types_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "nodeId": node_id_string("Root node ID to scan from, colon format e.g. '4029:12345'"),
            "types": { "type": "array", "items": { "type": "string" }, "description": "Node types to find e.g. ['FRAME', 'COMPONENT', 'INSTANCE']" }
        },
        "required": ["nodeId", "types"]
    })
}

fn get_annotations_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "nodeId": { "type": "string", "description": "Optional — scope results to annotations on this node and its descendants, colon format e.g. '4029:12345'" }
        }
    })
}

fn export_tokens_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "format": { "type": "string", "description": "Output format: json (default) or css" }
        }
    })
}

fn get_screenshot_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "nodeIds": { "type": "array", "items": { "type": "string" }, "description": "Optional node IDs to export, colon format. If empty, exports current selection." },
            "format": { "type": "string", "description": "Export format: PNG (default), SVG, JPG, or PDF" },
            "scale": { "type": "number", "description": "Export scale for raster formats (default 2)" }
        }
    })
}

fn export_frames_to_pdf_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "nodeIds": { "type": "array", "items": { "type": "string" }, "description": "Ordered list of frame node IDs to export as PDF pages, colon format e.g. '4029:12345'" },
            "outputPath": { "type": "string", "description": "File path to write the PDF to, must end in .pdf (relative to working directory or absolute)" }
        },
        "required": ["nodeIds", "outputPath"]
    })
}

fn save_screenshots_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "items": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "nodeId": { "type": "string", "description": "Node ID in colon format e.g. '4029:12345'" },
                        "outputPath": { "type": "string", "description": "File path to write the image to" },
                        "format": { "type": "string", "description": "Export format: PNG, SVG, JPG, or PDF" },
                        "scale": { "type": "number", "description": "Export scale for raster formats" }
                    },
                    "required": ["nodeId", "outputPath"]
                },
                "description": "List of {nodeId, outputPath, format?, scale?} objects"
            },
            "format": { "type": "string", "description": "Default export format: PNG (default), SVG, JPG, or PDF" },
            "scale": { "type": "number", "description": "Default export scale for raster formats (default 2)" }
        },
        "required": ["items"]
    })
}

fn create_frame_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "x": { "type": "number" }, "y": { "type": "number" },
            "width": { "type": "number" }, "height": { "type": "number" },
            "name": { "type": "string" }, "fillColor": { "type": "string" },
            "layoutMode": { "type": "string" },
            "paddingTop": { "type": "number" }, "paddingRight": { "type": "number" },
            "paddingBottom": { "type": "number" }, "paddingLeft": { "type": "number" },
            "itemSpacing": { "type": "number" },
            "primaryAxisAlignItems": { "type": "string" }, "counterAxisAlignItems": { "type": "string" },
            "primaryAxisSizingMode": { "type": "string" }, "counterAxisSizingMode": { "type": "string" },
            "layoutWrap": { "type": "string" }, "counterAxisSpacing": { "type": "number" },
            "parentId": { "type": "string" }
        }
    })
}

fn create_rectangle_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "x": { "type": "number" }, "y": { "type": "number" },
            "width": { "type": "number" }, "height": { "type": "number" },
            "name": { "type": "string" }, "fillColor": { "type": "string" },
            "cornerRadius": { "type": "number" }, "parentId": { "type": "string" }
        }
    })
}

fn create_ellipse_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "x": { "type": "number" }, "y": { "type": "number" },
            "width": { "type": "number" }, "height": { "type": "number" },
            "name": { "type": "string" }, "fillColor": { "type": "string" },
            "parentId": { "type": "string" }
        }
    })
}

fn create_text_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "text": { "type": "string" },
            "x": { "type": "number" }, "y": { "type": "number" },
            "fontSize": { "type": "number" }, "fontFamily": { "type": "string" },
            "fontStyle": { "type": "string" }, "fillColor": { "type": "string" },
            "name": { "type": "string" }, "parentId": { "type": "string" }
        },
        "required": ["text"]
    })
}

fn import_image_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "imageData": { "type": "string" },
            "x": { "type": "number" }, "y": { "type": "number" },
            "width": { "type": "number" }, "height": { "type": "number" },
            "name": { "type": "string" },
            "scaleMode": { "type": "string" },
            "parentId": { "type": "string" }
        },
        "required": ["imageData"]
    })
}

fn create_component_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "nodeId": node_id_string("FRAME node ID to convert, in colon format e.g. '4029:12345'"),
            "name": { "type": "string" }
        },
        "required": ["nodeId"]
    })
}

fn create_section_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "x": { "type": "number" }, "y": { "type": "number" },
            "width": { "type": "number" }, "height": { "type": "number" }
        }
    })
}

fn set_text_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "nodeId": node_id_string("TEXT node ID in colon format e.g. '4029:12345'"),
            "text": { "type": "string" }
        },
        "required": ["nodeId", "text"]
    })
}

fn set_fills_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "nodeId": node_id_string("Node ID in colon format e.g. '4029:12345'"),
            "color": { "type": "string", "description": "Fill color as hex: #RRGGBB or #RRGGBBAA" },
            "opacity": { "type": "number" }, "mode": { "type": "string" }
        },
        "required": ["nodeId", "color"]
    })
}

fn set_strokes_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "nodeId": node_id_string("Node ID in colon format e.g. '4029:12345'"),
            "color": { "type": "string", "description": "Stroke color as hex e.g. #000000" },
            "strokeWeight": { "type": "number" }, "mode": { "type": "string" }
        },
        "required": ["nodeId", "color"]
    })
}

fn move_nodes_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "nodeIds": { "type": "array", "items": { "type": "string" } },
            "x": { "type": "number" }, "y": { "type": "number" }
        },
        "required": ["nodeIds"]
    })
}

fn resize_nodes_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "nodeIds": { "type": "array", "items": { "type": "string" } },
            "width": { "type": "number" }, "height": { "type": "number" }
        },
        "required": ["nodeIds"]
    })
}

fn rename_node_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "nodeId": node_id_string("Node ID in colon format e.g. '4029:12345'"),
            "name": { "type": "string" }
        },
        "required": ["nodeId", "name"]
    })
}

fn clone_node_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "nodeId": node_id_string("Source node ID in colon format e.g. '4029:12345'"),
            "x": { "type": "number" }, "y": { "type": "number" },
            "parentId": { "type": "string" }
        },
        "required": ["nodeId"]
    })
}

fn set_opacity_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "nodeIds": { "type": "array", "items": { "type": "string" } },
            "opacity": { "type": "number", "description": "Opacity value between 0 and 1" }
        },
        "required": ["nodeIds", "opacity"]
    })
}

fn set_corner_radius_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "nodeIds": { "type": "array", "items": { "type": "string" } },
            "cornerRadius": { "type": "number" },
            "topLeftRadius": { "type": "number" }, "topRightRadius": { "type": "number" },
            "bottomLeftRadius": { "type": "number" }, "bottomRightRadius": { "type": "number" }
        },
        "required": ["nodeIds"]
    })
}

fn set_auto_layout_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "nodeId": node_id_string("Frame node ID in colon format e.g. '4029:12345'"),
            "layoutMode": { "type": "string" },
            "paddingTop": { "type": "number" }, "paddingRight": { "type": "number" },
            "paddingBottom": { "type": "number" }, "paddingLeft": { "type": "number" },
            "itemSpacing": { "type": "number" },
            "primaryAxisAlignItems": { "type": "string" }, "counterAxisAlignItems": { "type": "string" },
            "primaryAxisSizingMode": { "type": "string" }, "counterAxisSizingMode": { "type": "string" },
            "layoutWrap": { "type": "string" }, "counterAxisSpacing": { "type": "number" }
        },
        "required": ["nodeId"]
    })
}

fn set_visible_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "nodeIds": { "type": "array", "items": { "type": "string" } },
            "visible": { "type": "boolean" }
        },
        "required": ["nodeIds", "visible"]
    })
}

fn rotate_nodes_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "nodeIds": { "type": "array", "items": { "type": "string" } },
            "rotation": { "type": "number" }
        },
        "required": ["nodeIds", "rotation"]
    })
}

fn reorder_nodes_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "nodeIds": { "type": "array", "items": { "type": "string" } },
            "order": { "type": "string", "description": "bringToFront | sendToBack | bringForward | sendBackward" }
        },
        "required": ["nodeIds", "order"]
    })
}

fn set_blend_mode_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "nodeIds": { "type": "array", "items": { "type": "string" } },
            "blendMode": { "type": "string" }
        },
        "required": ["nodeIds", "blendMode"]
    })
}

fn set_constraints_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "nodeIds": { "type": "array", "items": { "type": "string" } },
            "horizontal": { "type": "string" }, "vertical": { "type": "string" }
        },
        "required": ["nodeIds"]
    })
}

fn reparent_nodes_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "nodeIds": { "type": "array", "items": { "type": "string" } },
            "parentId": { "type": "string" }
        },
        "required": ["nodeIds", "parentId"]
    })
}

fn batch_rename_nodes_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "nodeIds": { "type": "array", "items": { "type": "string" } },
            "find": { "type": "string" }, "replace": { "type": "string" },
            "useRegex": { "type": "boolean" }, "regexFlags": { "type": "string" },
            "prefix": { "type": "string" }, "suffix": { "type": "string" }
        },
        "required": ["nodeIds"]
    })
}

fn find_replace_text_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "find": { "type": "string" }, "replace": { "type": "string" },
            "nodeId": { "type": "string" },
            "useRegex": { "type": "boolean" }, "regexFlags": { "type": "string" }
        },
        "required": ["find", "replace"]
    })
}

fn create_paint_style_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" }, "color": { "type": "string" },
            "description": { "type": "string" }
        },
        "required": ["name", "color"]
    })
}

fn create_text_style_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" }, "fontSize": { "type": "number" },
            "fontFamily": { "type": "string" }, "fontStyle": { "type": "string" },
            "textDecoration": { "type": "string" },
            "lineHeightValue": { "type": "number" }, "lineHeightUnit": { "type": "string" },
            "letterSpacingValue": { "type": "number" }, "letterSpacingUnit": { "type": "string" },
            "description": { "type": "string" }
        },
        "required": ["name"]
    })
}

fn create_effect_style_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" }, "type": { "type": "string" },
            "color": { "type": "string" }, "opacity": { "type": "number" },
            "radius": { "type": "number" },
            "offsetX": { "type": "number" }, "offsetY": { "type": "number" },
            "spread": { "type": "number" }, "description": { "type": "string" }
        },
        "required": ["name"]
    })
}

fn create_grid_style_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" }, "pattern": { "type": "string" },
            "count": { "type": "number" }, "gutterSize": { "type": "number" },
            "offset": { "type": "number" }, "alignment": { "type": "string" },
            "sectionSize": { "type": "number" }, "color": { "type": "string" },
            "opacity": { "type": "number" }, "description": { "type": "string" }
        },
        "required": ["name"]
    })
}

fn update_paint_style_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "styleId": { "type": "string" }, "name": { "type": "string" },
            "color": { "type": "string" }, "description": { "type": "string" }
        },
        "required": ["styleId"]
    })
}

fn delete_style_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "styleId": { "type": "string" }
        },
        "required": ["styleId"]
    })
}

fn apply_style_to_node_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "nodeId": node_id_string("Target node ID in colon format e.g. '4029:12345'"),
            "styleId": { "type": "string" }, "target": { "type": "string" }
        },
        "required": ["nodeId", "styleId"]
    })
}

fn set_effects_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "nodeId": node_id_string("Target node ID in colon format e.g. '4029:12345'"),
            "effects": { "type": "array", "items": { "type": "object" } }
        },
        "required": ["nodeId", "effects"]
    })
}

fn bind_variable_to_node_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "nodeId": node_id_string("Target node ID in colon format e.g. '4029:12345'"),
            "variableId": { "type": "string" }, "field": { "type": "string" }
        },
        "required": ["nodeId", "variableId", "field"]
    })
}

fn create_variable_collection_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" }, "initialModeName": { "type": "string" }
        },
        "required": ["name"]
    })
}

fn add_variable_mode_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "collectionId": { "type": "string" }, "modeName": { "type": "string" }
        },
        "required": ["collectionId", "modeName"]
    })
}

fn create_variable_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" }, "collectionId": { "type": "string" },
            "type": { "type": "string", "description": "COLOR | FLOAT | STRING | BOOLEAN" },
            "value": { "type": "string" }
        },
        "required": ["name", "collectionId", "type"]
    })
}

fn set_variable_value_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "variableId": { "type": "string" }, "modeId": { "type": "string" },
            "value": { "type": "string" }
        },
        "required": ["variableId", "modeId", "value"]
    })
}

fn delete_variable_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "variableId": { "type": "string" }, "collectionId": { "type": "string" }
        }
    })
}

fn add_page_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" }, "index": { "type": "number" }
        }
    })
}

fn delete_page_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "pageId": { "type": "string" }, "pageName": { "type": "string" }
        }
    })
}

fn rename_page_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "pageId": { "type": "string" }, "pageName": { "type": "string" },
            "newName": { "type": "string" }
        },
        "required": ["newName"]
    })
}

fn navigate_to_page_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "pageId": { "type": "string" }, "pageName": { "type": "string" }
        }
    })
}

fn group_nodes_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "nodeIds": { "type": "array", "items": { "type": "string" } },
            "name": { "type": "string" }
        },
        "required": ["nodeIds"]
    })
}

fn swap_component_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "nodeId": node_id_string("INSTANCE node ID in colon format e.g. '4029:12345'"),
            "componentId": { "type": "string" }
        },
        "required": ["nodeId", "componentId"]
    })
}

fn set_reactions_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "nodeId": node_id_string("Node ID in colon format e.g. '4029:12345'"),
            "reactions": { "type": "array", "items": { "type": "object" } },
            "mode": { "type": "string", "description": "'replace' (default) or 'append'" }
        },
        "required": ["nodeId", "reactions"]
    })
}

fn remove_reactions_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "nodeId": node_id_string("Node ID in colon format e.g. '4029:12345'"),
            "indices": { "type": "array", "items": { "type": "number" } }
        },
        "required": ["nodeId"]
    })
}
