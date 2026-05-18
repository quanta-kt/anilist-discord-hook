use chrono::DateTime;
use reqwest::Client;
use rss::Channel;
use std::io::Cursor;

const MAL_RSS_BASE: &str = "https://myanimelist.net/rss.php";

pub struct MalClient<'a> {
    client: &'a Client,
}

pub struct RssEntry {
    pub title: String,
    pub url: String,
    pub description: String,
    pub pub_date: String,
}

impl RssEntry {
    pub fn timestamp(&self) -> Option<i64> {
        DateTime::parse_from_rfc2822(&self.pub_date)
            .ok()
            .map(|dt| dt.timestamp())
    }
}

impl MalClient<'_> {
    pub fn new(http_client: &Client) -> MalClient<'_> {
        MalClient {
            client: http_client,
        }
    }

    pub async fn fetch_history(
        &self,
        username: &str,
        after: Option<i64>,
    ) -> Result<Vec<RssEntry>, Box<dyn std::error::Error>> {
        let url = format!("{}?type=rw&u={}", MAL_RSS_BASE, username);
        let text = self.client.get(&url).send().await?.text().await?;
        let channel = Channel::read_from(Cursor::new(text.as_bytes()))?;

        let entries: Vec<RssEntry> = channel
            .items()
            .iter()
            .filter_map(|item| {
                let title = item.title()?.to_string();
                let url = item.link()?.to_string();
                let description = item.description()?.to_string();
                let pub_date = item.pub_date()?.to_string();

                Some(RssEntry {
                    title,
                    url,
                    description,
                    pub_date,
                })
            })
            .collect();

        Ok(match after {
            Some(after_ts) => entries
                .into_iter()
                .filter(|e| e.timestamp().map(|ts| ts > after_ts).unwrap_or(false))
                .collect(),
            None => entries,
        })
    }
}
