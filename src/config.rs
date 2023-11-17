use std::env;

#[derive(Debug)]
pub struct Config {
    pub user_ids: Vec<u32>,
    pub webhook_url: String,
}

impl Config {
    pub fn read() -> Config {
        let _ = dotenvy::dotenv();

        let user_ids = Config::read_user_ids();
        let webhook_url = Config::read_webhook_url();

        Config { user_ids, webhook_url }
    }

    fn read_user_ids() -> Vec<u32> {
        env::var("ANILIST_USER_IDS")
            .and_then(|v| {
                Ok(v.split(",")
                    .filter_map(|v| match v.parse() {
                        Ok(id) => Some(id),
                        _ => {
                            eprintln!("Invalid value in ANILIST_USER_IDS: {}", v);
                            None
                        }
                    })
                    .collect())
            })
            .unwrap_or(vec![])
    }

    fn read_webhook_url() -> String {
        env::var("DISCORD_WEBHOOK_URL").expect("DISCORD_WEBHOOK_URL must be set")
    }
}
