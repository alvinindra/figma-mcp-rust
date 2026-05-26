import { describe, it, expect, beforeEach } from "bun:test";
import { handleWriteStyleRequest } from "./write-styles";

// ── Figma global mock ─────────────────────────────────────────────────────────

let mockNodes: Record<string, any>;
let commitUndoCalled: boolean;

const makeRequest = (type: string, nodeIds?: string[], params?: any) => ({
  type,
  requestId: "req-test-1",
  nodeIds: nodeIds ?? [],
  params: params ?? {},
});

beforeEach(() => {
  commitUndoCalled = false;
  mockNodes = {};
  (globalThis as any).figma = {
    getNodeByIdAsync: async (id: string) => mockNodes[id] ?? null,
    commitUndo: () => { commitUndoCalled = true; },
    getLocalPaintStylesAsync: async () => [],
    getLocalTextStylesAsync:  async () => [],
    getLocalEffectStylesAsync: async () => [],
    getLocalGridStylesAsync:  async () => [],
    getStyleByIdAsync: async () => null,
    loadFontAsync: async () => {},
    variables: {
      getVariableByIdAsync: async () => null,
      getVariableCollectionByIdAsync: async () => null,
    },
  };
});

// ── set_effects ───────────────────────────────────────────────────────────────

describe("set_effects", () => {
  it("sets a drop shadow effect", async () => {
    mockNodes["1:1"] = { id: "1:1", name: "Card", effects: [] };
    const res = await handleWriteStyleRequest(makeRequest("set_effects", ["1:1"], {
      effects: [{ type: "DROP_SHADOW", color: "#000000", opacity: 0.3, radius: 8, offsetX: 0, offsetY: 4 }],
    }));
    expect(mockNodes["1:1"].effects).toHaveLength(1);
    expect(mockNodes["1:1"].effects[0].type).toBe("DROP_SHADOW");
    expect(mockNodes["1:1"].effects[0].radius).toBe(8);
    expect(mockNodes["1:1"].effects[0].color.a).toBe(0.3);
    expect(res?.data.effectCount).toBe(1);
    expect(commitUndoCalled).toBe(true);
  });

  it("sets an inner shadow effect", async () => {
    mockNodes["1:1"] = { id: "1:1", effects: [] };
    await handleWriteStyleRequest(makeRequest("set_effects", ["1:1"], {
      effects: [{ type: "INNER_SHADOW", radius: 4 }],
    }));
    expect(mockNodes["1:1"].effects[0].type).toBe("INNER_SHADOW");
  });

  it("sets a layer blur effect", async () => {
    mockNodes["1:1"] = { id: "1:1", effects: [] };
    await handleWriteStyleRequest(makeRequest("set_effects", ["1:1"], {
      effects: [{ type: "LAYER_BLUR", radius: 10 }],
    }));
    expect(mockNodes["1:1"].effects[0].type).toBe("LAYER_BLUR");
    expect(mockNodes["1:1"].effects[0].radius).toBe(10);
  });

  it("sets a background blur effect", async () => {
    mockNodes["1:1"] = { id: "1:1", effects: [] };
    await handleWriteStyleRequest(makeRequest("set_effects", ["1:1"], {
      effects: [{ type: "BACKGROUND_BLUR", radius: 20 }],
    }));
    expect(mockNodes["1:1"].effects[0].type).toBe("BACKGROUND_BLUR");
  });

  it("sets multiple effects at once", async () => {
    mockNodes["1:1"] = { id: "1:1", effects: [] };
    await handleWriteStyleRequest(makeRequest("set_effects", ["1:1"], {
      effects: [
        { type: "DROP_SHADOW", radius: 4 },
        { type: "LAYER_BLUR", radius: 2 },
      ],
    }));
    expect(mockNodes["1:1"].effects).toHaveLength(2);
  });

  it("clears effects when empty array provided", async () => {
    mockNodes["1:1"] = { id: "1:1", effects: [{ type: "DROP_SHADOW" }] };
    const res = await handleWriteStyleRequest(makeRequest("set_effects", ["1:1"], { effects: [] }));
    expect(mockNodes["1:1"].effects).toHaveLength(0);
    expect(res?.data.effectCount).toBe(0);
  });

  it("uses default values for shadow", async () => {
    mockNodes["1:1"] = { id: "1:1", effects: [] };
    await handleWriteStyleRequest(makeRequest("set_effects", ["1:1"], {
      effects: [{ type: "DROP_SHADOW" }],
    }));
    const shadow = mockNodes["1:1"].effects[0];
    expect(shadow.radius).toBe(4);
    expect(shadow.offset.x).toBe(0);
    expect(shadow.offset.y).toBe(4);
    expect(shadow.color.a).toBe(0.25); // default opacity
  });

  it("throws for unknown effect type", async () => {
    mockNodes["1:1"] = { id: "1:1", effects: [] };
    await expect(handleWriteStyleRequest(makeRequest("set_effects", ["1:1"], {
      effects: [{ type: "GLOW" }],
    }))).rejects.toThrow("Unknown effect type");
  });

  it("throws if nodeId is missing", async () => {
    await expect(handleWriteStyleRequest(makeRequest("set_effects", [], {
      effects: [{ type: "DROP_SHADOW" }],
    }))).rejects.toThrow("nodeId is required");
  });

  it("throws if effects is not an array", async () => {
    mockNodes["1:1"] = { id: "1:1", effects: [] };
    await expect(handleWriteStyleRequest(makeRequest("set_effects", ["1:1"], {
      effects: "shadow",
    }))).rejects.toThrow("effects array is required");
  });

  it("throws if node not found", async () => {
    await expect(handleWriteStyleRequest(makeRequest("set_effects", ["9:9"], {
      effects: [{ type: "DROP_SHADOW" }],
    }))).rejects.toThrow("Node not found");
  });

  it("throws if node does not support effects", async () => {
    mockNodes["1:1"] = { id: "1:1" }; // no effects property
    await expect(handleWriteStyleRequest(makeRequest("set_effects", ["1:1"], {
      effects: [{ type: "DROP_SHADOW" }],
    }))).rejects.toThrow("does not support effects");
  });
});

