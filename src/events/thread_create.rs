use twilight_model::gateway::payload::incoming::ThreadCreate;

use crate::BotFramework;

pub async fn handle(framework: BotFramework, event: Box<ThreadCreate>) -> anyhow::Result<()> {
    if event.parent_id.is_none() && event.guild_id.is_none() {
        return Ok(()); // not a thread
    }

    if !event.newly_created.unwrap_or(false) {
        return Ok(()); // not a new thread
    }

    match framework.http_client().join_thread(event.id).await {
        Ok(_) => {}
        Err(e) => {
            tracing::error!("failed to join thread: {e:#?}");
        }
    }

    Ok(())
}
