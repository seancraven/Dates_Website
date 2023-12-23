use chrono::{DateTime, Local};
use serde::Serialize;
use sqlx::FromRow;
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum Status {
    Suggested,
    JointApproved,
    Rejected,
}

#[derive(Debug, Serialize, Clone, PartialEq, FromRow)]
pub struct Description {
    text: String,
    status: Status,
    day: chrono::DateTime<chrono::Local>,
}
impl Description {
    pub fn new(text: impl Into<String>, day: DateTime<Local>, status: Status) -> Description {
        Description {
            text: text.into(),
            status,
            day,
        }
    }
    pub fn approve(&mut self) {
        self.status = Status::JointApproved;
    }
    pub fn reject(&mut self) {
        self.status = Status::Rejected;
    }
}
#[derive(Debug, Serialize, Clone, PartialEq, FromRow)]
/// Date storage
///
/// * `name`: The name of the date
/// * `count`: The number of upvotes for the date.
pub struct Date {
    pub name: String,
    pub count: i32,
    pub id: uuid::Uuid,
    pub description: Option<Description>,
}
impl Date {
    pub fn new(name: impl Into<String>) -> Date {
        Date {
            name: name.into(),
            count: 0,
            id: uuid::Uuid::new_v4(),
            description: None,
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
