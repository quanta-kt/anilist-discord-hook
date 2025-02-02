use sqlx::{
    types::time::{OffsetDateTime, PrimitiveDateTime, UtcOffset},
    Pool, Postgres,
};

pub struct Datastore {
    db: Pool<Postgres>,
}

impl Datastore {
    pub fn new(db: Pool<Postgres>) -> Datastore {
        Datastore { db }
    }

    pub async fn get_last_published_timestamp(&self) -> Option<i64> {
        let row = sqlx::query!("SELECT last_published_timestamp FROM LastPublishedTimestamp")
            .fetch_one(&self.db)
            .await
            .ok()?;

        Some(
            row.last_published_timestamp
                .assume_offset(UtcOffset::UTC)
                .unix_timestamp(),
        )
    }

    pub async fn set_last_published_timestamp(&self, timestamp: i64) -> Result<(), sqlx::Error> {
        let timestamp = OffsetDateTime::from_unix_timestamp(timestamp).unwrap();

        sqlx::query!(
            r#"
            INSERT
            INTO LastPublishedTimestamp(last_published_timestamp, id)
            VALUES($1, 0)
            ON CONFLICT (id)
            DO UPDATE
                SET last_published_timestamp = $1
            "#,
            PrimitiveDateTime::new(timestamp.date(), timestamp.time())
        )
        .execute(&self.db)
        .await
        .map(|_| ())
    }
}
