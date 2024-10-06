use std::sync::Arc;

use sqlx::PgPool;
use twilight_cache_inmemory::InMemoryCache;

use crate::hypervisor::Hypervisor;

#[derive(Clone)]
pub struct BotState {
    pub cache: Arc<InMemoryCache>,
    pub hypervisor: Arc<Hypervisor>,
    pub db: PgPool,
}
