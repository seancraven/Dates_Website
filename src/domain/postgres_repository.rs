use log::error;
use shuttle_runtime::async_trait;
use sqlx::{prelude::FromRow, PgPool};

use super::repository::{Date, Repository};

pub struct PgRepo {
    pub pool: PgPool,
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
    async fn add(&self, date: super::repository::Date) -> anyhow::Result<()> {
        sqlx::query!(
            r#"INSERT INTO dates (id, name, count_ ) VALUES ($1, $2, $3)"#,
            date.id.to_string(),
            date.name.clone(),
            date.count,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
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
    async fn remove(&self, date_id: &uuid::Uuid) -> anyhow::Result<()> {
        sqlx::query!(r#"DELETE FROM dates WHERE id=$1"#, date_id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    async fn get_all(&self) -> Vec<Date> {
        match sqlx::query_as!(PgDate, r#"SELECT * FROM dates"#)
            .fetch_all(&self.pool)
            .await
        {
            Ok(d) => {
                let mut v = d
                    .into_iter()
                    .filter_map(|d| d.try_into().ok())
                    .collect::<Vec<Date>>();
                v.sort_by(|a, b| b.count.cmp(&a.count));
                v
            }

            Err(e) => {
                error!("Database Query error: {}", e);
                vec![]
            }
        }
    }
    async fn decrement_date_count(&self, _date_id: &uuid::Uuid) -> anyhow::Result<()> {
        sqlx::query!(
            r#"UPDATE dates SET count_=count_-1 WHERE id = $1 and count_ > 0"#,
            _date_id.to_string()
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    async fn increment_date_count(&self, _date_id: &uuid::Uuid) -> anyhow::Result<()> {
        sqlx::query!(
            r#"UPDATE dates SET count_=count_+1 WHERE id = $1"#,
            _date_id.to_string()
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    async fn update(&self, _date: Date) -> anyhow::Result<()> {
        sqlx::query!(
            r#"UPDATE dates SET count_=$1, name=$2 WHERE id = $3"#,
            _date.count,
            _date.name,
            _date.id.to_string()
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
