use app_log::LogLevel;
use dotenv::dotenv;
use log::*;
use serde::{Deserialize, Serialize};
use std::{env, fs};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub backend_bind: String, // 0.0.0.0:9000
    pub mcp_bind: String,     // 0.0.0.0:9001
    pub mcp_token: String,    // <TOKEN>
    pub log_level: LogLevel,  // Debug, Info, Warn, Error, Trace
    pub pg_connection: usize,
    pub redis_url: String,  // redis://127.0.0.1:6379
    pub redis_session: u64, // 10000
    pub jwt_access_key: String,
    pub jwt_access_session_minutes: i64,
    pub jwt_refresh_key: String,
    pub jwt_refresh_session_days: i64,
    pub rsa_private_key: String,
    pub rsa_public_key: String,
}

impl AppConfig {
    pub fn new() -> Self {
        dotenv().ok();
        match env::var("APP_CONFIG") {
            Err(e) => {
                debug!("{}", &e);
                panic!(
                    "Cannot locate config file; please set APP_CONFIG env variable! {}",
                    &e
                );
            }
            Ok(config_file_path) => match fs::File::open(config_file_path) {
                Err(e) => {
                    debug!("{}", &e);
                    panic!("Cannot read config file! {}", &e);
                }
                Ok(config_file) => match serde_json::from_reader(config_file) {
                    Err(e) => {
                        debug!("{}", &e);
                        panic!("Cannot parse json! {}", &e);
                    }
                    Ok(json) => return json,
                },
            },
        };
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn check_app_config() {
        let config = AppConfig::new();
        println!("{:#?}", config);
    }
}
