import { describe, it, expect, beforeEach } from "bun:test";
import { handleWritePageRequest } from "./write-page";

// ── Figma global mock ─────────────────────────────────────────────────────────

let mockNodes: Record<string, any>;
let commitUndoCalled: boolean;
let mockPages: any[];

const makeRequest = (type: string, nodeIds?: string[], params?: any) => ({
  type,
  requestId: "req-test-1",
  nodeIds: nodeIds ?? [],
  params: params ?? {},
});

beforeEach(() => {
  commitUndoCalled = false;
  mockNodes = {};
  mockPages = [{ id: "0:1", name: "Page 1", type: "PAGE", remove: () => { mockPages.splice(mockPages.indexOf(page1), 1); } }];
  const page1 = mockPages[0];
  (globalThis as any).figma = {
    createPage: () => {
      const page: any = {
        id: `page:${Date.now()}`,
        name: "Page",
        type: "PAGE",
        remove() { mockPages.splice(mockPages.indexOf(this), 1); },
      };
      mockPages.push(page);
      return page;
    },
    getNodeByIdAsync: async (id: string) => mockNodes[id] ?? null,
    commitUndo: () => { commitUndoCalled = true; },
    root: {
      get children() { return mockPages; },
      insertChild(index: number, page: any) {
        const i = mockPages.indexOf(page);
        if (i !== -1) mockPages.splice(i, 1);
        mockPages.splice(index, 0, page);
      },
    },
  };
});

// ── add_page ──────────────────────────────────────────────────────────────────

describe("add_page", () => {
  it("creates a new page with name", async () => {
    const res = await handleWritePageRequest(makeRequest("add_page", [], { name: "Flows" }));
    expect(res?.data.name).toBe("Flows");
    expect(mockPages).toHaveLength(2);
    expect(commitUndoCalled).toBe(true);
  });

  it("creates page with default name when none provided", async () => {
    const res = await handleWritePageRequest(makeRequest("add_page", [], {}));
    expect(res?.data.id).toBeDefined();
    expect(mockPages).toHaveLength(2);
  });

  it("inserts page at specified index", async () => {
    // add a second page first
    mockPages.push({ id: "0:2", name: "Page 2", type: "PAGE", remove: () => {} });
    const res = await handleWritePageRequest(makeRequest("add_page", [], { name: "Inserted", index: 0 }));
    expect(mockPages[0].name).toBe("Inserted");
  });

  it("returns page id, name, and index", async () => {
    const res = await handleWritePageRequest(makeRequest("add_page", [], { name: "New Page" }));
    expect(res?.data.id).toBeDefined();
    expect(res?.data.name).toBe("New Page");
    expect(typeof res?.data.index).toBe("number");
  });
});

// ── delete_page ───────────────────────────────────────────────────────────────

describe("delete_page", () => {
  it("deletes page by pageId", async () => {
    const page2: any = { id: "0:2", name: "Page 2", type: "PAGE", remove() { mockPages.splice(mockPages.indexOf(this), 1); } };
    mockPages.push(page2);
    mockNodes["0:2"] = page2;
    const res = await handleWritePageRequest(makeRequest("delete_page", [], { pageId: "0:2" }));
    expect(res?.data.deleted).toBe(true);
    expect(mockPages).toHaveLength(1);
    expect(commitUndoCalled).toBe(true);
  });

  it("deletes page by pageName", async () => {
    const page2: any = { id: "0:2", name: "Flows", type: "PAGE", remove() { mockPages.splice(mockPages.indexOf(this), 1); } };
    mockPages.push(page2);
    const res = await handleWritePageRequest(makeRequest("delete_page", [], { pageName: "Flows" }));
    expect(res?.data.deleted).toBe(true);
    expect(mockPages).toHaveLength(1);
  });

  it("throws when trying to delete the only page", async () => {
    mockNodes["0:1"] = mockPages[0];
    await expect(handleWritePageRequest(makeRequest("delete_page", [], { pageId: "0:1" }))).rejects.toThrow("only page");
  });

  it("throws when page not found by id", async () => {
    mockPages.push({ id: "0:2", name: "P2", type: "PAGE" });
    await expect(handleWritePageRequest(makeRequest("delete_page", [], { pageId: "9:9" }))).rejects.toThrow("Page not found");
  });

  it("throws when page not found by name", async () => {
    await expect(handleWritePageRequest(makeRequest("delete_page", [], { pageName: "NonExistent" }))).rejects.toThrow("Page not found");
  });

  it("throws when neither pageId nor pageName given", async () => {
    await expect(handleWritePageRequest(makeRequest("delete_page", [], {}))).rejects.toThrow("pageId or pageName is required");
  });

  it("throws when node is not a PAGE", async () => {
    mockNodes["1:1"] = { id: "1:1", type: "FRAME" };
    mockPages.push({ id: "0:2", name: "P2" });
    await expect(handleWritePageRequest(makeRequest("delete_page", [], { pageId: "1:1" }))).rejects.toThrow("is not a PAGE");
  });
});

// ── rename_page ───────────────────────────────────────────────────────────────

describe("rename_page", () => {
  it("renames page by pageId", async () => {
    mockNodes["0:1"] = mockPages[0];
    const res = await handleWritePageRequest(makeRequest("rename_page", [], { pageId: "0:1", newName: "Redesign" }));
    expect(mockPages[0].name).toBe("Redesign");
    expect(res?.data.name).toBe("Redesign");
    expect(commitUndoCalled).toBe(true);
  });

  it("renames page by pageName", async () => {
    const res = await handleWritePageRequest(makeRequest("rename_page", [], { pageName: "Page 1", newName: "Updated" }));
    expect(mockPages[0].name).toBe("Updated");
    expect(res?.data.name).toBe("Updated");
  });

  it("returns oldName and new name", async () => {
    mockNodes["0:1"] = mockPages[0];
    const res = await handleWritePageRequest(makeRequest("rename_page", [], { pageId: "0:1", newName: "New Name" }));
    expect(res?.data.oldName).toBe("Page 1");
    expect(res?.data.name).toBe("New Name");
  });

  it("throws when newName is missing", async () => {
    mockNodes["0:1"] = mockPages[0];
    await expect(handleWritePageRequest(makeRequest("rename_page", [], { pageId: "0:1" }))).rejects.toThrow("newName is required");
  });

  it("throws when page not found by id", async () => {
    await expect(handleWritePageRequest(makeRequest("rename_page", [], { pageId: "9:9", newName: "X" }))).rejects.toThrow("Page not found");
  });

  it("throws when page not found by name", async () => {
    await expect(handleWritePageRequest(makeRequest("rename_page", [], { pageName: "Ghost", newName: "X" }))).rejects.toThrow("Page not found");
  });

  it("throws when neither pageId nor pageName given", async () => {
    await expect(handleWritePageRequest(makeRequest("rename_page", [], { newName: "X" }))).rejects.toThrow("pageId or pageName is required");
  });
});

// ── unknown type ──────────────────────────────────────────────────────────────

describe("handleWritePageRequest unknown", () => {
  it("returns null for unrecognised type", async () => {
    const res = await handleWritePageRequest(makeRequest("unknown_page_op"));
    expect(res).toBeNull();
  });
});
