use std::sync::Arc;

use sqlx::PgPool;

use crate::hypervisor::Hypervisor;

#[derive(Clone)]
pub struct BotState {
    pub hypervisor: Arc<Hypervisor>,
    pub db: PgPool,
}
