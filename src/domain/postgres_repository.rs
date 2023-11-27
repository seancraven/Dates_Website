use shuttle_runtime::async_trait;
use sqlx::{types::BigDecimal, PgPool};

use super::repository::Repository;

struct PgRepo {
    pool: PgPool,
}
#[async_trait]
impl Repository for PgRepo {
    async fn add(&self, date: super::repository::Date) {
        sqlx::query!(
            r#"INSERT INTO dates (id_1, id_2, name, count_ ) VALUES ($1, $2, $3, $4) RETURNING (id_1, id_2)"#,
            BigDecimal::from(date.id.as_u64_pair().0),
            BigDecimal::from(date.id.as_u64_pair().1),
            date.name.clone(),
            BigDecimal::from(date.count as u32),
        ).fetch_all(&self.pool).await.unwrap();
    }
    async fn get(&self, date_id: &uuid::Uuid) -> Option<super::repository::Date> {
        todo!()
    }
    async fn remove(&self, date: super::repository::Date) {
        todo!()
    }
    async fn get_all(&self) -> Vec<super::repository::Date> {
        todo!()
    }
    async fn decrement_date_count(&self, date_id: &uuid::Uuid) -> anyhow::Result<()> {
        todo!()
    }
    async fn increment_date_count(&self, date_id: &uuid::Uuid) -> anyhow::Result<()> {
        todo!()
    }
    async fn update(&self, date: super::repository::Date) -> anyhow::Result<()> {
        todo!()
    }
}
