use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct PostUserInput {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PatchChangePasswordInput {
    pub email: String,
    pub password: String,
    pub new_password: String,
}
