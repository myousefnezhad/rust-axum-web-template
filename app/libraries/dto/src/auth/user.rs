use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct PostUserInput {
    pub name: String,
    pub email: String,
    pub password: String,
}
