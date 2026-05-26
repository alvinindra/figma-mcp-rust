import { describe, it, expect, beforeEach } from "bun:test";
import {
  hexToRgb,
  makeSolidPaint,
  applyAutoLayout,
  base64ToBytes,
  getParentNode,
} from "./write-helpers";

// ── Figma global mock ─────────────────────────────────────────────────────────

let mockCurrentPage: any;
let mockGetNodeByIdAsync: (id: string) => Promise<any>;

beforeEach(() => {
  mockCurrentPage = { id: "0:1", name: "Page 1" };
  mockGetNodeByIdAsync = async (_id: string) => null;
  (globalThis as any).figma = {
    get currentPage() { return mockCurrentPage; },
    getNodeByIdAsync: (id: string) => mockGetNodeByIdAsync(id),
  };
});

// ── hexToRgb ──────────────────────────────────────────────────────────────────

describe("hexToRgb", () => {
  it("converts 6-char hex to rgb with alpha 1", () => {
    const result = hexToRgb("#ff0000");
    expect(result.r).toBeCloseTo(1);
    expect(result.g).toBe(0);
    expect(result.b).toBe(0);
    expect(result.a).toBe(1);
  });

  it("converts black #000000", () => {
    const result = hexToRgb("#000000");
    expect(result.r).toBe(0);
    expect(result.g).toBe(0);
    expect(result.b).toBe(0);
    expect(result.a).toBe(1);
  });

  it("converts white #ffffff", () => {
    const result = hexToRgb("#ffffff");
    expect(result.r).toBeCloseTo(1);
    expect(result.g).toBeCloseTo(1);
    expect(result.b).toBeCloseTo(1);
    expect(result.a).toBe(1);
  });

  it("converts 8-char hex with alpha", () => {
    // #ff000080 → alpha = 0x80 / 255 ≈ 0.502
    const result = hexToRgb("#ff000080");
    expect(result.r).toBeCloseTo(1);
    expect(result.a).toBeCloseTo(128 / 255);
  });

  it("works without leading #", () => {
    const result = hexToRgb("00ff00");
    expect(result.g).toBeCloseTo(1);
  });
});

// ── makeSolidPaint ────────────────────────────────────────────────────────────

describe("makeSolidPaint", () => {
  it("creates a solid paint from a hex string", () => {
    const paint = makeSolidPaint("#ff0000");
    expect(paint.type).toBe("SOLID");
    expect((paint.color as any).r).toBeCloseTo(1);
    expect((paint as any).opacity).toBeUndefined();
  });

  it("omits opacity when alpha is 1", () => {
    const paint = makeSolidPaint("#ffffff");
    expect((paint as any).opacity).toBeUndefined();
  });

  it("sets opacity when alpha < 1", () => {
    // #ff000080 → a ≈ 128/255
    const paint = makeSolidPaint("#ff000080");
    expect((paint as any).opacity).toBeCloseTo(128 / 255);
  });

  it("uses opacityOverride over alpha channel", () => {
    const paint = makeSolidPaint("#ff000080", 0.25);
    expect((paint as any).opacity).toBe(0.25);
  });

  it("creates paint from an object color input", () => {
    const paint = makeSolidPaint({ r: 0, g: 1, b: 0, a: 1 });
    expect(paint.type).toBe("SOLID");
    expect((paint.color as any).g).toBeCloseTo(1);
    expect((paint as any).opacity).toBeUndefined();
  });

  it("uses opacity from object input when a < 1", () => {
    const paint = makeSolidPaint({ r: 0, g: 0, b: 1, a: 0.5 });
    expect((paint as any).opacity).toBe(0.5);
  });

  it("defaults a to 1 when not provided in object input", () => {
    const paint = makeSolidPaint({ r: 0, g: 0, b: 1 });
    expect((paint as any).opacity).toBeUndefined();
  });
});

// ── applyAutoLayout ───────────────────────────────────────────────────────────

