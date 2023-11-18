use actix_web::{get, HttpResponse, Responder};
use serde::Serialize;
use std::fs::read;
use tera::{self, Context};

#[get("/")]
async fn index() -> impl Responder {
    let dates = vec![
        Date {
            name: String::from("placeholder"),
        };
        5
    ];
    HttpResponse::Ok().body(template_load(dates).expect("Templating failed."))
}

#[derive(Debug, Serialize, Clone)]
struct Date {
    name: String,
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
        let dates = vec![
            Date {
                name: String::from("dave")
            };
            5
        ];
        assert!(!template_load(dates.clone())?.contains("% for date in dates %"));
        assert!(template_load(dates)?.contains("dave"));
        Ok(())
    }
}
