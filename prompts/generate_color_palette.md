# Generate Color Palette

Given one or more brand colors, generate a full design-system color palette with a primitive
scale and semantic aliases, then create them as Figma variables.

## Input

Ask the user for:
- Primary brand color (hex) — required
- Secondary/accent color (hex) — optional
- Whether to include neutral/gray scale — default yes
- Whether to generate dark mode — default yes

## Color Scale Algorithm

For each brand color, generate a 9-step scale (50, 100, 200, 300, 400, 500, 600, 700, 800, 900)
by varying lightness in HSL space:
- 50  → lightest tint  (~95% lightness)
- 100 → (~90% lightness)
- 200 → (~80% lightness)
- 300 → (~70% lightness)
- 400 → (~60% lightness)
- 500 → base color (the input hex)
- 600 → (~45% lightness)
- 700 → (~35% lightness)
- 800 → (~25% lightness)
- 900 → darkest shade (~15% lightness)

For neutrals: use the primary hue but desaturate to 5–10% saturation.

Show the full color table to the user for review before creating anything.

## Semantic Aliases

After the primitive scale, create semantic tokens that reference primitives:

Light mode:
- Color/Background/Default  → Neutral/50
- Color/Background/Subtle   → Neutral/100
- Color/Text/Default        → Neutral/900
- Color/Text/Subtle         → Neutral/600
- Color/Text/Disabled       → Neutral/400
- Color/Primary/Default     → Primary/500
- Color/Primary/Hover       → Primary/600
- Color/Primary/Active      → Primary/700
- Color/Primary/Subtle      → Primary/100
- Color/Border/Default      → Neutral/200
- Color/Border/Focus        → Primary/500

Dark mode (add a "Dark" mode to the same collection):
- Color/Background/Default  → Neutral/900
- Color/Background/Subtle   → Neutral/800
- Color/Text/Default        → Neutral/50
- Color/Text/Subtle         → Neutral/300
- Color/Primary/Default     → Primary/400
- (etc.)

## Creation Steps

1. create_variable_collection(name="Primitives", modeName="Value")
2. For each color in the scale: create_variable(type="COLOR", name="Primary/500", collectionId=...)
   then set_variable_value(variableId, modeId, value="#hexcolor")
3. Repeat for secondary and neutrals.
4. create_variable_collection(name="Semantic Colors", modeName="Light")
5. add_variable_mode(collectionId, modeName="Dark") — if dark mode requested
6. For each semantic alias: create_variable + set_variable_value for Light mode, then Dark mode.

## Rules
- Always show the color table preview before executing creation.
- Create Primitives collection first, Semantic collection second.
- Use only hex values for variable colors.
- Semantic variable values reference other variables conceptually — set the actual resolved hex value
  since variable aliasing (variable-to-variable binding) is not yet supported via MCP.
