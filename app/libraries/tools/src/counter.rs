#![allow(dead_code)]
use app_adk_utils::session::tools::AdkInjectContext;
use app_error::AppError;
use app_state::AppState;
use rmcp::{
    ErrorData as McpError, RoleServer,
    handler::server::{
        router::{prompt::PromptRouter, tool::ToolRouter},
        wrapper::{Json, Parameters},
    },
    model::*,
    prompt, prompt_router, schemars,
    service::RequestContext,
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::*;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct StructRequest {
    pub a: i32,
    pub b: i32,
}

#[derive(Clone)]
pub struct CounterTools {
    state: Arc<AppState>,
    counter: Arc<Mutex<i32>>,
    pub tool_router: ToolRouter<Self>,
    pub prompt_router: PromptRouter<Self>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct CounterResult {
    value: i32,
}

#[tool_router]
impl CounterTools {
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            state,
            counter: Arc::new(Mutex::new(0)),
            tool_router: Self::tool_router(),
            prompt_router: Self::prompt_router(),
        }
    }

    // Helper for resources (internal logic)
    pub fn _create_resource_text(&self, uri: &str, name: &str) -> Resource {
        RawResource::new(uri, name.to_string()).no_annotation()
    }

    #[tool(description = "Increment the counter by 1")]
    async fn increment(
        &self,
        ctx_req: RequestContext<RoleServer>,
    ) -> Result<Json<CounterResult>, McpError> {
        // This is example of how access to adk information in MCP
        let tool_name = ctx_req
            .meta
            .get("__adk_tool_name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let tool_args = ctx_req.meta.get("__adk_tool_args").cloned();
        let adk = AdkInjectContext::extract_adk_context(tool_args);
        debug!("tool_name={tool_name}");
        debug!("tool_args={adk:#?}");
        let mut counter = self.counter.lock().await;
        *counter += 1;
        Ok(Json(CounterResult { value: *counter }))
    }

    #[tool(description = "Get the current counter value")]
    async fn get_value(&self) -> Result<Json<CounterResult>, McpError> {
        let counter = self.counter.lock().await;
        Ok(Json(CounterResult { value: *counter }))
    }

    #[tool(description = "This only makes an error")]
    async fn make_error(&self) -> Result<Json<CounterResult>, McpError> {
        Err(AppError::internal("Just a simple error").into())
    }

    // Explicitly defining manual resource handlers here for main.rs to call
    pub async fn list_my_resources(&self) -> Vec<Resource> {
        vec![
            self._create_resource_text("str:////home/tony/temp/", "cwd"),
            self._create_resource_text("memo://insights", "memo-name"),
        ]
    }

    pub async fn read_my_resource(&self, uri: &str) -> Result<String, McpError> {
        match uri {
            "str:////home/tony/temp/" => {
                Ok(include_str!("../../../mcp_contents/test_content.txt").to_string())
            }
            "memo://insights" => Ok("Business Intelligence Memo...".to_string()),
            _ => Err(McpError::resource_not_found(
                "not found",
                Some(json!({ "uri": uri })),
            )),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct CounterAnalysisArgs {
    pub goal: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SummaryArgs {
    #[schemars(description = "Style of summary: 'brief' or 'verbose'")]
    pub style: Option<String>,
}

#[prompt_router]
impl CounterTools {
    #[prompt(name = "counter_analysis")]
    async fn counter_analysis(
        &self,
        Parameters(args): Parameters<CounterAnalysisArgs>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        let current = *self.counter.lock().await;
        Ok(GetPromptResult {
            description: Some("Analysis".into()),
            messages: vec![PromptMessage::new_text(
                PromptMessageRole::User,
                format!("Current: {}, Goal: {}", current, args.goal),
            )],
        })
    }

    #[prompt(
        name = "summarize_counter",
        description = "Returns a textual summary of the current counter state"
    )]
    async fn summarize_counter(
        &self,
        Parameters(args): Parameters<SummaryArgs>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        // Logic: Lock the counter and read the value
        let current_value = *self.counter.lock().await;

        // Logic: Handle the optional argument
        let style = args.style.as_deref().unwrap_or("brief");

        let summary_text = match style {
            "verbose" => format!(
                "SYSTEM STATUS REPORT:\nThe core counter variable is currently holding the integer value of {}.\nSystem is operational.",
                current_value
            ),
            _ => format!("The counter value is {}.", current_value), // brief default
        };

        Ok(GetPromptResult {
            description: Some("Counter State Summary".into()),
            messages: vec![PromptMessage::new_text(
                PromptMessageRole::User,
                summary_text,
            )],
        })
    }

    #[prompt(
        name = "summarize_content",
        description = "Returns a textual summary of the content"
    )]
    async fn summarize_content(
        &self,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        Ok(GetPromptResult {
            description: Some("Counter State Summary".into()),
            messages: vec![PromptMessage::new_text(
                PromptMessageRole::User,
                format!("Summarize the following content:"),
            )],
        })
    }
}
