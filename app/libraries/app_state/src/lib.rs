use app_config::AppConfig;
use deadpool_redis::Pool as RedisPool;
use sqlx::Pool as PostgresPool;
use sqlx::postgres::Postgres;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub pg: PostgresPool<Postgres>,
    pub redis: RedisPool,
}
