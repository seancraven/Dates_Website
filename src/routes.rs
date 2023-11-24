use actix_web::web;
use actix_web::{get, HttpResponse, Responder};
use serde::Serialize;
use std::fs::read;
use std::sync::Mutex;
use tera::{self, Context, Tera};

type State = Mutex<DateRepo>;
#[get("/")]
async fn index(repo: web::Data<State>) -> impl Responder {
    log::info!("Serving index page");
    HttpResponse::Ok()
        .body(template_load(repo.lock().unwrap().get_all()).expect("Templating failed."))
}

// #[derive(Deserialize, Debug, Serialize)]
// struct DateInfo {
//     name: String,
// }

#[get("/date_button/{date_info}/{index}")]
async fn update_date(
    date_info: web::Path<(String, usize)>,
    app_state: web::Data<State>,
) -> impl Responder {
    let date_name = &date_info.0;
    let idx = date_info.1;
    log::info!("Date button pushed on: {:?}", &date_name);
    match app_state.lock() {
        Ok(mut repo) => {
            // log::debug!("Responding with: {:?}", repo.get_all());
            let mut ctx = Context::new();
            ctx.insert("date", &repo.get(date_name).expect("Date doesnt exitst"));
            ctx.insert("index", &idx);

            let html = Tera::one_off(
                std::str::from_utf8(
                    &read("./pages/button.html").expect("File system read failed."),
                )
                .unwrap(),
                &ctx,
                false,
            );
            repo.update_count(date_name);
            HttpResponse::Ok().body(html.unwrap())
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
#[derive(Debug, Serialize, Clone)]
pub struct Date {
    name: String,
    count: usize,
}
impl Date {
    pub fn new(name: String) -> Date {
        Date { name, count: 0 }
    }
    fn add(&mut self) {
        self.count += 1;
    }
}

trait Repository {
    fn add(&mut self, date: Date);
    fn remove(&mut self, date: Date);
    fn get_all(&self) -> Vec<Date>;
    fn update_count(&mut self, date_name: &str);
    fn get(&self, date_name: &str) -> Option<Date>;
}

pub struct DateRepo {
    pub dates: Vec<Date>,
}
impl Repository for DateRepo {
    fn add(&mut self, date: Date) {
        self.dates.push(date);
    }
    fn get(&self, date_name: &str) -> Option<Date> {
        for date in &self.dates {
            if date.name == date_name {
                return Some(date.clone());
            }
        }
        None
    }
    fn get_all(&self) -> Vec<Date> {
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
