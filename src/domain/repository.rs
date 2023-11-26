use actix_web::web;
use serde::Serialize;
use std::sync::Mutex;

pub type AppState = web::Data<Mutex<Box<dyn Repository + Send + Sync>>>;
/// Abstraction over storage, so that it can be in memory or persistent.
pub trait Repository {
    /// Add a date to the repository.
    ///
    /// * `date`:
    fn add(&mut self, date: Date);
    /// Remove a date from the repository.
    ///
    /// * `date`:
    fn remove(&mut self, date: Date);
    /// Return a copy of the repository's contents.
    fn get_all(&self) -> Vec<Date>;
    /// Return a mutable ref to date.
    ///
    /// * `date_name`:
    fn get_mut(&mut self, date_name: &str) -> Option<&mut Date>;
    fn get(&self, date_name: &str) -> Option<&Date>;
}
#[derive(Debug, Serialize, Clone, PartialEq)]
/// Date storage
///
/// * `name`: The name of the date
/// * `count`: The number of upvotes for the date.
pub struct Date {
    name: String,
    count: usize,
}
impl Date {
    pub fn new(name: String) -> Date {
        Date { name, count: 0 }
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
#[derive(Clone)]
pub struct VecRepo {
    pub dates: Vec<Date>,
}
impl VecRepo {
    pub fn new(dates: Vec<Date>) -> VecRepo {
        VecRepo { dates }
    }
}
impl Repository for VecRepo {
    fn add(&mut self, date: Date) {
        self.dates.push(date);
    }
    fn get_mut(&mut self, date_name: &str) -> Option<&mut Date> {
        self.dates.iter_mut().find(|x| x.name == date_name)
    }
    fn get(&self, date_name: &str) -> Option<&Date> {
        self.dates.iter().find(|x| x.name == date_name)
    }
    fn get_all(&self) -> Vec<Date> {
        self.dates.clone()
    }

    fn remove(&mut self, date: Date) {
        let mut removal_ind = None;
        for (i, _date) in self.dates.iter().enumerate() {
            if _date.name == date.name {
                removal_ind = Some(i);
                break;
            }
        }
        if let Some(r_ind) = removal_ind {
            self.dates.remove(r_ind);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add() {
        let mut repo = VecRepo::new(vec![]);
        let date = Date::new("Sexy".into());
        repo.add(date.clone());
        assert!(repo.dates.contains(&date));
    }
}
