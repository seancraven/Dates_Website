use crate::auth::user::{AuthorizedUser, NoGroupUser, UserRepository};
use crate::domain::dates::Date;
use crate::domain::repository::{DateRepository, InsertDateError, Repository};
use anyhow::anyhow;
use serde::Deserialize;
use shuttle_runtime::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Deserialize, Default)]
pub struct VecRepo {
    dates: Mutex<HashMap<Uuid, Vec<Date>>>,
    users: Mutex<Vec<AuthorizedUser>>,
    groups: Mutex<Vec<i32>>,
}
impl VecRepo {
    pub fn new() -> VecRepo {
        VecRepo {
            dates: Mutex::new(HashMap::new()),
            users: Mutex::new(vec![]),
            groups: Mutex::new(vec![]),
        }
    }
}
#[async_trait]
impl Repository for VecRepo {}
#[async_trait]
impl DateRepository for VecRepo {
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
#[async_trait]
impl UserRepository for VecRepo {
    async fn create_group(&self) -> anyhow::Result<i32> {
        let mut dates_list = self.groups.lock().unwrap();
        let len = dates_list.len() as i32;
        dates_list.push(len);
        Ok(len)
    }
    async fn add_user_to_group(
        &self,
        user: crate::auth::user::NoGroupUser,
        group: i32,
    ) -> anyhow::Result<crate::auth::user::GroupUser> {
        if self.groups.lock().unwrap().contains(&group) {
            Ok(user.join_group(group))
        } else {
            Err(anyhow!("Group doesn't exist"))
        }
    }
    async fn get_group_by_email(&self, email: &str) -> anyhow::Result<i32> {
        match self
            .users
            .lock()
            .unwrap()
            .iter()
            .find(|u| match u {
                AuthorizedUser::GroupUser(u) => u.email == email,
                AuthorizedUser::NoGroupUser(u) => u.email == email,
            })
            .ok_or(anyhow!("Can't find user"))?
        {
            AuthorizedUser::GroupUser(u) => Ok(u.user_group),
            AuthorizedUser::NoGroupUser(_) => Err(anyhow!("User is not part of a group.")),
        }
    }
    async fn change_user_password(
        &self,
        user: AuthorizedUser,
        new_password: secrecy::Secret<String>,
    ) -> anyhow::Result<AuthorizedUser> {
        todo!();
    }
    async fn remove_user(&self, user_id: &Uuid) -> anyhow::Result<()> {
        todo!()
    }
    async fn validate_user(
        &self,
        user: &crate::auth::user::UnauthorizedUser,
    ) -> Result<AuthorizedUser, crate::auth::user::UserValidationError> {
        todo!();
    }
    async fn create_authorized_user(
        &self,
        user: crate::auth::user::UnauthorizedUser,
    ) -> anyhow::Result<NoGroupUser> {
        todo!();
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
