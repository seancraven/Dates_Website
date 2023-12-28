use serde::{Deserialize, Serialize};
use sqlx;
use sqlx::sqlx_macros::Type;
use sqlx::types::chrono::{DateTime, Local};
use sqlx::types::uuid::Uuid;
use sqlx::FromRow;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Type, Deserialize)]
#[repr(i32)]
pub enum Status {
    Suggested,
    Approved,
    Rejected,
}

#[derive(Debug, Serialize, Clone, PartialEq, FromRow, Deserialize)]
pub struct Description {
    pub text: String,
    pub status: Status,
    pub day: Option<DateTime<Local>>,
}
impl std::default::Default for Description {
    fn default() -> Description {
        Description {
            text: "".into(),
            status: Status::Suggested,
            day: None,
        }
    }
}
impl Description {
    pub fn new(text: String, status: Status, day: Option<DateTime<Local>>) -> Description {
        Description { text, status, day }
    }
    pub fn approve(&mut self) {
        self.status = Status::Approved;
    }
    pub fn reject(&mut self) {
        self.status = Status::Rejected;
    }
    pub fn render_date(&self) -> String {
        match self.day {
            Some(day) => day.format("%H:%M %d/%m/%Y").to_string(),
            None => "No date set".into(),
        }
    }
    pub fn render_status(&self) -> String {
        match self.status {
            Status::Suggested => "Waiting for approval.".into(),
            Status::Approved => "Approved".into(),
            Status::Rejected => "Rejected".into(),
        }
    }
}
#[derive(Debug, Serialize, Clone, PartialEq, Deserialize)]
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
