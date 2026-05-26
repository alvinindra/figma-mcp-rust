# Generate Component Variants

Given an existing frame or component, produce a set of visual variants (e.g. sizes, color themes,
states) by cloning and mutating it. Arrange the variants in a tidy grid for review.

## Input

Ask the user:
- Source node ID (the base component or frame to clone)
- What variants to generate — choose one or more:
  a) **Sizes** — Small, Medium, Large (scale width/height, adjust font size and padding)
  b) **Color themes** — e.g. Primary, Secondary, Danger, Success, Warning
  c) **States** — Default, Hover, Pressed, Disabled, Loading
  d) **Dark mode** — duplicate with inverted background/text colors
- Arrange output on same page or new frame? (default: new container frame)

## Steps

### 1. Inspect the source
Call get_node(sourceNodeId) to understand:
- Width, height, position
- Fill colors (note hex values)
- Text content and sizes
- Child structure

### 2. Plan the variant grid
Calculate layout:
- Each clone = source width × source height
- Gap between clones = 24px
- Label each clone with its variant name (create_text node below each)
- Total container width = (cloneWidth + 24) × columns

### 3. Create container frame (if requested)
create_frame(name="Variants/ComponentName", width=totalWidth, height=totalHeight,
             layoutMode="HORIZONTAL", itemSpacing=24, paddingTop=32, paddingLeft=32,
             paddingRight=32, paddingBottom=32)

### 4. For each variant

**Sizes:**
- Clone source: clone_node(sourceId, parentId=containerId)
- Compute scale factor (SM=0.75, MD=1.0, LG=1.5)
- resize_nodes to new dimensions
- For TEXT children: set_text to same content (font size cannot be changed via MCP — note this limitation)
- rename_node to "ComponentName/SM" etc.

**Color themes:**
- Clone source: clone_node(sourceId, parentId=containerId)
- For each fill-bearing child: set_fills(nodeId, color=themeHex)
- Color mapping suggestion:
  - Primary   → use brand primary color
  - Secondary → use brand secondary color
  - Danger    → #EF4444
  - Success   → #22C55E
  - Warning   → #F59E0B
- rename_node to "ComponentName/Primary" etc.

**States:**
- Clone source: clone_node(sourceId, parentId=containerId)
- Disabled: set_fills on background to gray (#94A3B8), reduce fill opacity of text nodes
- Hover: slightly lighten the primary fill
- rename_node to "ComponentName/Hover" etc.

**Dark mode:**
- Clone source: clone_node(sourceId, parentId=containerId)
- Swap background fill to dark (#1E293B or similar)
- Swap text fills to light (#F8FAFC)
- rename_node to "ComponentName/Dark"

### 5. Summarize
Report all created node IDs and names. Ask the user if they want further adjustments.

## Rules
- Always inspect the source node before cloning.
- Never modify the original source node.
- Keep all variants on the same page unless the user requests otherwise.
- Add a text label below each variant showing its name.
