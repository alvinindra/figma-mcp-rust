# Style Audit Strategy

Find all nodes that use raw (unlinked) fill colors, text styles, or effect styles instead of the
design system's named styles or variables. Report findings and optionally fix them.

## Steps

1. **Collect the design system**
   - Call get_styles() to list all local paint, text, effect, and grid styles (note their names and IDs).
   - Call get_variable_defs() to list all local COLOR variables (note their names and IDs).

2. **Scan the design**
   - Call get_design_context() with detail="compact" to get the full node tree.
   - For each node that has a fills, strokes, or textStyle property:
     - If the node's style field shows a named style (e.g. "fillStyle": "Brand/Primary") → already linked, skip.
     - If the node shows a raw fill color (e.g. "fills": [{"type":"SOLID","color":...}]) without a style name → flag it.
     - If a TEXT node shows raw fontFamily/fontSize without a textStyle name → flag it.

3. **Match raw values to existing styles**
   - For each flagged node, check whether the raw hex color matches any existing paint style color.
   - If a match is found → recommend apply_style_to_node() to link the node to that style.
   - If no match is found → note the raw value as a design system gap (a new style may be needed).

4. **Report findings**
   Present a table:
   | Node ID | Node Name | Issue | Raw Value | Matching Style |
   |---------|-----------|-------|-----------|----------------|

5. **Fix (optional, ask user first)**
   For each node with a matching style, call:
     apply_style_to_node(nodeId, styleId, target)
   Batch nodes by styleId to minimize round trips.

## Rules
- Never change a node's visual appearance — only link it to a style that already matches.
- Skip INSTANCE nodes whose overrides intentionally diverge from the main component.
- Process in chunks of 20 nodes at a time when scanning large trees.
