import { describe, it, expect, beforeEach } from "bun:test";
import { handleReadStyleRequest } from "./read-styles";

// ── Figma global mock ─────────────────────────────────────────────────────────

const makeRequest = (type: string, params?: any) => ({
  type,
  requestId: "req-test-1",
  nodeIds: [],
  params: params ?? {},
});

beforeEach(() => {
  (globalThis as any).figma = {
    variables: {
      getLocalVariableCollectionsAsync: async () => [],
      getVariableByIdAsync: async () => null,
    },
    getLocalPaintStylesAsync: async () => [],
    getLocalTextStylesAsync: async () => [],
    getLocalEffectStylesAsync: async () => [],
    getLocalGridStylesAsync: async () => [],
    getStyleByIdAsync: async () => null,
  };
});

// ── export_tokens ─────────────────────────────────────────────────────────────

describe("export_tokens", () => {
  it("returns empty token object when there are no variables or styles", async () => {
    const res = await handleReadStyleRequest(makeRequest("export_tokens", { format: "json" }));
    expect(res?.data.tokens).toBeDefined();
    expect(typeof res?.data.tokens).toBe("object");
    expect(Object.keys(res?.data.tokens)).toHaveLength(0);
  });

  it("returns :root CSS block even when empty", async () => {
    const res = await handleReadStyleRequest(makeRequest("export_tokens", { format: "css" }));
    expect(res?.data.css).toBeDefined();
    expect(res?.data.css).toContain(":root {");
    expect(res?.data.css).toContain("}");
  });

  it("defaults to JSON when format is not specified", async () => {
    const res = await handleReadStyleRequest(makeRequest("export_tokens", {}));
    expect(res?.data.tokens).toBeDefined();
  });

  it("builds nested JSON token tree from variable names using / as separator", async () => {
    (globalThis as any).figma.variables = {
      getLocalVariableCollectionsAsync: async () => [
        {
          id: "col:1",
          name: "Brand",
          modes: [{ modeId: "m1", name: "Default" }],
          variableIds: ["var:1"],
        },
      ],
      getVariableByIdAsync: async (id: string) =>
        id === "var:1"
          ? {
              id: "var:1",
              name: "Primary/Blue",
              resolvedType: "COLOR",
              valuesByMode: { m1: { r: 0, g: 0.47, b: 1, a: 1 } },
            }
          : null,
    };

    const res = await handleReadStyleRequest(makeRequest("export_tokens", { format: "json" }));
    expect(res?.data.tokens["Brand"]).toBeDefined();
    expect(res?.data.tokens["Brand"]["Primary"]["Blue"]).toBeDefined();
    expect(res?.data.tokens["Brand"]["Primary"]["Blue"].type).toBe("COLOR");
    expect(res?.data.tokens["Brand"]["Primary"]["Blue"].value["Default"]).toBeDefined();
  });

  it("emits per-mode values in JSON output", async () => {
    (globalThis as any).figma.variables = {
      getLocalVariableCollectionsAsync: async () => [
        {
          id: "col:1",
          name: "Spacing",
          modes: [
            { modeId: "m1", name: "Default" },
            { modeId: "m2", name: "Dense" },
          ],
          variableIds: ["var:1"],
        },
      ],
      getVariableByIdAsync: async (id: string) =>
        id === "var:1"
          ? {
              id: "var:1",
              name: "base",
              resolvedType: "FLOAT",
              valuesByMode: { m1: 8, m2: 4 },
            }
          : null,
    };

    const res = await handleReadStyleRequest(makeRequest("export_tokens", { format: "json" }));
    const token = res?.data.tokens["Spacing"]["base"];
    expect(token.value["Default"]).toBe(8);
    expect(token.value["Dense"]).toBe(4);
  });

  it("emits CSS custom property with kebab-case name from / separator", async () => {
    (globalThis as any).figma.variables = {
      getLocalVariableCollectionsAsync: async () => [
        {
          id: "col:1",
          name: "Spacing",
          modes: [{ modeId: "m1", name: "Default" }],
          variableIds: ["var:1"],
        },
      ],
      getVariableByIdAsync: async (id: string) =>
        id === "var:1"
          ? {
              id: "var:1",
              name: "spacing/base",
              resolvedType: "FLOAT",
              valuesByMode: { m1: 8 },
            }
          : null,
    };

    const res = await handleReadStyleRequest(makeRequest("export_tokens", { format: "css" }));
    expect(res?.data.css).toContain("--spacing-base: 8;");
  });

  it("emits rgba() for COLOR variables with alpha < 1", async () => {
    (globalThis as any).figma.variables = {
      getLocalVariableCollectionsAsync: async () => [
        {
          id: "col:1",
          name: "Brand",
          modes: [{ modeId: "m1", name: "Default" }],
          variableIds: ["var:1"],
        },
      ],
      getVariableByIdAsync: async (id: string) =>
        id === "var:1"
          ? {
              id: "var:1",
              name: "overlay",
              resolvedType: "COLOR",
              valuesByMode: { m1: { r: 0, g: 0, b: 0, a: 0.5 } },
            }
          : null,
    };

    const res = await handleReadStyleRequest(makeRequest("export_tokens", { format: "css" }));
    expect(res?.data.css).toContain("rgba(0, 0, 0, 0.50)");
  });

  it("includes solid paint styles under _styles.paint in JSON", async () => {
    (globalThis as any).figma.getLocalPaintStylesAsync = async () => [
      {
        id: "s:1",
        name: "Neutral/Gray",
        paints: [{ type: "SOLID", color: { r: 0.5, g: 0.5, b: 0.5 }, opacity: 1 }],
      },
    ];

    const res = await handleReadStyleRequest(makeRequest("export_tokens", { format: "json" }));
    expect(res?.data.tokens["_styles"]).toBeDefined();
    const gray = res?.data.tokens["_styles"].paint["Neutral"]["Gray"];
    expect(gray.type).toBe("COLOR");
    expect(gray.value).toMatch(/^#[0-9a-f]{6}$/);
  });

  it("includes solid paint styles as CSS custom properties", async () => {
    (globalThis as any).figma.getLocalPaintStylesAsync = async () => [
      {
        id: "s:1",
        name: "brand/primary",
        paints: [{ type: "SOLID", color: { r: 1, g: 0, b: 0 }, opacity: 1 }],
      },
    ];

    const res = await handleReadStyleRequest(makeRequest("export_tokens", { format: "css" }));
    expect(res?.data.css).toContain("--brand-primary:");
  });

  it("skips non-solid paint styles", async () => {
    (globalThis as any).figma.getLocalPaintStylesAsync = async () => [
      {
        id: "s:2",
        name: "Gradient/Blue",
        paints: [{ type: "GRADIENT_LINEAR" }],
      },
    ];

    const res = await handleReadStyleRequest(makeRequest("export_tokens", { format: "json" }));
    // No _styles key since nothing was added
    expect(res?.data.tokens["_styles"]).toBeUndefined();
  });
});
