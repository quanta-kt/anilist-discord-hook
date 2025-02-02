use shuttle_runtime::SecretStore;

#[derive(Debug)]
pub struct Config {
    pub user_ids: Vec<u32>,
    pub webhook_url: String,
}

impl Config {
    pub fn read(secrets: &SecretStore) -> Config {
        let user_ids = Config::read_user_ids(secrets);
        let webhook_url = Config::read_webhook_url(secrets);

        Config {
            user_ids,
            webhook_url,
        }
    }

    fn read_user_ids(secrets: &SecretStore) -> Vec<u32> {
        secrets
            .get("ANILIST_USER_IDS")
            .map(|v| {
                v.split(',')
                    .filter_map(|v| match v.parse() {
                        Ok(id) => Some(id),
                        _ => {
                            eprintln!("Invalid value in ANILIST_USER_IDS: {}", v);
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn read_webhook_url(secrets: &SecretStore) -> String {
        secrets
            .get("DISCORD_WEBHOOK_URL")
            .expect("DISCORD_WEBHOOK_URL must be set")
    }
}
