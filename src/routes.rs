use actix_web::{get, post, HttpResponse, Responder};
use serde::Serialize;
use std::fs::read;
use tera::{self, Context};

#[get("/")]
async fn index() -> impl Responder {
    let dates = vec![Date::new(String::from("placeholder")); 5];
    HttpResponse::Ok().body(template_load(dates).expect("Templating failed."))
}

#[derive(Debug, Serialize, Clone)]
struct Date {
    name: String,
    count: usize,
}
impl Date {
    fn new(name: String) -> Date {
        Date { name, count: 0 }
    }
    fn add(&mut self) {
        self.count += 1;
    }
}

trait Repository {
    fn add(&mut self, date: Date);
    fn remove(&mut self, date: Date);
    fn get(&self) -> Vec<Date>;
    fn update_count(&mut self, date_name: &str);
}

struct DateRepo {
    dates: Vec<Date>,
}
impl Repository for DateRepo {
    fn add(&mut self, date: Date) {
        self.dates.push(date);
    }
    fn get(&self) -> Vec<Date> {
        self.dates.clone()
    }
    fn update_count(&mut self, date_name: &str) {
        for _date in self.dates.iter_mut() {
            if &*_date.name == date_name {
                _date.add();
                break;
            };
        }
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

fn template_load(dates: Vec<Date>) -> anyhow::Result<String> {
    let mut ctx = Context::new();
    ctx.insert("dates", &dates);
    let html = tera::Tera::one_off(
        std::str::from_utf8(&read("./pages/index.html").expect("File system read failed."))
            .expect("Failed at utf decoding."),
        &ctx,
        true,
    )?;
    Ok(html)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_template() -> anyhow::Result<()> {
        let dates = vec![Date::new(String::from("dave")); 5];
        assert!(!template_load(dates.clone())?.contains("% for date in dates %"));
        assert!(template_load(dates.clone())?.contains("dave"));
        println!("{}", template_load(dates)?);
        Ok(())
    }
}
