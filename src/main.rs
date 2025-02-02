use anilist::{Activity, AnilistClient};
use config::Config;
use datastore::Datastore;
use discord::{Author, DiscordClient, Embed, WebhookMessage};
use reqwest::Client;
use shuttle_runtime::{tokio::time::sleep, SecretStore};
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

mod anilist;
mod config;
mod datastore;
mod discord;

fn format_discord_message(activity: &Activity) -> WebhookMessage {
    let description = match activity.status.as_str() {
        "completed" => format!(
            "completed [{}]({})",
            &activity.media.title, &activity.media.site_url
        ),
        "plans to watch" => format!(
            "plans to watch [{}]({})",
            &activity.media.title, &activity.media.site_url
        ),
        "plans to read" => format!(
            "plans to read [{}]({})",
            &activity.media.title, &activity.media.site_url
        ),
        _ => format!(
            "{} {} of [{}]({})",
            &activity.status,
            &activity.progress.as_deref().unwrap_or("?"),
            &activity.media.title,
            &activity.media.site_url
        ),
    };

    let timestamp =
        chrono::DateTime::from_timestamp(activity.created_at as i64, 0).map(|ts| ts.to_rfc3339());

    let username = activity.user.name.as_deref().unwrap_or("?");

    let embed = Embed {
        color: activity
            .media
            .cover_image
            .as_ref()
            .and_then(|i| {
                i.color
                    .as_ref()
                    .map(|c| u8::from_str_radix(c.trim_start_matches('#'), 16).ok())
            })
            .flatten(),

        title: Some(activity.media.title.clone()),

        author: Some(Author {
            name: username.to_owned(),
            icon_url: activity.user.avatar.clone(),
        }),

        description: Some(description),

        image: activity
            .media
            .banner_image
            .as_ref()
            .map(|i| discord::Image { url: i.clone() }),

        thumbnail: None,
        url: Some(activity.media.site_url.clone()),
        embed_type: "rich".to_string(),

        timestamp,
    };

    WebhookMessage {
        content: None,
        username: activity.user.name.clone(),
        avatar_url: activity.user.avatar.clone(),
        embeds: Some(vec![embed]),
    }
}

struct Service {
    store: Datastore,
    config: Config,
}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for Service {
    async fn bind(self, _addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        let http = Client::new();
        let anilist = AnilistClient::new(&http);
        let discord = DiscordClient::new(&http);

        let config = &self.config;

        loop {
            let last_published_timestamp =
                self.store.get_last_published_timestamp().await.unwrap_or(0);

            let activities = anilist
                .fetch_activities(&config.user_ids, Some(last_published_timestamp))
                .await
                .unwrap();

            for activity in activities.iter().rev() {
                discord
                    .send(&config.webhook_url, format_discord_message(activity))
                    .await
                    .unwrap();
            }

            if let Some(activity) = activities.get(0) {
                self.store
                    .set_last_published_timestamp(activity.created_at)
                    .await
                    .unwrap();
            }

            sleep(Duration::from_secs(60 * 5)).await;
        }
    }
}

#[shuttle_runtime::main]
async fn shuttle_main(
    #[shuttle_shared_db::Postgres] conn_string: String,
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> Result<Service, shuttle_runtime::Error> {
    let db = PgPoolOptions::new()
        .max_connections(1)
        .connect(&conn_string)
        .await
        .unwrap();

    let store = Datastore::new(db);
    let config = Config::read(&secrets);

    Ok(Service { store, config })
}
