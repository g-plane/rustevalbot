use crate::bot::Bot;
use crate::utils::encode_with_code;
use htmlescape::encode_minimal;
use itertools::Itertools;
use log::{debug, info, warn};
use reqwest::{Client, IntoUrl};
use serde::Deserialize;
use std::borrow::Cow;
use std::sync::Arc;
use telegram_types::bot::inline_mode::{
    InlineQueryResult, InlineQueryResultArticle, InputMessageContent, InputTextMessageContent,
    ResultId,
};
use telegram_types::bot::types::{
    InlineKeyboardButton, InlineKeyboardButtonPressed, InlineKeyboardMarkup, ParseMode,
    UpdateContent, UpdateId,
};
use url::Url;

pub struct CratesioBot {
    client: Client,
    bot: Bot,
}

impl CratesioBot {
    pub fn new(client: Client, bot: Bot) -> Self {
        info!("CratesioBot authorized as @{}", bot.username);
        CratesioBot { client, bot }
    }

    pub async fn handle_update(self: Arc<Self>, _: UpdateId, content: UpdateContent) {
        let query = match content {
            UpdateContent::InlineQuery(query) => query,
            _ => return,
        };
        let result = if query.query.is_empty() {
            self.generate_results("https://crates.io/api/v1/summary", |resp: Summary| {
                resp.most_recently_downloaded
            })
            .await
        } else {
            let mut url = Url::parse("https://crates.io/api/v1/crates").unwrap();
            url.query_pairs_mut()
                .append_pair("q", &query.query)
                .append_pair("sort", "relevance")
                .append_pair("per_page", "50");
            self.generate_results(url, |resp: Crates| resp.crates).await
        };
        let result = match result {
            Ok(result) => result,
            Err(e) => {
                warn!("failed to get results: {:?}", e);
                return;
            }
        };
        debug!("replying: {:?}", result);
        let result = self
            .bot
            .answer_inline_query(query.id, &result)
            .execute()
            .await;
        if let Err(e) = result {
            warn!("failed to answer query: {:?}", e);
        }
    }

    async fn generate_results<T>(
        &self,
        url: impl IntoUrl,
        get_crates: impl FnOnce(T) -> Vec<Crate>,
    ) -> Result<Vec<InlineQueryResult<'static>>, reqwest::Error>
    where
        for<'de> T: Deserialize<'de>,
    {
        let resp = self.client.get(url).send().await?;
        let resp = resp.error_for_status()?;
        let resp = resp.json().await?;
        let crates = get_crates(resp)
            .into_iter()
            .map(|c| c.into_inline_query_result())
            .collect();
        Ok(crates)
    }
}

#[derive(Debug, Deserialize)]
struct Summary {
    most_recently_downloaded: Vec<Crate>,
}

#[derive(Debug, Deserialize)]
struct Crates {
    crates: Vec<Crate>,
}

#[derive(Debug, Deserialize)]
struct Crate {
    id: String,
    name: String,
    description: Option<String>,
    max_version: String,
    documentation: Option<String>,
    repository: Option<String>,
}

impl Crate {
    fn into_inline_query_result(self) -> InlineQueryResult<'static> {
        let Crate {
            id,
            name,
            description,
            max_version,
            documentation,
            repository,
        } = self;

        let description: Option<Cow<'_, str>> =
            description.map(|d| d.split_whitespace().join(" ").into());
        let title = format!("{name} {max_version}");
        let mut message = format!(
            "<b>{}</b> ({})",
            encode_minimal(&name),
            encode_minimal(&max_version)
        );
        if let Some(description) = &description {
            message.push('\n');
            encode_with_code(&mut message, description);
        }

        // The name can only use alphanumeric characters or `-` and `_`, so no escape is needed.
        // See https://doc.rust-lang.org/cargo/reference/manifest.html#the-name-field
        let crate_url = format!("https://crates.io/crates/{name}");
        let lib_rs_url = format!("https://lib.rs/crates/{name}");
        let doc_url = documentation.unwrap_or_else(|| format!("https://docs.rs/crate/{name}"));
        let mut buttons = vec![
            InlineKeyboardButton {
                text: "info".to_string(),
                pressed: InlineKeyboardButtonPressed::Url(crate_url),
            },
            InlineKeyboardButton {
                text: "lib.rs".to_string(),
                pressed: InlineKeyboardButtonPressed::Url(lib_rs_url),
            },
            InlineKeyboardButton {
                text: "doc".to_string(),
                pressed: InlineKeyboardButtonPressed::Url(doc_url),
            },
        ];
        if let Some(repo) = repository {
            buttons.push(InlineKeyboardButton {
                text: "repo".to_string(),
                pressed: InlineKeyboardButtonPressed::Url(repo),
            });
        }

        InlineQueryResult::Article(InlineQueryResultArticle {
            id: ResultId(id),
            title: title.into(),
            input_message_content: InputMessageContent::Text(InputTextMessageContent {
                message_text: message.into(),
                parse_mode: Some(ParseMode::HTML),
                disable_web_page_preview: Some(true),
            }),
            reply_markup: Some(InlineKeyboardMarkup {
                inline_keyboard: vec![buttons],
            }),
            url: None,
            hide_url: None,
            description,
            thumb_url: None,
            thumb_width: None,
            thumb_height: None,
        })
    }
}
