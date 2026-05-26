# Bulk Rename Strategy

Systematically rename nodes to follow a consistent naming convention without moving or
modifying any visual properties.

## Naming Convention (BEM-style, adapt as needed)

- Screens / pages:         "ScreenName" (PascalCase)
- Section frames:          "Section/Name"
- Component instances:     "ComponentName" (match main component name)
- Containers:              "ComponentName/Container"
- Content groups:          "ComponentName/Content"
- Interactive elements:    "ComponentName/ActionName" (e.g. "Card/CTAButton")
- Text nodes:              "Label", "Title", "Body", "Caption"
- Icon wrappers:           "Icon/IconName"
- Auto-generated Figma names to avoid: "Frame 123", "Rectangle 45", "Group 6"

## Steps

1. **Understand the scope**
   Ask the user: rename the entire page, a specific frame, or just selected nodes?
   - Entire page: use get_document() to get the root node ID, then scan_nodes_by_types().
   - Specific frame: use get_node(nodeId) to inspect it first.
   - Selection: use get_selection().

2. **Scan target nodes**
   Call scan_nodes_by_types(nodeId, types=["FRAME","GROUP","INSTANCE","TEXT","RECTANGLE","ELLIPSE","VECTOR"])
   to get a flat list of all nodes in scope.

3. **Identify nodes needing rename**
   Flag nodes whose names match Figma's auto-generated patterns:
   - "Frame \d+", "Rectangle \d+", "Group \d+", "Ellipse \d+", "Vector \d+", "Component \d+"
   - Any name the user considers non-descriptive.

4. **Propose names**
   For each flagged node, derive a new name from:
   - Its node type and content (TEXT nodes → use their text content as label).
   - Its position in the hierarchy (child of "Card" frame → "Card/...").
   - Its visual role (if it contains only an icon → "Icon/...").
   - For INSTANCE nodes → use the mainComponent name.
   Show a preview table to the user before applying:
   | Node ID | Current Name | Proposed Name |

5. **Apply renames (after user confirmation)**
   Call rename_node(nodeId, name) for each node.
   Process in batches — do not wait for user confirmation between individual renames once
   the full plan is approved.

## Rules
- Never rename nodes that already follow the convention.
- Never change names of COMPONENT master nodes (only instances and frames).
- Preserve "/" hierarchy separators — do not flatten them.
- If unsure about a name, leave it and flag it for the user to decide.
