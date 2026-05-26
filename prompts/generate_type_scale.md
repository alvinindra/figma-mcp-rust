# Generate Type Scale

Given a base font family and body size, generate a full typographic scale as Figma text styles.

## Input

Ask the user for:
- Font family (e.g. "Inter") — required
- Base body font size in px (e.g. 16) — required
- Scale ratio: "minor-third" (1.2), "major-third" (1.25), "perfect-fourth" (1.333), "golden" (1.618) — default major-third
- Include display sizes (above H1)? — default no
- Include mono / code style? — default no

## Scale Calculation

Using base size B and ratio R:

| Style Name       | Size formula    | Weight | Line Height | Letter Spacing |
|------------------|-----------------|--------|-------------|----------------|
| Display/2XL      | B × R^7 (≈ px)  | 700    | 1.1         | -0.02em        |
| Display/XL       | B × R^6         | 700    | 1.1         | -0.02em        |
| Heading/H1       | B × R^5         | 700    | 1.2         | -0.01em        |
| Heading/H2       | B × R^4         | 700    | 1.25        | -0.01em        |
| Heading/H3       | B × R^3         | 600    | 1.3         | 0              |
| Heading/H4       | B × R^2         | 600    | 1.35        | 0              |
| Body/XL          | B × R^1         | 400    | 1.6         | 0              |
| Body/Base        | B               | 400    | 1.6         | 0              |
| Body/SM          | B / R           | 400    | 1.5         | 0              |
| Label/LG         | B × R^0.5 (≈px) | 500    | 1.4         | +0.01em        |
| Label/Base       | B               | 500    | 1.4         | +0.01em        |
| Label/SM         | B / R           | 500    | 1.4         | +0.02em        |
| Caption/Base     | B / R^1.5 (≈px) | 400    | 1.4         | +0.02em        |
| Code/Base        | B               | 400    | 1.6         | 0 (mono font)  |

Round all sizes to nearest integer. Minimum size: 10px.

Show the full table to the user for review before creating anything.

## Creation Steps

1. For each style row, call create_text_style() with:
   - name: e.g. "Heading/H1"
   - fontFamily: the chosen font
   - fontStyle: map weight to style name ("Regular"=400, "Medium"=500, "SemiBold"=600, "Bold"=700)
   - fontSize: calculated value
   - lineHeightValue + lineHeightUnit="PIXELS" (convert ratio × size to px)
   - letterSpacingValue + letterSpacingUnit="PERCENT" (convert em to %)

2. Skip styles already present with the same name (create_text_style is idempotent).

## Rules
- Always show the scale preview table before executing.
- Use "Regular" font style for weight 400, "Medium" for 500, "SemiBold" for 600, "Bold" for 700.
  Adjust if the chosen font uses different style names (ask user to confirm).
- Line height in PIXELS = round(fontSize × lineHeightRatio).
- Letter spacing in PERCENT = letterSpacingEm × 100.
