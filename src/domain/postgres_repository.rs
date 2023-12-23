use super::repository::Repository;
use chrono::Local;
use log::error;
use shuttle_runtime::async_trait;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::types::uuid::Uuid;
use sqlx::FromRow;
use sqlx::PgPool;

use super::dates::{Date, Description, Status};

pub struct PgRepo {
    pub pool: PgPool,
}
#[derive(FromRow, Debug, Clone)]
struct PgDate {
    id: String,
    name: String,
    count_: i32,
    #[sqlx(default)]
    day: Option<DateTime<Utc>>,
    #[sqlx(default)]
    description: Option<String>,
    status: i32,
}
impl TryInto<Date> for PgDate {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<Date, Self::Error> {
        Ok(Date {
            id: uuid::Uuid::parse_str(&self.id)?,
            name: self.name,
            count: self.count_,
            description: Description::new(
                self.description.unwrap_or("".into()),
                match self.status {
                    0 => Status::Suggested,
                    1 => Status::Approved,
                    2 => Status::Rejected,
                    _ => return Err(anyhow::anyhow!("Invalid status")),
                },
                self.day.map(|d| d.with_timezone(&Local)),
            ),
        })
    }
}

#[async_trait]
impl Repository for PgRepo {
    async fn add(&self, date: Date) -> anyhow::Result<()> {
        sqlx::query!(
            r#"INSERT INTO dates (id, name, count_ , day , status,  description) VALUES ($1, $2, $3, $4, $5, $6)"#,
            date.id,
            date.name.clone(),
            date.count,
            date.description.day,
            date.description.status as i32,
            date.description.text,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    async fn get(&self, _date_id: &Uuid) -> Option<Date> {
        match sqlx::query_as!(PgDate, r#"SELECT * FROM dates WHERE id=$1"#, _date_id)
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
        sqlx::query!(r#"DELETE FROM dates WHERE id=$1"#, date_id)
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
            _date_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    async fn increment_date_count(&self, _date_id: &uuid::Uuid) -> anyhow::Result<()> {
        sqlx::query!(
            r#"UPDATE dates SET count_=count_+1 WHERE id = $1"#,
            _date_id
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
            _date.id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use env_logger::try_init;
    use sqlx::postgres::PgPool;

    fn init() {
        try_init().ok();
    }

    async fn db_setup() -> PgPool {
        PgPool::connect("postgres://postgres:postgres@localhost:17972/postgres")
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
            description: Description::default(),
        };
        repo.add(date.clone()).await.unwrap();
        let ret_date = repo.get(&date.id).await.unwrap();
        assert_eq!(ret_date, date);
        repo.remove(&date.id).await.unwrap();
    }
    #[tokio::test]
    async fn test_remove() {
        init();
        let repo = PgRepo {
            pool: db_setup().await,
        };
        let date = Date {
            name: "Test".into(),
            count: 0,
            id: uuid::Uuid::new_v4(),
            description: Description::default(),
        };
        repo.add(date.clone()).await.unwrap();
        repo.remove(&date.id).await.unwrap();
        assert!(repo.get(&date.id).await.is_none());
    }
    #[tokio::test]
    async fn test_increment() {
        init();
        let repo = PgRepo {
            pool: db_setup().await,
        };
        let date = Date {
            name: "Test".into(),
            count: 0,
            id: uuid::Uuid::new_v4(),
            description: Description::default(),
        };
        repo.add(date.clone()).await.unwrap();
        repo.increment_date_count(&date.id).await.unwrap();
        assert!(repo.get(&date.id).await.unwrap().count == 1);
        repo.remove(&date.id).await.unwrap();
    }
    #[tokio::test]
    async fn test_decrement() {
        init();
        let repo = PgRepo {
            pool: db_setup().await,
        };
        let date = Date {
            name: "Test".into(),
            count: 1,
            id: uuid::Uuid::new_v4(),
            description: Description::default(),
        };
        repo.add(date.clone()).await.unwrap();
        repo.decrement_date_count(&date.id).await.unwrap();
        assert!(repo.get(&date.id).await.unwrap().count == 0);
        repo.decrement_date_count(&date.id).await.unwrap();
        assert!(repo.get(&date.id).await.unwrap().count == 0);
        repo.remove(&date.id).await.unwrap();
    }

    #[tokio::test]
    async fn test_get_all() {
        init();
        let repo = PgRepo {
            pool: db_setup().await,
        };
        let mut dates = vec![];
        for i in 0..4 {
            let date = Date {
                name: "test_multi".into(),
                count: i,
                id: uuid::Uuid::new_v4(),
                description: Description::default(),
            };
            dates.push(date.clone());
            repo.add(date).await.unwrap();
        }

        let mut ret_dates: Vec<Date> = repo
            .get_all()
            .await
            .into_iter()
            .filter(|d| d.name == "test_multi")
            .collect();
        ret_dates.sort_by(|a, b| a.count.cmp(&b.count));
        dates.sort_by(|a, b| a.count.cmp(&b.count));
        assert_eq!(dates, ret_dates);
        for date in dates {
            repo.remove(&date.id).await.unwrap();
        }
    }
}
