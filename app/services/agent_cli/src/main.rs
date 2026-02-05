use adk_core::{Content, ReadonlyContext, Toolset};
use adk_model::{OpenAIClient, OpenAIConfig};
//use adk_rust::Launcher;
use adk_runner::{Runner, RunnerConfig};
use adk_rust::prelude::*;
use adk_rust::session::{CreateRequest, SessionService};
use app_config::AppConfig;
use futures::StreamExt;
use rmcp09::ServiceExt;
use rmcp09::transport::streamable_http_client::{
    StreamableHttpClientTransport, StreamableHttpClientTransportConfig,
};
use std::{
    collections::HashMap,
    io::{self, Write},
    sync::Arc,
};

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

    // Launcher::new(agent).run().await?;
    // Or manually run it
    let app_name = "app".to_string(); // Should be the same in session and runner
    let user_id = "user".to_string();
    let session_id = "console".to_string();

    let create_req = CreateRequest {
        app_name: app_name.clone(),
        user_id: user_id.clone(),
        session_id: Some(session_id.clone()),
        state: HashMap::new(),
    };

    // In memory session manager
    let sessions = Arc::new(InMemorySessionService::new());
    let _ = sessions.create(create_req).await?;

    let artifacts = Arc::new(InMemoryArtifactService::new());

    let runner = Runner::new(RunnerConfig {
        app_name: app_name.clone(),
        agent: agent,
        session_service: sessions.clone(),
        artifact_service: Some(artifacts),
        memory_service: None,
        run_config: None, // Uses default SSE streaming
    })?;
    let mut history: Vec<Event> = Vec::new();
    eprintln!(
        "Interactive console.\nexit\t\tQuit Agent\ndebug\t\tShow internal events\nclean\t\tClean internal events"
    );
    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut line = String::new();
        if io::stdin().read_line(&mut line)? == 0 {
            break;
        }
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }
        if line.eq_ignore_ascii_case("exit") {
            break;
        }
        if line.eq_ignore_ascii_case("debug") {
            for h in &history {
                println!("{:#?}", &h);
            }
            continue;
        }
        if line.eq_ignore_ascii_case("clean") {
            history = Vec::new();
            continue;
        }
        // Content is created like this (role + parts)
        let input = Content::new("user").with_text(line);

        // Run one turn; stream events
        let mut stream = runner
            .run(user_id.to_string(), session_id.to_string(), input)
            .await?;

        // Print only the final response content
        let mut buf = String::new();

        while let Some(ev) = stream.next().await {
            let ev = ev?;
            history.push(ev.clone());
            match &ev.content() {
                Some(ctx) => {
                    for part in ctx.parts.iter() {
                        match &part {
                            Part::Text { text } => buf.push_str(&text),
                            _ => (),
                        }
                    }
                }
                None => (),
            }
        }
        println!("{}", &buf);
    }
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
