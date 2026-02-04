use adk_core::{Content, ReadonlyContext, Toolset};
use adk_model::{OpenAIClient, OpenAIConfig};
use adk_rust::Launcher;
use adk_rust::prelude::*;
use app_config::AppConfig;
use rmcp09::ServiceExt;
use rmcp09::transport::streamable_http_client::{
    StreamableHttpClientTransport, StreamableHttpClientTransportConfig,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::new();
    // Tool Config
    let mcp_cfg = StreamableHttpClientTransportConfig::with_uri(config.mcp_base_url.clone())
        .auth_header(&config.mcp_token.clone());
    let transport = StreamableHttpClientTransport::from_config(mcp_cfg);
    let mcp_client = ().serve(transport).await?;
    let toolset = McpToolset::new(mcp_client);
    let _cancel = toolset.cancellation_token().await;
    let ctx: Arc<dyn ReadonlyContext> = Arc::new(SimpleContext);
    let tools = toolset.tools(ctx).await?;

    // Model Configuration
    let model_config = OpenAIConfig {
        api_key: config.llm_token.clone(),
        model: config.llm_model.clone(),
        base_url: Some(config.llm_base_url.clone()),
        project_id: None,
        organization_id: None,
    };
    let model = OpenAIClient::new(model_config)?;

    let mut builder = LlmAgentBuilder::new("Agent")
        .description("This is my first AI Agent")
        .instruction("You are a very frindly agent answering the questions")
        .model(Arc::new(model));
    // Add MCP tools
    for t in tools {
        builder = builder.tool(t);
    }
    // Add Google Search Tools
    // Note: compatible with Gemini 2 models
    // Need export GOOGLE_API_KEY="YOUR_GOOGLE_KEY"
    builder = builder.tool(Arc::new(GoogleSearchTool::new()));
    let agent = Arc::new(builder.build()?);

    Launcher::new(agent).run().await?;

    Ok(())
}

struct SimpleContext;

#[async_trait::async_trait]
impl ReadonlyContext for SimpleContext {
    fn invocation_id(&self) -> &str {
        "init"
    }
    fn agent_name(&self) -> &str {
        "assistant"
    }
    fn user_id(&self) -> &str {
        "user"
    }
    fn app_name(&self) -> &str {
        "app"
    }
    fn session_id(&self) -> &str {
        "init"
    }
    fn branch(&self) -> &str {
        "main"
    }
    fn user_content(&self) -> &Content {
        static CONTENT: std::sync::OnceLock<Content> = std::sync::OnceLock::new();
        CONTENT.get_or_init(|| Content::new("user").with_text(""))
    }
}
