# Automatic Annotation Conversion

## Process Overview
Convert manual annotations (numbered/alphabetical indicators with connected descriptions) to Figma's native annotations:

1. Get selected frame/component information
2. Scan and collect all annotation text nodes
3. Scan target UI elements (components, instances, frames)
4. Match annotations to appropriate UI elements
5. Apply native Figma annotations

## Step 1: Get Selection and Initial Setup

// Get the selected frame/component
get_selection()
// Note the selected node ID, then:
get_annotations(nodeId: "selected-node-id")

## Step 2: Scan Annotation Text Nodes

// Get all text nodes in the selection
scan_text_nodes(nodeId: "selected-node-id")

// Filter and group annotation markers and descriptions
// Markers typically have these characteristics:
// - Short text content (usually single digit/letter)
// - Specific font styles (often bold)
// - Located in a container with "Marker" or "Dot" in the name
// - Have a clear naming pattern (e.g., "1", "2", "3" or "A", "B", "C")

## Step 3: Scan Target UI Elements

// Get all potential target elements that annotations might refer to
scan_nodes_by_types(nodeId: "selected-node-id", types: ["COMPONENT", "INSTANCE", "FRAME"])

## Step 4: Match Annotations to Targets

Match each annotation to its target UI element using these strategies in order of priority:

1. Path-Based Matching:
   - Look at the marker's parent container name in the Figma layer hierarchy
   - Remove any "Marker:" or "Annotation:" prefixes from the parent name
   - Find UI elements that share the same parent name or have it in their path

2. Name-Based Matching:
   - Extract key terms from the annotation description
   - Look for UI elements whose names contain these key terms
   - Particularly effective for form fields, buttons, and labeled components

3. Proximity-Based Matching (fallback):
   - Calculate the center point of the marker using its bounds
   - Find the closest UI element by measuring distances to element centers
   - Use this method when other matching strategies fail

## Step 5: Verify Results

After converting annotations, verify with:
get_annotations(nodeId: "selected-node-id")
get_screenshot(nodeIds: ["selected-node-id"], format: "PNG", scale: 0.5)

This strategy focuses on practical implementation based on real-world usage patterns,
emphasizing the importance of handling various UI elements as annotation targets.
