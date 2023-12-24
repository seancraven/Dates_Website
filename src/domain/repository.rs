use super::dates::Date;
use actix_web::web;
use anyhow::anyhow;
use shuttle_runtime::async_trait;
use std::sync::Mutex;
use uuid::Uuid;

pub struct AppState {
    pub repo: Box<dyn Repository + Send + Sync>,
    pub cache: ExpansionCache,
}
impl AppState {
    pub fn new(repo: Box<dyn Repository + Send + Sync>) -> AppState {
        AppState {
            repo,
            cache: ExpansionCache::new(),
        }
    }
    pub fn new_in_web_data(repo: Box<dyn Repository + Send + Sync>) -> web::Data<AppState> {
        web::Data::new(AppState::new(repo))
    }
}

#[derive(Debug)]
pub struct ExpansionCache {
    cache: Mutex<Vec<Uuid>>,
}
impl ExpansionCache {
    pub fn remove(&self, id: &Uuid) {
        self.cache.lock().unwrap().retain(|x| x != id);
    }
    pub fn add(&self, id: Uuid) {
        self.cache.lock().unwrap().push(id);
    }
    pub fn contains(&self, id: &Uuid) -> bool {
        self.cache.lock().unwrap().contains(id)
    }
    pub fn reset(&self) {
        self.cache.lock().unwrap().clear();
    }
    pub fn new() -> ExpansionCache {
        ExpansionCache {
            cache: Mutex::new(vec![]),
        }
    }
}

#[async_trait]
/// Abstraction over storage, so that it can be in memory or persistent.
/// The repository shouldn't need to have mutable acess
pub trait Repository {
    /// Add a date to the repository.
    ///
    /// * `date`:
    async fn add(&self, date: Date) -> anyhow::Result<()>;
    /// Remove a date from the repository.
    ///
    /// * `date_id`:
    async fn remove(&self, date_id: &uuid::Uuid) -> anyhow::Result<()>;
    /// Return a copy of the repository's contents, sorted by from higest to lowest.
    async fn get_all(&self) -> Vec<Date>;
    /// Update's the repository entry for a given date.
    ///
    /// * `date_name`:
    async fn update(&self, date: Date) -> anyhow::Result<()>;
    /// Increment the count of a given date.
    ///
    /// * `date_id`: date to increment
    async fn increment_date_count(&self, date_id: &uuid::Uuid) -> anyhow::Result<()>;
    /// Decrement the count of a given date.
    ///
    /// * `date_id`: date to decrement
    async fn decrement_date_count(&self, date_id: &uuid::Uuid) -> anyhow::Result<()>;
    /// Get a date from the repository.
    ///
    /// * `date_id`:
    async fn get(&self, date_id: &uuid::Uuid) -> Option<Date>;
}

pub struct VecRepo {
    pub dates: Mutex<Vec<Date>>,
}
impl VecRepo {
    pub fn new(dates: Vec<Date>) -> VecRepo {
        VecRepo {
            dates: Mutex::new(dates),
        }
    }
}
#[async_trait]
impl Repository for VecRepo {
    async fn add(&self, date: Date) -> anyhow::Result<()> {
        self.dates.lock().unwrap().push(date);
        Ok(())
    }
    async fn update(&self, new_date: Date) -> anyhow::Result<()> {
        if let Some(date) = self
            .dates
            .lock()
            .unwrap()
            .iter_mut()
            .find(|d| d.id == new_date.id)
        {
            date.count = new_date.count;
            date.name = new_date.name;
            Ok(())
        } else {
            Err(anyhow!("{:?} doesn't exist", new_date))
        }
    }
    async fn increment_date_count(&self, date_id: &uuid::Uuid) -> anyhow::Result<()> {
        match self
            .dates
            .lock()
            .unwrap()
            .iter_mut()
            .find(|x| &x.id == date_id)
        {
            Some(date) => Ok(date.add()),
            None => Err(anyhow::anyhow!("No Date exists to increment.")),
        }
    }
    async fn decrement_date_count(&self, date_id: &uuid::Uuid) -> anyhow::Result<()> {
        match self
            .dates
            .lock()
            .unwrap()
            .iter_mut()
            .find(|x| &x.id == date_id)
        {
            Some(date) => Ok(date.minus()),
            None => Err(anyhow::anyhow!("No Date exists to decrement.")),
        }
    }

    async fn get(&self, date_id: &uuid::Uuid) -> Option<Date> {
        if let Some(date) = self.dates.lock().unwrap().iter().find(|x| &x.id == date_id) {
            return Some(date.clone());
        };
        None
    }
    async fn get_all(&self) -> Vec<Date> {
        let mut v = self.dates.lock().unwrap().clone();
        v.sort_by(|a, b| b.count.cmp(&a.count));
        v
    }

    async fn remove(&self, date_id: &uuid::Uuid) -> anyhow::Result<()> {
        let mut removal_ind = None;
        for (i, _date) in self.dates.lock().unwrap().iter().enumerate() {
            if _date.id == *date_id {
                removal_ind = Some(i);
                break;
            }
        }
        if let Some(r_ind) = removal_ind {
            self.dates.lock().unwrap().remove(r_ind);
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_add() {
        let repo = VecRepo::new(vec![]);
        let date = Date::new("Sexy");
        repo.add(date.clone()).await.unwrap();
        let test_date = repo.get(&date.id).await.unwrap();
        assert_eq!(test_date, date);
    }
}
