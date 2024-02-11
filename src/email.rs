use std::fs;

use anyhow::Context;
// File to manage accepting email_confirmation.
// I can use this an an excuse to make an email microservice.
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::auth::user::UnauthorizedUser;

struct EmailClient {
    c: Client,
}
impl EmailClient {
    /// Send an email that.
    async fn send_auth_email(&self, user: UnauthorizedUser, app_url: &str) -> anyhow::Result<()> {
        self.c
            .post("https://api.postmarkapp.com/email")
            .json(&PostMarkEmail::new_auth(user, app_url)?)
            .send()
            .await?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct PostMarkEmail {
    #[serde(rename = "From")]
    from: String,
    #[serde(rename = "To")]
    to: String,
    #[serde(rename = "Subject")]
    subject: String,
    #[serde(rename = "HtmlBody")]
    html: String,
}
impl PostMarkEmail {
    fn new_auth(user: UnauthorizedUser, app_url: &str) -> anyhow::Result<PostMarkEmail> {
        Ok(PostMarkEmail {
            html: render_email_html(&user.email, app_url)?,
            to: user.email,
            subject: "Date.rs Authentication".into(),
            from: "sean.craven.22@ucl.ac.uk".into(),
        })
    }
}

fn render_email_html(email: &str, app_url: &str) -> anyhow::Result<String> {
    let mut ctx = tera::Context::new();
    ctx.insert("daters_url", app_url);
    ctx.insert(
        "authentication_url",
        &format!("{}/{}/{}", app_url, "authenticate", email),
    );
    tera::Tera::one_off(
        &fs::read_to_string("./pages/welcome_email.html")?,
        &ctx,
        false,
    )
    .context("Rendering email template failed.")
}
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_html_render() -> anyhow::Result<()> {
        let response_html = render_email_html("test@email.com", "test.com").unwrap();
        assert!(response_html.contains("test@email.com"));
        Ok(())
    }
}
