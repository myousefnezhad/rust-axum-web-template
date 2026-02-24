use adk_core::Content;
use adk_rust::prelude::Part;
use adk_rust::session::{CreateRequest, GetRequest};
use app_error::AppError;
use app_state::AppState;
use askama::Template;
use axum::extract::{Json, State};
use axum::response::Html;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Template)]
#[template(path = "agent.html")]
struct AgentPage;

pub async fn get_agent() -> Result<Html<String>, AppError> {
    let page = AgentPage;
    Ok(Html(page.render()?))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatPostInput {
    pub session_id: Option<String>,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatPostOutput {
    pub session_id: String,
    pub content: String,
}

pub async fn post_agent(
    State(state): State<Arc<AppState>>,
    Json(args): Json<ChatPostInput>,
) -> Result<Json<ChatPostOutput>, AppError> {
    let agent_runner = match &state.agent_runner {
        None => {
            return Err(AppError::internal("Cannot find agent runner"));
        }
        Some(runner) => runner.clone(),
    };
    let agent_session = match &state.agent_session {
        None => {
            return Err(AppError::internal("Cannot find agent session"));
        }
        Some(session) => session.clone(),
    };
    let agent_app_id = match &state.agent_app_id {
        None => {
            return Err(AppError::internal("Cannot find agent app id"));
        }
        Some(id) => id.clone(),
    };

    let new_session_id = Uuid::new_v4().to_string();
    let agent_current_session = match &args.session_id {
        // Make a new session
        None => match agent_session
            .create(CreateRequest {
                app_name: agent_app_id.clone(),
                user_id: "default".to_string(),
                session_id: Some(new_session_id.clone()),
                state: HashMap::new(),
            })
            .await
        {
            Err(e) => {
                return Err(AppError::internal(&format!("{}", &e)));
            }
            Ok(_) => new_session_id,
        },
        // Find Exising Session
        Some(session_id) => match agent_session
            .get(GetRequest {
                app_name: agent_app_id.clone(),
                user_id: "default".to_string(),
                session_id: session_id.clone(),
                num_recent_events: None,
                after: None,
            })
            .await
        {
            Ok(_) => session_id.clone(),
            // Make a new Session if session not found
            Err(_) => match agent_session
                .create(CreateRequest {
                    app_name: agent_app_id.clone(),
                    user_id: "default".to_string(),
                    session_id: Some(new_session_id.clone()),
                    state: HashMap::new(),
                })
                .await
            {
                Err(e) => {
                    return Err(AppError::internal(&format!("{}", &e)));
                }
                Ok(_) => session_id.clone(),
            },
        },
    };
    // Making Agent Runner
    let user_input = Content::new("user").with_text(args.content);
    let stream = match agent_runner
        .run(
            "default".to_owned(),
            agent_current_session.clone(),
            user_input,
        )
        .await
    {
        Ok(s) => s,
        Err(e) => return Err(AppError::internal(&format!("{}", &e))),
    };
    // Generating Answer
    let mut answer_buf = String::new();
    let mut stream = stream;
    while let Some(ev) = stream.next().await {
        let ev = match ev {
            Err(e) => return Err(AppError::internal(&format!("{}", &e))),
            Ok(e) => e,
        };
        match &ev.content() {
            Some(ctx) => {
                for part in ctx.parts.iter() {
                    match &part {
                        Part::Text { text } => answer_buf.push_str(&text),
                        _ => (),
                    }
                }
            }
            None => (),
        }
    }

    Ok(Json(ChatPostOutput {
        session_id: agent_current_session.clone(),
        content: answer_buf,
    }))
}
