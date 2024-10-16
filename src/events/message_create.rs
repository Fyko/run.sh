use twilight_model::gateway::payload::incoming::MessageCreate;

use crate::{
    hypervisor::{format_output, languages::Languages},
    BotFramework,
};

pub async fn handle(framework: BotFramework, event: Box<MessageCreate>) -> anyhow::Result<()> {
    let message = event.0;
    if message.author.bot {
        return Ok(());
    }

    // todo: blocklist

    let Some(code) = crate::parsers::match_code(&message.content, false) else {
        return Ok(());
    };
    tracing::info!("matched code: {code:#?}");
    let Some(language) = Languages::from_codeblock_language(code.language) else {
        if let Err(e) = framework
            .http_client()
            .create_message(message.channel_id)
            .reply(message.id)
            .content(&format!(
                "Unsupported language `{language}`",
                language = code.language
            ))
            .await
        {
            tracing::error!("failed to reply to message - {e}");
        }

        return Ok(());
    };
    if !language.enabled() {
        if let Err(e) = framework
            .http_client()
            .create_message(message.channel_id)
            .reply(message.id)
            .content(&format!(
                "Unsupported language `{language}`",
                language = code.language
            ))
            .await
        {
            tracing::error!("failed to reply to message - {e}");
        }

        return Ok(());
    }

    let _ = framework
        .http_client()
        .create_typing_trigger(message.channel_id)
        .await?;

    let code_result = match framework.data.hypervisor.exec(&language, code.code).await {
        Ok(res) => res,
        Err(e) => {
            tracing::error!("failed to execute code - {e:#?}");

            if let Err(e) = framework
                .http_client()
                .create_message(message.channel_id)
                .reply(message.id)
                .content(&format!("Failed to execute code: {e}"))
                .await
            {
                tracing::error!("failed to reply to message - {e}");
            }

            return Ok(());
        }
    };

    let out = format_output(code_result);

    let res = match framework
        .http_client()
        .create_message(message.channel_id)
        .reply(message.id)
        .content(&format!(
            "```{language}\n{out}\n```\n-# ℹ️ Edit your message and the output will update"
        ))
        .await
    {
        Ok(res) => res.model().await?,
        Err(e) => {
            tracing::error!("failed to reply to message - {e}");
            return Ok(());
        }
    };

    let channel_id = message.channel_id.to_string();
    let message_id = message.id.to_string();
    let language = language.to_string();
    let reply_id = res.id.to_string();
    sqlx::query!(
        "insert into execution (channel_id, message_id, language, reply_id) values ($1, $2, $3, $4) returning *;",
        channel_id,
        message_id,
        language,
        reply_id
    )
    .fetch_one(&framework.data.db)
    .await?;

    Ok(())
}