// ── bind_variable_to_node – strokeColor ──────────────────────────────────────

describe("bind_variable_to_node strokeColor", () => {
  const mockVariable = { id: "v1", name: "color/primary", resolvedType: "COLOR" };
  const mockPaint = { type: "SOLID", color: { r: 1, g: 0, b: 0 } };

  beforeEach(() => {
    (globalThis as any).figma.variables = {
      getVariableByIdAsync: async (id: string) => id === "v1" ? mockVariable : null,
      setBoundVariableForPaint: (_paint: any, _field: string, _variable: any) => mockPaint,
    };
  });

  it("binds a variable to strokeColor", async () => {
    mockNodes["1:1"] = { id: "1:1", name: "Frame", strokes: [], setBoundVariable: () => {} };
    const res = await handleWriteStyleRequest(makeRequest("bind_variable_to_node", ["1:1"], {
      variableId: "v1", field: "strokeColor",
    }));
    expect(res?.data.field).toBe("strokeColor");
    expect(mockNodes["1:1"].strokes).toHaveLength(1);
    expect(commitUndoCalled).toBe(true);
  });

  it("uses existing stroke as base when binding strokeColor", async () => {
    const existingStroke = { type: "SOLID", color: { r: 0, g: 0, b: 0 } };
    mockNodes["1:1"] = { id: "1:1", strokes: [existingStroke], setBoundVariable: () => {} };
    await handleWriteStyleRequest(makeRequest("bind_variable_to_node", ["1:1"], {
      variableId: "v1", field: "strokeColor",
    }));
    expect(mockNodes["1:1"].strokes).toHaveLength(1);
  });

  it("throws if node does not support strokes", async () => {
    mockNodes["1:1"] = { id: "1:1", name: "Text" }; // no strokes
    await expect(handleWriteStyleRequest(makeRequest("bind_variable_to_node", ["1:1"], {
      variableId: "v1", field: "strokeColor",
    }))).rejects.toThrow("does not support strokes");
  });
});
