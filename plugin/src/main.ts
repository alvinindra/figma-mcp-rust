// Plugin core — entry point, UI bootstrap, and request dispatch.

import { handleReadRequest } from "./read-handlers";
import { handleWriteRequest } from "./write-handlers";
import { clearStyleCache } from "./serializers";

// Reads that walk large parts of the tree. For these, skipping invisible nodes
// inside instances makes traversal several times faster (findAllWithCriteria up
// to hundreds of times). The flag is scoped to the request and reset afterwards:
// while it is on, getNodeByIdAsync returns null for those nodes and reading a
// property on one throws, which would break writes that target a hidden layer
// inside an instance.
const FAST_TRAVERSAL = new Set([
  "get_document",
  "get_design_context",
  "get_fonts",
  "get_annotations",
  "get_local_components",
  "search_nodes",
  "scan_text_nodes",
  "scan_nodes_by_types",
]);

const sendStatus = () => {
  figma.ui.postMessage({
    type: "plugin-status",
    payload: {
      fileName: figma.root.name,
      pageName: figma.currentPage.name,
      selectionCount: figma.currentPage.selection.length,
    },
  });
};

const handleRequest = async (request: any) => {
  clearStyleCache();
  // Restore rather than hardcode false: the flag defaults to true in Dev Mode.
  const prevSkip = figma.skipInvisibleInstanceChildren;
  const fast = FAST_TRAVERSAL.has(request.type);
  if (fast) figma.skipInvisibleInstanceChildren = true;
  try {
    const result =
      (await handleReadRequest(request)) ?? (await handleWriteRequest(request));
    if (result === null)
      throw new Error(`Unknown request type: ${request.type}`);
    return result;
  } catch (error) {
    return {
      type: request.type,
      requestId: request.requestId,
      error: error instanceof Error ? error.message : String(error),
    };
  } finally {
    if (fast) figma.skipInvisibleInstanceChildren = prevSkip;
  }
};

figma.showUI(__html__, { width: 320, height: 230 });
sendStatus();

figma.on("selectionchange", () => {
  sendStatus();
});

figma.on("currentpagechange", () => {
  sendStatus();
});

figma.ui.onmessage = async (message) => {
  if (message.type === "ui-ready") {
    sendStatus();
    return;
  }
  if (message.type === "get_ws_config") {
    const config = await figma.clientStorage.getAsync("ws_config");
    figma.ui.postMessage({
      type: "ws_config",
      host: config?.host ?? "127.0.0.1",
      port: config?.port ?? "1994",
    });
    return;
  }
  if (message.type === "save_ws_config") {
    await figma.clientStorage.setAsync("ws_config", {
      host: message.host,
      port: message.port,
    });
    return;
  }
  if (message.type === "server-request") {
    const response = await handleRequest(message.payload);
    try {
      figma.ui.postMessage(response);
    } catch (err) {
      figma.ui.postMessage({
        type: response.type,
        requestId: response.requestId,
        error: err instanceof Error ? err.message : String(err),
      });
    }
  }
};
