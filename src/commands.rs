use twilight_model::{
    channel::message::MessageFlags,
    http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType},
};
use vesper::prelude::*;

use crate::state::BotState;

pub mod execute_code;
pub mod languages;

pub async fn text_response(
    ctx: &SlashContext<'_, BotState>,
    text: String,
    ephemeral: bool,
) -> DefaultCommandResult {
    ctx.interaction_client
        .create_response(
            ctx.interaction.id,
            &ctx.interaction.token,
            &InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(InteractionResponseData {
                    content: Some(text),
                    flags: if ephemeral {
                        Some(MessageFlags::EPHEMERAL)
                    } else {
                        None
                    },
                    ..Default::default()
                }),
            },
        )
        .await?;

    Ok(())
}

pub async fn defer_response(ctx: &SlashContext<'_, BotState>) -> anyhow::Result<()> {
    ctx.interaction_client
        .create_response(
            ctx.interaction.id,
            &ctx.interaction.token,
            &InteractionResponse {
                kind: InteractionResponseType::DeferredChannelMessageWithSource,
                data: Some(InteractionResponseData {
                    content: None,
                    flags: None,
                    ..Default::default()
                }),
            },
        )
        .await?;

    Ok(())
}

pub async fn edit_response(ctx: &SlashContext<'_, BotState>, text: String) -> DefaultCommandResult {
    ctx.interaction_client
        .update_response(&ctx.interaction.token)
        .content(Some(&text))
        .await?;

    Ok(())
}
