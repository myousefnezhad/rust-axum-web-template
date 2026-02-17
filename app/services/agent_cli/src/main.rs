use adk_core::{Content, ReadonlyContext, Toolset};
use adk_model::{OpenAIClient, OpenAIConfig};
//use adk_rust::Launcher;
use adk_runner::{Runner, RunnerConfig};
use adk_rust::prelude::*;
use adk_rust::session::{CreateRequest, SessionService};
use app_adk_session_manager::PgSessionService;
use app_config::AppConfig;
use async_trait::async_trait;
use dotenv::dotenv;
use futures::StreamExt;
use rmcp09::ServiceExt;
use rmcp09::transport::streamable_http_client::{
    StreamableHttpClientTransport, StreamableHttpClientTransportConfig,
};
use serde_json::{Map, Value, json};
use sqlx::PgPool;
use std::{
    collections::HashMap,
    env,
    io::{self, Write},
    sync::Arc,
};

pub struct InjectSessionTool {
    inner: Arc<dyn Tool>,
}

impl InjectSessionTool {
    pub fn new(inner: Arc<dyn Tool>) -> Self {
        Self { inner }
    }
}

fn ensure_object(args: Value) -> Map<String, Value> {
    match args {
        Value::Object(m) => m,
        other => {
            let mut m = Map::new();
            m.insert("input".to_string(), other);
            m
        }
    }
}

#[async_trait]
impl Tool for InjectSessionTool {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn description(&self) -> &str {
        self.inner.description()
    }

    async fn execute(&self, ctx: Arc<dyn ToolContext>, args: Value) -> Result<Value> {
        let mut obj = ensure_object(args);

        // Pull identity directly from ToolContext
        obj.insert(
            "_adk".to_string(),
            json!({
                "app_name": ctx.app_name(),
                "user_id": ctx.user_id(),
                "session_id": ctx.session_id(),
                "agent_name": ctx.agent_name(),
                "invocation_id": ctx.invocation_id(),
                "branch": ctx.branch(),
            }),
        );

        self.inner.execute(ctx, Value::Object(obj)).await
    }
}

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
    let ctx: Arc<dyn ReadonlyContext> = Arc::new(SimpleContext);
    let tools = toolset.tools(ctx).await?;

    // Agent Builder
    let mut builder = LlmAgentBuilder::new("Agent")
        .description("This is my first AI Agent")
        .instruction("You are a very frindly agent answering the questions")
        .model(Arc::new(model));
    // Add custom MCP tools to Agent
    for t in tools {
        builder = builder.tool(Arc::new(InjectSessionTool::new(t)));
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
