use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_flat_path::flat_path;

const ANILIST_API_URL: &str = "https://graphql.anilist.co/";

const QUERY_FETCH_ACTIVITIES: &str = "
    query($user_ids: [Int], $after: Int) {
      Page(perPage: 10) {
        activities(type: MEDIA_LIST, userId_in: $user_ids, sort: ID_DESC, createdAt_greater: $after) {
          ...on ListActivity {
            id
            status
            progress
            createdAt
            user {
              name
              avatar {
                medium
              }
            }
            media {
              title {
                romaji
              }
              siteUrl

              coverImage {
                medium
                color
              }
            }
          }
        }
      }
    }
";

pub struct AnilistClient<'a> {
    client: &'a Client,
}

#[derive(Serialize)]
struct QueryPayload<'a, T>
where
    T: Serialize,
{
    query: &'a str,
    variables: T,
}

#[flat_path]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub name: Option<String>,

    #[flat_path("avatar.medium")]
    pub avatar: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CoverImage {
    pub color: Option<String>,

    #[serde(rename = "medium")]
    pub url: String,
}

#[flat_path]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    #[flat_path("title.romaji")]
    pub title: String,

    pub site_url: String,

    pub cover_image: Option<CoverImage>,

    pub banner_image: Option<String>,
}

#[flat_path]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    pub id: u32,
    pub status: String,
    pub progress: Option<String>,
    pub created_at: i64,
    pub user: User,
    pub media: Media,
}

#[flat_path]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FetchActivitiesResponse {
    #[flat_path("data.Page.activities")]
    activities: Vec<Activity>,
}

impl AnilistClient<'_> {
    pub fn new(http_client: &Client) -> AnilistClient<'_> {
        AnilistClient {
            client: http_client,
        }
    }

    pub async fn fetch_activities(
        &self,
        user_ids: &[u32],
        after: Option<i64>,
    ) -> Result<Vec<Activity>, reqwest::Error> {
        #[derive(Serialize)]
        struct Variables<'a> {
            user_ids: &'a [u32],

            #[serde(skip_serializing_if = "Option::is_none")]
            after: Option<i64>,
        }

        let variables = Variables { user_ids, after };

        let query = AnilistClient::build_query(QUERY_FETCH_ACTIVITIES, variables);

        let resp = self
            .client
            .post(ANILIST_API_URL)
            .json(&query)
            .send()
            .await?;

        resp.json::<FetchActivitiesResponse>()
            .await
            .map(|out| out.activities)
    }

    fn build_query<T>(query: &str, variables: T) -> QueryPayload<T>
    where
        T: Serialize,
    {
        QueryPayload { query, variables }
    }
}
