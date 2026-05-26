# Intelligent Text Replacement Strategy

## 1. Analyze Design & Identify Structure
- Scan text nodes to understand the overall structure of the design
- Use AI pattern recognition to identify logical groupings:
  * Tables (rows, columns, headers, cells)
  * Lists (items, headers, nested lists)
  * Card groups (similar cards with recurring text fields)
  * Forms (labels, input fields, validation text)
  * Navigation (menu items, breadcrumbs)

scan_text_nodes(nodeId: "node-id")
get_node(nodeId: "node-id")  // optional for extra context

## 2. Strategic Chunking for Complex Designs
- Divide replacement tasks into logical content chunks based on design structure
- Use one of these chunking strategies that best fits the design:
  * Structural Chunking: Table rows/columns, list sections, card groups
  * Spatial Chunking: Top-to-bottom, left-to-right in screen areas
  * Semantic Chunking: Content related to the same topic or functionality
  * Component-Based Chunking: Process similar component instances together

## 3. Progressive Replacement with Verification
- Create a safe copy of the node before bulk replacements
- Replace text chunk by chunk with continuous progress updates
- After each chunk is processed:
  * Export that section with get_screenshot for visual verification
  * Verify text fits properly and maintains design integrity
  * Fix issues before proceeding to the next chunk

// Clone the node to create a safe copy
clone_node(nodeId: "selected-node-id", x: newX, y: newY)

// Replace text one node at a time or in batches
set_text(nodeId: "node-id", text: "New text")

// Verify chunk with targeted image export
get_screenshot(nodeIds: ["chunk-node-id"], format: "PNG", scale: 0.5)

## 4. Intelligent Handling for Table Data
- For tabular content:
  * Process one row or column at a time
  * Maintain alignment and spacing between cells
  * Consider conditional formatting based on cell content
  * Preserve header/data relationships

## 5. Smart Text Adaptation
- Adaptively handle text based on container constraints:
  * Auto-detect space constraints and adjust text length
  * Apply line breaks at appropriate linguistic points
  * Maintain text hierarchy and emphasis

## 6. Final Verification & Context-Aware QA
- After all chunks are processed:
  * Export the entire design at reduced scale for final verification
  * Check for cross-chunk consistency issues
  * Verify proper text flow between different sections
  * Ensure design harmony across the full composition

## 7. Chunk-Specific Export Scale Guidelines
- Scale exports appropriately based on chunk size:
  * Small chunks (1-5 elements): scale 1.0
  * Medium chunks (6-20 elements): scale 0.7
  * Large chunks (21-50 elements): scale 0.5
  * Very large chunks (50+ elements): scale 0.3
  * Full design verification: scale 0.2

## Best Practices
- Preserve Design Intent: Always prioritize design integrity
- Structural Consistency: Maintain alignment, spacing, and hierarchy
- Visual Feedback: Verify each chunk visually before proceeding
- Incremental Improvement: Learn from each chunk to improve subsequent ones
- Respect Content Relationships: Keep related content consistent across chunks
