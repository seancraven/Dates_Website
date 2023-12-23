use serde::Serialize;
use sqlx;
use sqlx::sqlx_macros::Type;
use sqlx::types::chrono::{DateTime, Local};
use sqlx::types::uuid::Uuid;
use sqlx::FromRow;
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Type)]
#[repr(i32)]
pub enum Status {
    Suggested,
    Approved,
    Rejected,
}

#[derive(Debug, Serialize, Clone, PartialEq, FromRow)]
pub struct Description {
    pub text: String,
    pub status: Status,
    pub day: Option<DateTime<Local>>,
}
impl Description {
    pub fn default() -> Description {
        Description {
            text: "".into(),
            status: Status::Suggested,
            day: None,
        }
    }
    pub fn new(text: String, status: Status, day: Option<DateTime<Local>>) -> Description {
        Description { text, status, day }
    }
    pub fn approve(&mut self) {
        self.status = Status::Approved;
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
    pub id: Uuid,
    pub description: Description,
}
impl Date {
    pub fn new(name: impl Into<String>) -> Date {
        Date {
            name: name.into(),
            count: 0,
            id: uuid::Uuid::new_v4(),
            description: Description::default(),
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
