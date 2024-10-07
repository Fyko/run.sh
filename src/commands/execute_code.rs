use twilight_model::application::interaction::InteractionData;
use vesper::prelude::*;

use crate::{
    hypervisor::{format_output, languages::Languages},
    state::BotState,
};

use super::{defer_response, edit_response, text_response};

#[command(message, name = "Execute Code")]
#[description = "Execute code in a sandboxed environment"]
pub async fn execute_code(ctx: &SlashContext<'_, BotState>) -> DefaultCommandResult {
    let data = match &ctx.interaction.data {
        Some(InteractionData::ApplicationCommand(data)) => data,
        _ => return Ok(()),
    };
    let (_, message) = data
        .resolved
        .clone() // ouch
        .unwrap()
        .messages
        .into_iter()
        .next()
        .unwrap();

    let Some(code) = crate::parsers::match_code(&message.content, true) else {
        return text_response(ctx, "Code input could not be parsed.".to_string(), true).await;
    };
    let Some(language) = Languages::from_codeblock_language(code.language) else {
        if let Err(e) = text_response(
            ctx,
            format!(
                "Unsupported language `{language}`",
                language = code.language
            ),
            true,
        )
        .await
        {
            tracing::error!("failed to reply to interaction - {e}");
        }

        return Ok(());
    };
    if !language.enabled() {
        if let Err(e) = text_response(
            ctx,
            format!(
                "Unsupported language `{language}`",
                language = code.language
            ),
            true,
        )
        .await
        {
            tracing::error!("failed to reply to message - {e}");
        }

        return Ok(());
    }

    defer_response(ctx).await?;

    let code_result = match ctx.data.hypervisor.exec(&language, code.code).await {
        Ok(res) => res,
        Err(e) => {
            tracing::error!("failed to execute code - {e:#?}");

            if let Err(e) = edit_response(ctx, format!("Failed to execute code: {e}")).await {
                tracing::error!("failed to reply to interaction - {e}");
            }

            return Ok(());
        }
    };

    let out = format_output(code_result);

    if let Err(e) = edit_response(ctx, format!("```{language}\n{out}\n```")).await {
        tracing::error!("failed to reply to interaction - {e}");
        return Ok(());
    };

    let channel_id = message.channel_id.to_string();
    let message_id = message.id.to_string();
    let language = language.to_string();
    sqlx::query!(
        r#"insert into execution (channel_id, message_id, language, reply_id) values ($1, $2, $3, 'interaction') returning *;"#,
        channel_id,
        message_id,
        language,
    )
    .fetch_one(&ctx.data.db)
    .await?;

    Ok(())
}
