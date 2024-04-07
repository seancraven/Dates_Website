use crate::auth::user::UserRepository;
use crate::email::EmailClient;

use super::dates::Date;
use actix_web::web;
use shuttle_runtime::async_trait;
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use thiserror::Error;
use tracing::info;
use uuid::Uuid;

pub struct AppState {
    pub repo: Box<dyn Repository + Send + Sync>,
    pub cache: ExpansionCache,
    pub email_client: EmailClient,
}
impl AppState {
    pub fn new(repo: Box<dyn Repository + Send + Sync>, email_client: EmailClient) -> AppState {
        AppState {
            repo,
            cache: ExpansionCache::new(),
            email_client,
        }
    }
    pub fn new_in_web_data(
        repo: Box<dyn Repository + Send + Sync>,
        email_client: EmailClient,
    ) -> web::Data<AppState> {
        web::Data::new(AppState::new(repo, email_client))
    }
}

#[derive(Error, Debug)]
#[error("User isn't in cache")]
pub struct MissingUserError;

#[derive(Debug, Default)]
/// Cache that stores wheather a user has the page open.
///
/// * `cache`:
pub struct ExpansionCache {
    cache: Mutex<HashMap<Uuid, Vec<Uuid>>>,
    queue: Mutex<VecDeque<Uuid>>,
    queue_len: usize,
}
impl ExpansionCache {
    pub fn remove(&self, date_id: &Uuid, user_id: &Uuid) -> Result<(), MissingUserError> {
        self.cache
            .lock()
            .unwrap()
            .get_mut(user_id)
            .ok_or(MissingUserError)?
            .retain(|x| {
                info!("removing {:?} from cache", date_id);
                x != date_id
            });
        Ok(())
    }
    /// Adds a date id if the user is in the cache.
    /// Otherwise adds user and date id.
    ///
    /// * `id`:
    /// * `user_id`:
    pub fn add(&self, date_id: Uuid, user_id: &Uuid) {
        let mut cache = self.cache.lock().unwrap();
        match cache.get_mut(user_id) {
            Some(user_cache) => user_cache.push(date_id),
            None => {
                cache.insert(*user_id, vec![date_id]);
                let mut queue = self.queue.lock().unwrap();
                queue.push_back(*user_id);
                if queue.len() > self.queue_len {
                    let to_drop = queue.pop_front().unwrap();
                    cache.remove(&to_drop).unwrap();
                }
            }
        };
    }
    pub fn contains(&self, date_id: &Uuid, user_id: &Uuid) -> Result<bool, MissingUserError> {
        Ok(self
            .cache
            .lock()
            .unwrap()
            .get(user_id)
            .ok_or(MissingUserError)?
            .contains(date_id))
    }
    pub fn reset(&self, user_id: &Uuid) -> Result<(), MissingUserError> {
        self.cache
            .lock()
            .unwrap()
            .get_mut(user_id)
            .ok_or(MissingUserError)?
            .clear();
        Ok(())
    }
    pub fn pop_user_cache(&self, user_id: &Uuid) {
        self.cache.lock().unwrap().remove(user_id);
    }
    pub fn new() -> ExpansionCache {
        let default_cache_size = 1000;
        ExpansionCache {
            cache: Mutex::new(HashMap::with_capacity(default_cache_size)),
            queue: Mutex::new(VecDeque::with_capacity(default_cache_size)),
            queue_len: default_cache_size,
        }
    }
}
#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::ExpansionCache;

    #[test]
    fn test_cache_len() {
        let mut cache = ExpansionCache::new();
        cache.queue_len = 100;
        for _ in 0..1000 {
            let date_id = Uuid::new_v4();
            let user_id = Uuid::new_v4();
            cache.add(date_id, &user_id);
            assert!(cache.cache.lock().unwrap().len() <= 100)
        }
    }
    #[test]
    fn test_cache_add() {
        let cache = ExpansionCache::new();
        let user_id = Uuid::new_v4();
        let date_id = Uuid::new_v4();
        cache.add(date_id, &user_id);
        assert!(cache.contains(&date_id, &user_id).unwrap());
    }
    #[test]
    fn test_cache_remove() {
        let cache = ExpansionCache::new();
        let user_id = Uuid::new_v4();
        cache.add(Uuid::new_v4(), &user_id);
        cache.pop_user_cache(&user_id);
        assert!(cache.contains(&Uuid::new_v4(), &user_id).is_err());
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
pub trait Repository: UserRepository + DateRepository {}
#[async_trait]
/// Abstraction over storage, so that it can be in memory or persistent.
/// The repository shouldn't need to have mutable acess
pub trait DateRepository {
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
    /// Return a copy of the all the user's dates in a sorted fashion.
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
