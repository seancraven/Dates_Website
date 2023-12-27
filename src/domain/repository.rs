use super::dates::Date;
use actix_web::web;
use anyhow::anyhow;
use serde::Deserialize;
use shuttle_runtime::async_trait;
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use thiserror::Error;
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

#[derive(Error, Debug)]
#[error("User isn't in cache")]
pub struct MissingUserError;

#[derive(Debug)]
/// Cache that stores wheather a user has the page open.
///
/// * `cache`:
pub struct ExpansionCache {
    cache: Mutex<HashMap<Uuid, Vec<Uuid>>>,
    queue: Mutex<VecDeque<Uuid>>,
    queue_len: usize,
}
impl ExpansionCache {
    fn add_new_user(&self, user_id: &str) {}
    pub fn remove(&self, id: &Uuid, user_id: &Uuid) -> Result<(), MissingUserError> {
        self.cache
            .lock()
            .unwrap()
            .get_mut(user_id)
            .ok_or(MissingUserError)?
            .retain(|x| x != id);
        Ok(())
    }
    /// Adds a date id if the user is in the cache.
    /// Otherwise adds user and date id.
    ///
    /// * `id`:
    /// * `user_id`:
    pub fn add(&self, id: Uuid, user_id: &Uuid) -> Result<(), MissingUserError> {
        match self.cache.lock().unwrap().get_mut(user_id) {
            Some(user_cache) => user_cache.push(id),
            None => {
                self.cache
                    .lock()
                    .unwrap()
                    .insert(*user_id, vec![id])
                    .ok_or(MissingUserError)?;
                let mut queue = self.queue.lock().unwrap();
                queue.push_back(*user_id);
                if queue.len() > self.queue_len {
                    let to_drop = queue.pop_front().unwrap();
                    self.cache.lock().unwrap().remove(&to_drop).unwrap();
                }
            }
        };
        Ok(())
    }
    pub fn contains(&self, id: &Uuid, user_id: &Uuid) -> Result<bool, MissingUserError> {
        Ok(self
            .cache
            .lock()
            .unwrap()
            .get(user_id)
            .ok_or(MissingUserError)?
            .contains(id))
    }
    pub fn reset(&self, user_id: &Uuid) -> Result<(), MissingUserError> {
        Ok(self
            .cache
            .lock()
            .unwrap()
            .get_mut(user_id)
            .ok_or(MissingUserError)?
            .clear())
    }
    pub fn pop_user_cache(&self, user_id: &Uuid) {
        self.cache.lock().unwrap().remove(user_id);
    }
    pub fn new() -> ExpansionCache {
        ExpansionCache {
            cache: Mutex::new(HashMap::new()),
            queue: Mutex::new(VecDeque::new()),
            queue_len: 1000,
        }
    }
}
#[derive(Error, Debug)]
pub enum InsertDateError {
    #[error("Query Error")]
    QueryError,
    #[error("User isn't part of a group")]
    GroupMembershipError,
}
#[async_trait]
/// Abstraction over storage, so that it can be in memory or persistent.
/// The repository shouldn't need to have mutable acess
pub trait Repository {
    /// Add a date to the repository.
    ///
    /// * `date`:
    async fn add(&self, date: Date, user_id: Uuid) -> Result<(), InsertDateError>;
    /// Remove a date from the repository.
    ///
    /// * `date_id`:
    async fn remove<'a, 'ui, 'st>(
        &'a self,
        date_id: &'ui Uuid,
        user_id: &'st Uuid,
    ) -> anyhow::Result<()>;
    /// Return a copy of the repository's contents, sorted by from higest to lowest.

    async fn get_all(&self, user_id: &Uuid) -> Vec<Date>;
    /// Update's the repository entry for a given date.
    ///
    /// * `date_name`:
    async fn update(&self, date: Date, user_id: &Uuid) -> anyhow::Result<()>;
    /// Increment the count of a given date.
    ///
    /// * `date_id`: date to increment
    async fn increment_date_count<'a, 'ui, 'st>(
        &'a self,
        date_id: &'ui Uuid,
        user_id: &'st Uuid,
    ) -> anyhow::Result<()>;
    /// Decrement the count of a given date.
    ///
    /// * `date_id`: date to decrement
    async fn decrement_date_count<'a, 'ui, 'st>(
        &'a self,
        date_id: &'ui Uuid,
        user_id: &'st Uuid,
    ) -> anyhow::Result<()>;
    /// Get a date from the repository.
    ///
    /// * `date_id`:
    async fn get<'a, 'ui, 'st>(&'a self, date_id: &'ui Uuid, user_id: &'st Uuid) -> Option<Date>;

    /// Check that the user has access to the repository.
    ///
    /// * `user_id`:
    async fn check_user_has_access(&self, user_id: &Uuid) -> bool;
}
#[derive(Deserialize)]
pub struct VecRepo {
    dates: Mutex<HashMap<Uuid, Vec<Date>>>,
}
impl VecRepo {
    pub fn new() -> VecRepo {
        VecRepo {
            dates: Mutex::new(HashMap::new()),
        }
    }
}
#[async_trait]
impl Repository for VecRepo {
    async fn add(&self, date: Date, user_id: Uuid) -> Result<(), InsertDateError> {
        let mut map = self.dates.lock().unwrap();
        match map.get_mut(&user_id) {
            Some(date_vec) => date_vec.push(date),
            None => {
                map.insert(user_id, vec![date]);
            }
        }
        Ok(())
    }
    async fn update<'a, 'st>(&'a self, new_date: Date, user_id: &'st Uuid) -> anyhow::Result<()> {
        if let Some(date) = self
            .dates
            .lock()
            .unwrap()
            .get_mut(user_id)
            .unwrap()
            .iter_mut()
            .find(|d| d.id == new_date.id)
        {
            tracing::info!("Updating date: {:?}", date);
            tracing::info!("with: {:?}", &new_date);

            *date = new_date;
            Ok(())
        } else {
            Err(anyhow!("{:?} doesn't exist", new_date))
        }
    }
    async fn increment_date_count<'a, 'ui, 'st>(
        &'a self,
        date_id: &'ui Uuid,
        user_id: &'st Uuid,
    ) -> anyhow::Result<()> {
        match self
            .dates
            .lock()
            .unwrap()
            .get_mut(user_id)
            .unwrap()
            .iter_mut()
            .find(|x| &x.id == date_id)
        {
            Some(date) => {
                date.add();
                Ok(())
            }
            None => Err(anyhow::anyhow!("No Date exists to increment.")),
        }
    }
    async fn decrement_date_count<'a, 'ui, 'st>(
        &'a self,
        date_id: &'ui Uuid,
        user_id: &'st Uuid,
    ) -> anyhow::Result<()> {
        match self
            .dates
            .lock()
            .unwrap()
            .get_mut(user_id)
            .unwrap()
            .iter_mut()
            .find(|x| &x.id == date_id)
        {
            Some(date) => {
                date.minus();
                Ok(())
            }
            None => Err(anyhow::anyhow!("No Date exists to decrement.")),
        }
    }

    async fn get<'a, 'ui, 'st>(&'a self, date_id: &'ui Uuid, user_id: &'st Uuid) -> Option<Date> {
        tracing::info!("Getting date id: {}, user_id: {}", date_id, user_id);
        if let Some(date) = self
            .dates
            .lock()
            .unwrap()
            .get(user_id)
            .unwrap()
            .iter()
            .find(|x| &x.id == date_id)
        {
            return Some(date.clone());
        };
        None
    }

    async fn get_all(&self, user_id: &Uuid) -> Vec<Date> {
        let mut v = self.dates.lock().unwrap().get(user_id).unwrap().clone();
        v.sort_by(|a, b| b.count.cmp(&a.count));
        v
    }

    async fn remove<'a, 'ui, 'st>(
        &'a self,
        date_id: &'ui Uuid,
        user_id: &'st Uuid,
    ) -> anyhow::Result<()> {
        let mut removal_ind = None;
        for (i, _date) in self
            .dates
            .lock()
            .unwrap()
            .get(user_id)
            .unwrap()
            .iter()
            .enumerate()
        {
            if _date.id == *date_id {
                removal_ind = Some(i);
                break;
            }
        }
        if let Some(r_ind) = removal_ind {
            self.dates
                .lock()
                .unwrap()
                .get_mut(user_id)
                .unwrap()
                .remove(r_ind);
        }
        Ok(())
    }
    async fn check_user_has_access(&self, user_id: &Uuid) -> bool {
        self.dates.lock().unwrap().contains_key(user_id)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_add() {
        let repo = VecRepo::new();
        let date = Date::new("Sexy");
        let id = Uuid::new_v4();
        repo.add(date.clone(), id).await.unwrap();
        let test_date = repo.get(&date.id, &id).await.unwrap();
        assert_eq!(test_date, date);
    }
}
