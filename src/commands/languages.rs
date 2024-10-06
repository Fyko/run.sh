use vesper::prelude::*;

use crate::{config::CONFIG, hypervisor::languages::LANGUAGES, state::BotState};

use super::text_response;

#[command(name = "languages")]
#[description = "Lists all available languages"]
pub async fn languages(ctx: &SlashContext<'_, BotState>) -> DefaultCommandResult {
    let enabled = join_vec_with_and(
        CONFIG
            .languages
            .clone()
            .iter()
            .map(|l| format!("`{l}`"))
            .collect(),
    );

    let mut disabled = vec![];
    for language in LANGUAGES {
        if !CONFIG.languages.contains(&(*language).to_string()) {
            disabled.push((*language).to_string());
        }
    }
    let disabled = join_vec_with_and(disabled.iter().map(|l| format!("`{l}`")).collect());

    let content = indoc::formatdoc! {r#"
		Enabled languages: {enabled}
		Disabled languages: {disabled}
		-# Don't see your language here? Consider making a [Feature Request](<https://github.com/Fyko/run.sh/issues/new?assignees=&labels=language+request&projects=&template=language_request.yml&title=request%3A+>)
	"#};

    text_response(ctx, content, false).await
}

/// Joins a vector of strings with commas and an "and" at the end
fn join_vec_with_and(items: Vec<String>) -> String {
    match items.len() {
        0 => String::new(),
        1 => items[0].clone(),
        2 => format!("{} and {}", items[0], items[1]),
        _ => {
            let mut result = items[..items.len() - 1].join(", ");
            result.push_str(", and ");
            result.push_str(&items[items.len() - 1]);
            result
        }
    }
}
