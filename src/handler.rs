//! MCP `ServerHandler` implementation that exposes the 73 Figma tools and 12 prompts.
//!
//! Tools and prompts are described declaratively in [`crate::tools::definitions`]
//! and [`crate::prompts`] respectively; this file just adapts them to the rmcp traits.

use std::borrow::Cow;
use std::sync::{Arc, OnceLock};

use rmcp::handler::server::ServerHandler;
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, ErrorData,
    GetPromptRequestParams, GetPromptResponse, GetPromptResult, Implementation, ListPromptsResult,
    ListToolsResult, PaginatedRequestParams, Prompt, PromptMessage, Role, ServerCapabilities,
    ServerInfo, Tool,
};
use rmcp::service::{NotificationContext, RequestContext, RoleServer};

use crate::node::Node;
use crate::prompts;
use crate::schema::validate_rpc;
use crate::tools::{self, extract_node_ids};

#[derive(Clone)]
pub struct Handler {
    pub node: Arc<Node>,
    pub version: String,
}

/// Shared reference to a Handler — passed into special-handler closures.
pub type HandlerArc = Arc<Handler>;

impl Handler {
    pub fn new(node: Arc<Node>, version: String) -> Self {
        Self { node, version }
    }
}

impl ServerHandler for Handler {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .build(),
        )
        .with_server_info(Implementation::new("figma-mcp-rust", self.version.clone()))
        .with_instructions(
            "Figma MCP server with full read/write access through a companion Figma plugin. \
             The plugin must be running in Figma Desktop (Plugins > Development > figma-mcp-rust) \
             with its window open; until it connects, tool calls fail with 'plugin not connected'. \
             Node IDs use colon format (e.g. 4029:12345). All write operations are undoable in Figma.",
        )
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult::with_all_items(tool_list().clone()))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let name = request.name.to_string();
        let def = match tools::find(&name) {
            Some(d) => d,
            None => return Ok(error_result(&format!("unknown tool: {name}"))),
        };

        let args = request.arguments.unwrap_or_default();

        // Special handlers do their own argument handling.
        if let Some(special) = def.special {
            let handler = Arc::new(self.clone());
            return match special(handler, args).await {
                Ok(text) => Ok(text_result(text)),
                Err(msg) => Ok(error_result(&msg)),
            };
        }

        // Generic pipeline: split into (nodeIDs, params), validate, forward to bridge.
        let (mut node_ids, mut params) = extract_node_ids(def.node_ids, args);
        for id in node_ids.iter_mut() {
            *id = crate::schema::normalize_node_id(id);
        }
        // Tools like scan_text_nodes/search_nodes carry the ID inside params —
        // normalize those too so hyphen-form IDs pass validation.
        for key in ["nodeId", "parentId"] {
            if let Some(serde_json::Value::String(s)) = params.get(key) {
                let normalized = crate::schema::normalize_node_id(s);
                params.insert(key.into(), serde_json::Value::String(normalized));
            }
        }

        if let Some(err) = validate_rpc(def.name, &node_ids, &params) {
            return Ok(error_result(&err));
        }

        match self.node.send(def.name, node_ids, params).await {
            Ok(resp) => {
                if !resp.error.is_empty() {
                    Ok(error_result(&resp.error))
                } else {
                    let data = resp.data.unwrap_or(serde_json::Value::Null);
                    let text = serde_json::to_string(&data)
                        .unwrap_or_else(|e| format!("marshal response: {e}"));
                    Ok(text_result(text))
                }
            }
            Err(e) => Ok(error_result(&e.to_string())),
        }
    }

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, ErrorData> {
        let prompts = prompts::all()
            .iter()
            .map(|p| Prompt::new(p.name, Some(p.description), None))
            .collect();
        Ok(ListPromptsResult::with_all_items(prompts))
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResponse, ErrorData> {
        let p = match prompts::find(&request.name) {
            Some(p) => p,
            None => {
                return Err(ErrorData::invalid_params(
                    format!("unknown prompt: {}", request.name),
                    None,
                ))
            }
        };
        Ok(
            GetPromptResult::new(vec![PromptMessage::new_text(Role::User, p.body)])
                .with_description(p.description)
                .into(),
        )
    }

    async fn on_initialized(&self, _context: NotificationContext<RoleServer>) {
        // No-op: client is ready.
    }
}

/// The 73 schemas are built from `json!` literals, so build them once and hand
/// out clones (name/description are borrowed, the schema is an `Arc`).
fn tool_list() -> &'static Vec<Tool> {
    static CACHE: OnceLock<Vec<Tool>> = OnceLock::new();
    CACHE.get_or_init(|| {
        tools::all()
            .iter()
            .map(|def| {
                Tool::new(
                    Cow::Borrowed(def.name),
                    Cow::Borrowed(def.description),
                    match (def.input_schema)() {
                        serde_json::Value::Object(m) => Arc::new(m),
                        _ => Arc::new(serde_json::Map::new()),
                    },
                )
            })
            .collect()
    })
}

fn text_result(text: String) -> CallToolResponse {
    CallToolResult::success(vec![ContentBlock::text(text)]).into()
}

fn error_result(msg: &str) -> CallToolResponse {
    CallToolResult::error(vec![ContentBlock::text(msg.to_string())]).into()
}
