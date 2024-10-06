use twilight_model::{
    application::interaction::{InteractionData, InteractionType},
    gateway::payload::incoming::InteractionCreate,
};

use crate::BotFramework;

pub async fn handle(framework: BotFramework, event: Box<InteractionCreate>) -> anyhow::Result<()> {
    let interaction = event.0;

    match interaction.kind {
        InteractionType::ApplicationCommand => match interaction.data {
            Some(InteractionData::ApplicationCommand(_)) => {
                framework.process(interaction).await; // todo: handle error

                Ok(())
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}
