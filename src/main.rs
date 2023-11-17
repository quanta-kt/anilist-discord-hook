use std::error::Error;

use anilist::{Activity, AnilistClient};
use config::Config;
use datastore::Datastore;
use discord::{DiscordClient, Embed, WebhookMessage, Author};
use reqwest::blocking::Client;

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

    let username = activity
        .user
        .name
        .as_ref()
        .map(|n| n.as_str())
        .unwrap_or("?");

    let embed = Embed {
        color: activity
            .media
            .cover_image
            .as_ref()
            .map(|i| {
                i.color
                    .as_ref()
                    .map(|c| u8::from_str_radix(c.trim_start_matches("#"), 16).ok())
            })
            .flatten()
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

fn main() -> Result<(), Box<dyn Error>> {
    let http = Client::new();
    let anilist = AnilistClient::new(&http);
    let discord = DiscordClient::new(&http);

    let mut datastore = Datastore::read();
    let config = Config::read();

    let activities =
        anilist.fetch_activities(config.user_ids, Some(datastore.last_published_timestamp))?;

    for activity in activities.iter().rev() {
        discord.send(&config.webhook_url, format_discord_message(activity))?;
    }

    if let Some(activity) = activities.get(0) {
        datastore.last_published_timestamp = activity.created_at;
        datastore.write();
    }

    Ok(())
}
