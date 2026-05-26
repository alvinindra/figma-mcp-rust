# Swap Component Instance and Override Strategy

## Overview
Transfer content and property overrides from a source instance to one or more target instances
in Figma, maintaining design consistency while reducing manual work.

## Step-by-Step Process

### 1. Selection Analysis
- Use get_selection() to identify the parent component or selected instances
- For parent components, scan for instances with:
  scan_nodes_by_types(nodeId: "parent-id", types: ["INSTANCE"])
- Identify custom slots by name patterns (e.g. "Custom Slot*" or "Instance Slot")
- Determine which is the source instance (with content to copy) and which are targets

### 2. Inspect Source Instance
- Use get_node(nodeId: "source-instance-id") to examine the source instance structure
- Use get_nodes_info(nodeIds: [...]) to batch-inspect multiple instances
- Use scan_text_nodes(nodeId: "source-instance-id") to capture all text content

### 3. Apply Overrides to Targets
- For text overrides: use set_text(nodeId: "target-text-node-id", text: "copied text")
- For fill overrides: use set_fills(nodeId: "target-node-id", color: "#hexcolor")
- For stroke overrides: use set_strokes(nodeId: "target-node-id", color: "#hexcolor")
- Process targets one at a time or identify patterns to apply systematically

### 4. Verification
- Verify results with get_node() or get_design_context()
- Confirm text content and style overrides have transferred successfully
- Use get_screenshot() for visual confirmation if needed

## Key Tips
- Use scan_nodes_by_types to enumerate all instances before starting
- When working with multiple targets, check the full selection with get_selection()
- Prefer reading the full node tree of the source first to understand its structure
- Keep related content consistent across all target instances
