use reqwest::{Client, Error};
use serde::Serialize;

pub struct DiscordClient<'a> {
    client: &'a Client,
}

#[derive(Serialize, Debug)]
pub struct WebhookMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub embeds: Option<Vec<Embed>>,
}

#[derive(Serialize, Debug)]
pub struct Image {
    pub url: String,
}

#[derive(Serialize, Debug)]
pub struct Author {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct Embed {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<Image>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<Author>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<u8>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    #[serde(rename = "type")]
    pub embed_type: String,

    pub timestamp: Option<String>,
}

impl DiscordClient<'_> {
    pub fn new(http_client: &Client) -> DiscordClient<'_> {
        DiscordClient {
            client: http_client,
        }
    }

    pub async fn send(&self, webhook_url: &str, message: WebhookMessage) -> Result<(), Error> {
        self.client
            .post(webhook_url)
            .json(&message)
            .send()
            .await
            .map(|_| ())
    }
}
