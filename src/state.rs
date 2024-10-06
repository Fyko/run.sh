use std::sync::Arc;

use sqlx::SqlitePool;
use twilight_cache_inmemory::InMemoryCache;

use crate::hypervisor::Docker;

#[derive(Clone)]
pub struct BotState {
    pub cache: Arc<InMemoryCache>,
    pub docker: Arc<Docker>,
    pub db: SqlitePool,
}
