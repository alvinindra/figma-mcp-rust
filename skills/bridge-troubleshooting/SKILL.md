---
name: bridge-troubleshooting
description: Diagnose figma-mcp-rust connection problems. Use when tools fail with "plugin not connected", requests time out, the port is already in use, or the Figma plugin cannot reach the MCP server. Covers installing the companion Figma plugin, port 1994, and running multiple MCP clients.
---

# Bridge Troubleshooting

figma-mcp-rust talks to Figma through a companion Figma plugin over a local
websocket (default ws://127.0.0.1:1994). Every tool call needs that plugin
running in Figma Desktop. Use this checklist when calls fail.

## Error: "plugin not connected"

The MCP server is running but no Figma plugin is attached.

1. Open Figma Desktop (not the browser) and open the file to work on.
2. If the plugin is not installed yet:
   - Download plugin.zip from https://github.com/alvinindra/figma-mcp-rust/releases
   - In Figma Desktop: Plugins, then Development, then "Import plugin from manifest"
   - Select manifest.json from the extracted plugin.zip
3. Run the plugin (Plugins, Development, figma-mcp-rust) and keep its window open.
4. Confirm the plugin UI shows a connected state, then retry the tool call.

The plugin window must stay open; closing it disconnects the bridge.

## Error: timeout

The plugin is connected but the operation took too long (30s default, 60s for
get_document).

- Very large pages: prefer get_design_context with detail="minimal" or "compact"
  and a small depth instead of get_document.
- Scan tools (scan_text_nodes, scan_nodes_by_types) on huge trees: target a
  smaller subtree nodeId.
- If Figma itself is frozen or showing a dialog, dismiss it and retry.

## Error: "port 1994 already in use"

Another process holds the port and is not a healthy figma-mcp-rust leader.

- If it is another figma-mcp-rust instance, that is normal: instances elect a
  leader on the port and the rest follow it automatically. No action needed.
- If an unrelated app owns the port, start the server with --port <other> and
  update the host/port in the Figma plugin UI to match.

## Multiple MCP clients (Claude Code + Cursor, etc.)

Running several clients at once is supported. The first server process binds
port 1994 and becomes the leader that owns the Figma websocket; later
processes detect the healthy leader and forward their requests to it. If the
leader exits, a follower takes over the port on its next attempt.

## Quick checklist

1. Figma Desktop open, target file open.
2. Companion plugin imported and running, window open.
3. Plugin host/port matches the server (default 127.0.0.1:1994).
4. Retry the failed tool call; write operations are undoable with Ctrl/Cmd+Z.
