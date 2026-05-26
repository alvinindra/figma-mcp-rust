//! MCP `ServerHandler` implementation that exposes the 73 Figma tools and 12 prompts.
//!
//! Tools and prompts are described declaratively in [`crate::tools::definitions`]
//! and [`crate::prompts`] respectively; this file just adapts them to the rmcp traits.

use std::borrow::Cow;
use std::sync::Arc;

use rmcp::handler::server::ServerHandler;
use rmcp::model::{
    CallToolRequestParam, CallToolResult, Content, ErrorData, GetPromptRequestParam,
    GetPromptResult, Implementation, ListPromptsResult, ListToolsResult, PaginatedRequestParam,
    Prompt, PromptMessage, PromptMessageContent, PromptMessageRole, ProtocolVersion,
    ServerCapabilities, ServerInfo, Tool,
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
        ServerInfo {
            protocol_version: ProtocolVersion::default(),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .build(),
            server_info: Implementation {
                name: "figma-mcp-rust".into(),
                version: self.version.clone(),
            },
            instructions: Some("Figma MCP server with full read/write access via plugin.".into()),
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let tools = tools::all()
            .iter()
            .map(|def| {
                let schema = (def.input_schema)();
                let input_schema = match schema {
                    serde_json::Value::Object(m) => Arc::new(m),
                    _ => Arc::new(serde_json::Map::new()),
                };
                Tool {
                    name: Cow::Borrowed(def.name),
                    description: Some(Cow::Borrowed(def.description)),
                    input_schema,
                    annotations: None,
                }
            })
            .collect();
        Ok(ListToolsResult {
            tools,
            next_cursor: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
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
        let (mut node_ids, params) = extract_node_ids(def.node_ids, args);
        for id in node_ids.iter_mut() {
            *id = crate::schema::normalize_node_id(id);
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
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, ErrorData> {
        let prompts = prompts::all()
            .iter()
            .map(|p| Prompt {
                name: p.name.to_string(),
                description: Some(p.description.to_string()),
                arguments: None,
            })
            .collect();
        Ok(ListPromptsResult {
            prompts,
            next_cursor: None,
        })
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, ErrorData> {
        let p = match prompts::find(&request.name) {
            Some(p) => p,
            None => {
                return Err(ErrorData::invalid_params(
                    format!("unknown prompt: {}", request.name),
                    None,
                ))
            }
        };
        Ok(GetPromptResult {
            description: Some(p.description.to_string()),
            messages: vec![PromptMessage {
                role: PromptMessageRole::User,
                content: PromptMessageContent::Text {
                    text: p.body.to_string(),
                },
            }],
        })
    }

    async fn on_initialized(&self, _context: NotificationContext<RoleServer>) {
        // No-op: client is ready.
    }
}

fn text_result(text: String) -> CallToolResult {
    CallToolResult {
        content: vec![Content::text(text)],
        is_error: Some(false),
    }
}

fn error_result(msg: &str) -> CallToolResult {
    CallToolResult {
        content: vec![Content::text(msg.to_string())],
        is_error: Some(true),
    }
}
