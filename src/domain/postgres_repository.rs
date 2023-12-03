use log::debug;
use log::error;
use shuttle_runtime::async_trait;
use sqlx::{prelude::FromRow, PgPool};

use super::repository::{Date, Repository};

struct PgRepo {
    pool: PgPool,
}
#[derive(FromRow, Debug, Clone)]
struct PgDate {
    id: String,
    name: String,
    count_: i32,
}
impl TryInto<Date> for PgDate {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<Date, Self::Error> {
        Ok(Date {
            id: uuid::Uuid::parse_str(&self.id)?,
            name: self.name,
            count: self.count_,
        })
    }
}

#[async_trait]
impl Repository for PgRepo {
    async fn add(&self, date: super::repository::Date) {
        sqlx::query!(
            r#"INSERT INTO dates (id, name, count_ ) VALUES ($1, $2, $3)"#,
            date.id.to_string(),
            date.name.clone(),
            date.count,
        )
        .execute(&self.pool)
        .await
        .unwrap();
    }
    async fn get(&self, _date_id: &uuid::Uuid) -> Option<super::repository::Date> {
        match sqlx::query_as!(
            PgDate,
            r#"SELECT * FROM dates WHERE id=$1"#,
            _date_id.to_string(),
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(d) => match d.try_into() {
                Ok(date) => Some(date),
                Err(e) => {
                    error!("Query conversion error: {} on converting Uuid", e);
                    None
                }
            },
            Err(e) => {
                error!("Database Query error: {}", e);
                None
            }
        }
    }
    async fn remove(&self, _date: super::repository::Date) {
        todo!()
    }
    async fn get_all(&self) -> Vec<super::repository::Date> {
        todo!()
    }
    async fn decrement_date_count(&self, _date_id: &uuid::Uuid) -> anyhow::Result<()> {
        todo!()
    }
    async fn increment_date_count(&self, _date_id: &uuid::Uuid) -> anyhow::Result<()> {
        todo!()
    }
    async fn update(&self, _date: super::repository::Date) -> anyhow::Result<()> {
        todo!()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use env_logger::try_init;
    use sqlx::postgres::PgPool;
    use std::fs::read_to_string;
    fn init() {
        try_init();
    }

    async fn db_setup() -> PgPool {
        let secret_toml: toml::Value =
            toml::from_str(&*read_to_string("Secrets.toml").expect("Can't load Secrets.toml"))
                .unwrap();
        PgPool::connect(
            secret_toml
                .get("DATABASE_URL")
                .expect("Couldn't find key in toml")
                .as_str()
                .expect("decoding to str failed"),
        )
        .await
        .unwrap()
    }
    #[tokio::test]
    async fn test_add_get() {
        init();
        let repo = PgRepo {
            pool: db_setup().await,
        };
        let date = Date {
            name: "Test".into(),
            count: 0,
            id: uuid::Uuid::new_v4(),
        };
        repo.add(date.clone()).await;
        let ret_date = repo.get(&date.id).await.unwrap();
        assert_eq!(ret_date, date);
    }
}
