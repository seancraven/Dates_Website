use actix_web::web;
use anyhow::anyhow;
use serde::Serialize;
use shuttle_runtime::async_trait;
use std::sync::Mutex;

pub type AppState = web::Data<Box<dyn Repository + Send + Sync>>;
#[async_trait]
/// Abstraction over storage, so that it can be in memory or persistent.
/// The repository shouldn't need to have mutable acess
pub trait Repository {
    /// Add a date to the repository.
    ///
    /// * `date`:
    async fn add(&self, date: Date);
    /// Remove a date from the repository.
    ///
    /// * `date`:
    async fn remove(&self, date: Date);
    /// Return a copy of the repository's contents.
    async fn get_all(&self) -> Vec<Date>;
    /// Update's the repository entry for a given date.
    ///
    /// * `date_name`:
    async fn update(&self, date: Date) -> anyhow::Result<()>;
    async fn increment_date_count(&self, date_id: &uuid::Uuid) -> anyhow::Result<()>;
    async fn decrement_date_count(&self, date_id: &uuid::Uuid) -> anyhow::Result<()>;
    async fn get(&self, date_id: &uuid::Uuid) -> Option<Date>;
}
#[derive(Debug, Serialize, Clone, PartialEq)]
/// Date storage
///
/// * `name`: The name of the date
/// * `count`: The number of upvotes for the date.
pub struct Date {
    pub name: String,
    pub count: usize,
    pub id: uuid::Uuid,
}
impl Date {
    pub fn new(name: impl Into<String>) -> Date {
        Date {
            name: name.into(),
            count: 0,
            id: uuid::Uuid::new_v4(),
        }
    }
    pub fn add(&mut self) {
        self.count += 1;
    }
    pub fn minus(&mut self) {
        if self.count > 0 {
            self.count -= 1;
        }
    }
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
    async fn add(&self, date: Date) {
        self.dates.lock().unwrap().push(date);
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
        self.dates.lock().unwrap().clone()
    }

    async fn remove(&self, date: Date) {
        let mut removal_ind = None;
        for (i, _date) in self.dates.lock().unwrap().iter().enumerate() {
            if _date.name == date.name {
                removal_ind = Some(i);
                break;
            }
        }
        if let Some(r_ind) = removal_ind {
            self.dates.lock().unwrap().remove(r_ind);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add() {
        let mut repo = VecRepo::new(vec![]);
        let date = Date::new("Sexy");
        repo.add(date.clone());
        assert!(repo.dates.lock().unwrap().contains(&date));
    }
}