describe("applyAutoLayout", () => {
  const makeFrame = () => ({
    layoutMode: "NONE" as string,
    paddingTop: 0,
    paddingRight: 0,
    paddingBottom: 0,
    paddingLeft: 0,
    itemSpacing: 0,
    primaryAxisAlignItems: undefined as any,
    counterAxisAlignItems: undefined as any,
    primaryAxisSizingMode: undefined as any,
    counterAxisSizingMode: undefined as any,
    layoutWrap: undefined as any,
    counterAxisSpacing: undefined as any,
  });

  it("sets layoutMode", () => {
    const frame = makeFrame();
    applyAutoLayout(frame as any, { layoutMode: "HORIZONTAL" });
    expect(frame.layoutMode).toBe("HORIZONTAL");
  });

  it("sets padding values", () => {
    const frame = makeFrame();
    applyAutoLayout(frame as any, { paddingTop: 8, paddingRight: 16, paddingBottom: 8, paddingLeft: 16 });
    expect(frame.paddingTop).toBe(8);
    expect(frame.paddingRight).toBe(16);
  });

  it("sets itemSpacing", () => {
    const frame = makeFrame();
    applyAutoLayout(frame as any, { itemSpacing: 12 });
    expect(frame.itemSpacing).toBe(12);
  });

  it("sets axis alignment when layoutMode is not NONE", () => {
    const frame = makeFrame();
    applyAutoLayout(frame as any, {
      layoutMode: "HORIZONTAL",
      primaryAxisAlignItems: "CENTER",
      counterAxisAlignItems: "MIN",
      primaryAxisSizingMode: "FIXED",
      counterAxisSizingMode: "AUTO",
      layoutWrap: "NO_WRAP",
    });
    expect(frame.primaryAxisAlignItems).toBe("CENTER");
    expect(frame.counterAxisAlignItems).toBe("MIN");
    expect(frame.primaryAxisSizingMode).toBe("FIXED");
    expect(frame.counterAxisSizingMode).toBe("AUTO");
    expect(frame.layoutWrap).toBe("NO_WRAP");
  });

  it("does not set axis props when layoutMode is NONE", () => {
    const frame = makeFrame();
    applyAutoLayout(frame as any, {
      primaryAxisAlignItems: "CENTER",
    });
    expect(frame.primaryAxisAlignItems).toBeUndefined();
  });

  it("sets counterAxisSpacing only when layoutWrap is WRAP", () => {
    const frame = makeFrame();
    applyAutoLayout(frame as any, {
      layoutMode: "HORIZONTAL",
      layoutWrap: "WRAP",
      counterAxisSpacing: 8,
    });
    expect(frame.counterAxisSpacing).toBe(8);
  });

  it("skips counterAxisSpacing when not WRAP", () => {
    const frame = makeFrame();
    applyAutoLayout(frame as any, {
      layoutMode: "HORIZONTAL",
      layoutWrap: "NO_WRAP",
      counterAxisSpacing: 8,
    });
    expect(frame.counterAxisSpacing).toBeUndefined();
  });
});

// ── base64ToBytes ─────────────────────────────────────────────────────────────

describe("base64ToBytes", () => {
  it("decodes a known base64 string", () => {
    // "Man" → TWFu
    const bytes = base64ToBytes("TWFu");
    expect(bytes).toEqual(new Uint8Array([77, 97, 110]));
  });

  it("decodes base64 with single padding", () => {
    // "Ma" → TWE=
    const bytes = base64ToBytes("TWE=");
    expect(bytes).toEqual(new Uint8Array([77, 97]));
  });

  it("decodes base64 with double padding", () => {
    // "M" → TQ==
    const bytes = base64ToBytes("TQ==");
    expect(bytes).toEqual(new Uint8Array([77]));
  });

  it("decodes a longer string", () => {
    // "Hello" → SGVsbG8=
    const bytes = base64ToBytes("SGVsbG8=");
    expect(Array.from(bytes)).toEqual([72, 101, 108, 108, 111]);
  });

  it("strips non-base64 characters (e.g. newlines)", () => {
    const bytes = base64ToBytes("TW\nFu");
    expect(bytes).toEqual(new Uint8Array([77, 97, 110]));
  });
});

// ── getParentNode ─────────────────────────────────────────────────────────────

describe("getParentNode", () => {
  it("returns currentPage when no parentId given", async () => {
    const result = await getParentNode(undefined);
    expect(result).toBe(mockCurrentPage);
  });

  it("throws when parentId node is not found", async () => {
    await expect(getParentNode("1:999")).rejects.toThrow("Parent node not found: 1:999");
  });

  it("throws when found node cannot have children", async () => {
    mockGetNodeByIdAsync = async () => ({ id: "1:2", name: "rect" }); // no appendChild
    await expect(getParentNode("1:2")).rejects.toThrow("cannot have children");
  });

  it("returns node when it supports appendChild", async () => {
    const parentNode = { id: "1:3", name: "frame", appendChild: () => {} };
    mockGetNodeByIdAsync = async () => parentNode;
    const result = await getParentNode("1:3");
    expect(result).toBe(parentNode);
  });
});
