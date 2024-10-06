use std::sync::Arc;

use state::BotState;
use vesper::prelude::Framework;

pub mod commands;
pub mod config;
pub mod events;
pub mod hypervisor;
pub mod parsers;
pub mod state;

pub type BotFramework = Arc<Framework<BotState>>;
