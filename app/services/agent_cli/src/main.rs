use adk_core::{Content, ReadonlyContext, Toolset};
use adk_model::{OpenAIClient, OpenAIConfig};
//use adk_rust::Launcher;
use adk_runner::{Runner, RunnerConfig};
use adk_rust::prelude::*;
use adk_rust::session::{CreateRequest, SessionService};
use app_adk_utils::{
    content::SimpleContext,
    session::{postgres::PgSessionService, tools::AdkInjectSessionTool},
};
use app_config::AppConfig;
use dotenv::dotenv;
use futures::StreamExt;
use rmcp09::ServiceExt;
use rmcp09::transport::streamable_http_client::{
    StreamableHttpClientTransport, StreamableHttpClientTransportConfig,
};
use sqlx::PgPool;
use std::{
    collections::HashMap,
    env,
    io::{self, Write},
    sync::Arc,
};

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    // This should be determine from setting in real application
    let app_name = "app".to_string(); // Should be the same in session and runner
    let user_id = "user".to_string();
    let session_id = "console".to_string(); // This could be UUID

    let config = AppConfig::new();
    let db_url = env::var("DATABASE_URL")?;
    let pg_pool = PgPool::connect(&db_url).await?;

    // Model Configuration
    let model_config = OpenAIConfig {
        api_key: config.llm_token.clone(),
        model: config.llm_model.clone(),
        base_url: Some(config.llm_base_url.clone()),
        project_id: None,
        organization_id: None,
    };
    let model = OpenAIClient::new(model_config)?;

    // Launcher::new(agent).run().await?;
    // Or manually run it
    let create_req = CreateRequest {
        app_name: app_name.clone(),
        user_id: user_id.clone(),
        session_id: Some(session_id.clone()),
        state: HashMap::new(),
    };

    // Agent Session Manager
    // In memory session manager
    // let sessions = Arc::new(InMemorySessionService::new());
    // let _ = sessions.create(create_req).await?;
    let sessions = Arc::new(PgSessionService::new(pg_pool).await?);
    sessions.migrate().await?;
    sessions.create(create_req).await?;

    // MCP Tool Config
    let mcp_cfg = StreamableHttpClientTransportConfig::with_uri(config.mcp_base_url.clone())
        .auth_header(&config.mcp_token.clone());
    let transport = StreamableHttpClientTransport::from_config(mcp_cfg);
    let mcp_client = ().serve(transport).await?;
    let toolset = McpToolset::new(mcp_client);
    let _cancel = toolset.cancellation_token().await;
    let ctx: Arc<dyn ReadonlyContext> = Arc::new(SimpleContext {
        app_name: Some(app_name.clone()),
        ..Default::default()
    });
    let tools = toolset.tools(ctx).await?;

    // Agent Builder
    let mut builder = LlmAgentBuilder::new("Agent")
        .description("This is my first AI Agent")
        .instruction("You are a very frindly agent answering the questions")
        .model(Arc::new(model));
    // Add custom MCP tools to Agent
    for t in tools {
        builder = builder.tool(Arc::new(AdkInjectSessionTool::new(t)));
    }
    // Add Google Search Tools
    // Note: compatible with Gemini 2 models
    // Need export GOOGLE_API_KEY="YOUR_GOOGLE_KEY"
    builder = builder.tool(Arc::new(GoogleSearchTool::new()));
    let agent = Arc::new(builder.build()?);

    // Agent Runner
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
