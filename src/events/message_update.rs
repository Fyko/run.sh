use std::str::FromStr;

use twilight_model::{
    gateway::payload::incoming::MessageUpdate,
    id::{marker::MessageMarker, Id},
};

use crate::{hypervisor::languages::Languages, BotFramework};

pub async fn handle(framework: BotFramework, message: Box<MessageUpdate>) -> anyhow::Result<()> {
    let message_id = message.id.to_string();
    let Some(existing_execution) =
        sqlx::query!("select * from execution where message_id = ?1", message_id)
            .fetch_optional(&framework.data.db)
            .await?
    else {
        return Ok(());
    };
    let content = message.content.unwrap();

    let Some(code) = crate::parsers::match_code(&content, false) else {
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
            ))?
            .await
        {
            tracing::error!("failed to reply to message - {e}");
        }

        return Ok(());
    };
    tracing::info!("matched language: {language:#?}");

    let res = match framework.data.docker.exec(&language, code.code).await {
        Ok(res) => res,
        Err(e) => {
            if let Err(e) = framework
                .http_client()
                .create_message(message.channel_id)
                .reply(message.id)
                .content(&format!("Failed to execute code: {e}"))?
                .await
            {
                tracing::error!("failed to reply to message - {e}");
            }

            return Ok(());
        }
    };

    let out = String::from_utf8_lossy(&res);
    let reply_id = Id::<MessageMarker>::from_str(&existing_execution.reply_id).unwrap();
    if let Err(e) = framework
        .http_client()
        .update_message(message.channel_id, reply_id)
        .content(Some(&format!(
            "```{language}\n{out}\n```\n-# ℹ️ Edit your message and the output will update"
        )))?
        .await
    {
        tracing::error!("failed to edit message - {e}");
    }

    Ok(())
}
