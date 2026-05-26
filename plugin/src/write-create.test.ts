import { describe, it, expect, beforeEach } from "bun:test";
import { handleWriteCreateRequest } from "./write-create";

// ── Figma global mock ─────────────────────────────────────────────────────────

let mockNodes: Record<string, any>;
let commitUndoCalled: boolean;
let createdComponents: any[];

const makeRequest = (type: string, nodeIds?: string[], params?: any) => ({
  type,
  requestId: "req-test-1",
  nodeIds: nodeIds ?? [],
  params: params ?? {},
});

beforeEach(() => {
  commitUndoCalled = false;
  createdComponents = [];
  mockNodes = {};
  (globalThis as any).figma = {
    get currentPage() { return { id: "0:1", name: "Page 1", appendChild: () => {} }; },
    getNodeByIdAsync: async (id: string) => mockNodes[id] ?? null,
    createComponent: () => {
      const comp: any = {
        id: "comp:new",
        name: "Component",
        type: "COMPONENT",
        x: 0, y: 0, width: 100, height: 100,
        fills: [], strokes: [], cornerRadius: 0, layoutMode: "NONE",
        children: [] as any[],
        resize(w: number, h: number) { this.width = w; this.height = h; },
        appendChild(child: any) { this.children.push(child); },
      };
      createdComponents.push(comp);
      return comp;
    },
    commitUndo: () => { commitUndoCalled = true; },
    mixed: Symbol("mixed"),
  };
});

// ── create_component ──────────────────────────────────────────────────────────

describe("create_component", () => {
  const makeParent = () => ({
    id: "0:1",
    children: [] as any[],
    insertChild(_: number, c: any) { this.children.push(c); },
  });

  it("converts a FRAME to a COMPONENT in place", async () => {
    const child = { id: "2:1", type: "RECTANGLE" };
    let frameRemoved = false;
    const parent = makeParent();
    const frame = {
      id: "1:1", name: "Card", type: "FRAME",
      x: 10, y: 20, width: 200, height: 100,
      fills: [{ type: "SOLID" }], strokes: [],
      cornerRadius: 8, layoutMode: "NONE",
      children: [child], parent,
      remove() { frameRemoved = true; },
    };
    parent.children = [frame];
    mockNodes["1:1"] = frame;

    const res = await handleWriteCreateRequest(makeRequest("create_component", ["1:1"]));
    expect(res?.data.type).toBe("COMPONENT");
    expect(createdComponents[0].name).toBe("Card");
    expect(createdComponents[0].cornerRadius).toBe(8);
    expect(createdComponents[0].children).toContain(child);
    expect(frameRemoved).toBe(true);
    expect(commitUndoCalled).toBe(true);
  });

  it("copies frame dimensions", async () => {
    const parent = makeParent();
    const frame = {
      id: "1:1", name: "Banner", type: "FRAME",
      x: 0, y: 0, width: 320, height: 64,
      fills: [], strokes: [], cornerRadius: 0, layoutMode: "NONE",
      children: [], parent,
      remove() {},
    };
    parent.children = [frame];
    mockNodes["1:1"] = frame;

    await handleWriteCreateRequest(makeRequest("create_component", ["1:1"]));
    expect(createdComponents[0].width).toBe(320);
    expect(createdComponents[0].height).toBe(64);
  });

  it("uses custom name when provided", async () => {
    const parent = makeParent();
    const frame = {
      id: "1:1", name: "Frame", type: "FRAME",
      x: 0, y: 0, width: 100, height: 100,
      fills: [], strokes: [], cornerRadius: 0, layoutMode: "NONE",
      children: [], parent,
      remove() {},
    };
    parent.children = [frame];
    mockNodes["1:1"] = frame;

    await handleWriteCreateRequest(makeRequest("create_component", ["1:1"], { name: "Button" }));
    expect(createdComponents[0].name).toBe("Button");
  });

  it("copies auto-layout properties when layoutMode is set", async () => {
    const parent = makeParent();
    const frame = {
      id: "1:1", name: "Row", type: "FRAME",
      x: 0, y: 0, width: 200, height: 48,
      fills: [], strokes: [], cornerRadius: 0,
      layoutMode: "HORIZONTAL",
      paddingTop: 8, paddingRight: 16, paddingBottom: 8, paddingLeft: 16,
      itemSpacing: 12,
      primaryAxisAlignItems: "CENTER",
      counterAxisAlignItems: "CENTER",
      children: [], parent,
      remove() {},
    };
    parent.children = [frame];
    mockNodes["1:1"] = frame;

    await handleWriteCreateRequest(makeRequest("create_component", ["1:1"]));
    const comp = createdComponents[0];
    expect(comp.layoutMode).toBe("HORIZONTAL");
    expect(comp.paddingTop).toBe(8);
    expect(comp.paddingRight).toBe(16);
    expect(comp.itemSpacing).toBe(12);
    expect(comp.primaryAxisAlignItems).toBe("CENTER");
  });

  it("throws when nodeId not found", async () => {
    await expect(
      handleWriteCreateRequest(makeRequest("create_component", ["9:9"]))
    ).rejects.toThrow("Node not found: 9:9");
  });

  it("throws when node is not a FRAME", async () => {
    mockNodes["1:1"] = { id: "1:1", type: "RECTANGLE" };
    await expect(
      handleWriteCreateRequest(makeRequest("create_component", ["1:1"]))
    ).rejects.toThrow("is not a FRAME");
  });

  it("throws when no nodeId provided", async () => {
    await expect(
      handleWriteCreateRequest(makeRequest("create_component", []))
    ).rejects.toThrow("nodeId is required");
  });
});

// ── create_section ────────────────────────────────────────────────────────────

describe("create_section", () => {
  let createdSection: any;

  beforeEach(() => {
    createdSection = null;
    (globalThis as any).figma = {
      ...(globalThis as any).figma,
      currentPage: { id: "0:1", name: "Page 1", appendChild: () => {} },
      createSection: () => {
        createdSection = {
          id: "section:new", name: "Section", type: "SECTION",
          x: 0, y: 0, width: 200, height: 200,
          resizeWithoutConstraints(w: number, h: number) { this.width = w; this.height = h; },
        };
        return createdSection;
      },
    };
  });

  it("creates a section with a name", async () => {
    const res = await handleWriteCreateRequest(makeRequest("create_section", [], { name: "Sprint 1" }));
    expect(createdSection.name).toBe("Sprint 1");
    expect(res?.data.type).toBe("SECTION");
    expect(res?.data.id).toBe("section:new");
    expect(commitUndoCalled).toBe(true);
  });

  it("creates a section at a specific position", async () => {
    const res = await handleWriteCreateRequest(makeRequest("create_section", [], { x: 100, y: 200 }));
    expect(createdSection.x).toBe(100);
    expect(createdSection.y).toBe(200);
  });

  it("creates a section with custom size", async () => {
    await handleWriteCreateRequest(makeRequest("create_section", [], { width: 800, height: 600 }));
    expect(createdSection.width).toBe(800);
    expect(createdSection.height).toBe(600);
  });

  it("creates a section with default values when no params given", async () => {
    const res = await handleWriteCreateRequest(makeRequest("create_section", [], {}));
    expect(res?.data.id).toBe("section:new");
  });
});
