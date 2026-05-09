use chrono::DateTime;
use reqwest::Client;
use serde::Deserialize;

const JIKAN_API_URL: &str = "https://api.jikan.moe/v4";

pub struct MalClient<'a> {
    client: &'a Client,
}

#[derive(Deserialize, Debug)]
pub struct HistoryEntry {
    pub entry: EntryInfo,
    pub increment: i32,
    pub date: String,
}

#[derive(Deserialize, Debug)]
pub struct EntryInfo {
    #[serde(rename = "name")]
    pub title: String,
    pub url: String,
    #[serde(rename = "type")]
    pub media_type: String,
}

#[derive(Deserialize)]
struct HistoryResponse {
    data: Vec<HistoryEntry>,
}

impl HistoryEntry {
    pub fn timestamp(&self) -> Option<i64> {
        DateTime::parse_from_rfc3339(&self.date)
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
    ) -> Result<Vec<HistoryEntry>, reqwest::Error> {
        let url = format!("{}/users/{}/history", JIKAN_API_URL, username);
        let resp = self.client.get(&url).send().await?;
        let entries = resp.json::<HistoryResponse>().await?.data;

        Ok(match after {
            Some(after_ts) => entries
                .into_iter()
                .filter(|e| e.timestamp().map(|ts| ts > after_ts).unwrap_or(false))
                .collect(),
            None => entries,
        })
    }
}
