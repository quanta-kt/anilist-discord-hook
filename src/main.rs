use anilist::{Activity, AnilistClient};
use config::Config;
use datastore::Datastore;
use discord::{Author, DiscordClient, Embed, WebhookMessage};
use mal::{HistoryEntry, MalClient};
use reqwest::Client;
use tokio::time::sleep;

use std::error::Error;
use std::time::Duration;

mod anilist;
mod config;
mod datastore;
mod discord;
mod mal;

fn format_discord_message(activity: &Activity) -> WebhookMessage {
    let description = match activity.status.as_str() {
        "dropped" => format!(
            "Dropped [{}]({})",
            &activity.media.title, &activity.media.site_url
        ),
        "completed" => format!(
            "Completed [{}]({})",
            &activity.media.title, &activity.media.site_url
        ),
        "watched episode" => format!(
            "Watched episode {} of [{}]({})",
            &activity.progress.as_deref().unwrap_or("?"),
            &activity.media.title,
            &activity.media.site_url,
        ),
        "rewatched episode" => format!(
            "Rewatched episode {} of [{}]({})",
            &activity.progress.as_deref().unwrap_or("?"),
            &activity.media.title,
            &activity.media.site_url
        ),
        "plans to watch" => format!(
            "Plans to watch [{}]({})",
            &activity.media.title, &activity.media.site_url
        ),
        "paused watching" => format!(
            "Paused watching [{}]({})",
            &activity.media.title, &activity.media.site_url,
        ),
        "read chapter" => format!(
            "Read chapter {} of [{}]({})",
            &activity.progress.as_deref().unwrap_or("?"),
            &activity.media.title,
            &activity.media.site_url
        ),
        "reread chapter" => format!(
            "Read chapter {} of [{}]({})",
            &activity.progress.as_deref().unwrap_or("?"),
            &activity.media.title,
            &activity.media.site_url
        ),
        "plans to read" => format!(
            "Plans to read [{}]({})",
            &activity.media.title, &activity.media.site_url
        ),
        "paused reading" => format!(
            "Paused reading [{}]({})",
            &activity.media.title, &activity.media.site_url,
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
                    .map(|c| u32::from_str_radix(c.trim_start_matches('#'), 16).ok())
            })
            .flatten(),

        title: Some(activity.media.title.clone()),

        author: Some(Author {
            name: username.to_owned(),
            icon_url: activity.user.avatar.clone(),
        }),

        description: Some(description),

        thumbnail: activity.media.cover_image.as_ref().map(|i| i.url.clone()),

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

fn format_mal_discord_message(username: &str, entry: &HistoryEntry) -> WebhookMessage {
    let description = match entry.entry.media_type.as_str() {
        "anime" => match entry.increment {
            1 => format!(
                "Watched an episode of [{}]({})",
                entry.entry.title, entry.entry.url
            ),
            n if n > 1 => format!(
                "Watched {} episodes of [{}]({})",
                n, entry.entry.title, entry.entry.url
            ),
            _ => format!("Updated [{}]({})", entry.entry.title, entry.entry.url),
        },
        "manga" => match entry.increment {
            1 => format!(
                "Read a chapter of [{}]({})",
                entry.entry.title, entry.entry.url
            ),
            n if n > 1 => format!(
                "Read {} chapters of [{}]({})",
                n, entry.entry.title, entry.entry.url
            ),
            _ => format!("Updated [{}]({})", entry.entry.title, entry.entry.url),
        },
        _ => format!("Updated [{}]({})", entry.entry.title, entry.entry.url),
    };

    let timestamp = entry
        .timestamp()
        .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
        .map(|dt| dt.to_rfc3339());

    let embed = Embed {
        color: Some(0x2E51A2),
        title: Some(entry.entry.title.clone()),
        author: Some(Author {
            name: username.to_owned(),
            icon_url: None,
        }),
        description: Some(description),
        thumbnail: None,
        url: Some(entry.entry.url.clone()),
        embed_type: "rich".to_string(),
        timestamp,
    };

    WebhookMessage {
        content: None,
        username: Some(username.to_owned()),
        avatar_url: None,
        embeds: Some(vec![embed]),
    }
}

struct Service {
    store: Datastore,
    config: Config,
}

impl Service {
    async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let http = Client::new();
        let anilist = AnilistClient::new(&http);
        let mal = MalClient::new(&http);
        let discord = DiscordClient::new(&http);

        let config = &self.config;

        loop {
            let last_ts = self.store.get_last_published_timestamp().unwrap_or(0);
            let now = chrono::Utc::now().timestamp();

            if !config.user_ids.is_empty() {
                let activities = anilist
                    .fetch_activities(&config.user_ids, Some(last_ts))
                    .await
                    .unwrap();

                for activity in activities.iter().rev() {
                    discord
                        .send(&config.webhook_url, format_discord_message(activity))
                        .await
                        .unwrap();
                }
            }

            if !config.mal_usernames.is_empty() {
                let mut all_entries: Vec<(String, HistoryEntry)> = Vec::new();
                for username in &config.mal_usernames {
                    let entries = mal.fetch_history(username, Some(last_ts)).await.unwrap();
                    for entry in entries {
                        all_entries.push((username.clone(), entry));
                    }
                }

                all_entries.sort_by_key(|(_, e)| e.timestamp().unwrap_or(0));

                for (username, entry) in &all_entries {
                    discord
                        .send(
                            &config.webhook_url,
                            format_mal_discord_message(username, entry),
                        )
                        .await
                        .unwrap();
                }
            }

            self.store.set_last_published_timestamp(now).unwrap();

            sleep(Duration::from_secs(60 * 5)).await;
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let store = Datastore::new();
    let config = Config::read();

    let mut service = Service { store, config };
    service.run().await?;

    Ok(())
}
