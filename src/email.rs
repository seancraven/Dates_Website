use std::fs;

use anyhow::Context;
// File to manage accepting email_confirmation.
// I can use this an an excuse to make an email microservice.
use reqwest::Client;
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct EmailClient {
    c: Client,
    api_token: Secret<String>,
    app_url: String,
    from_email: String,
}
impl EmailClient {
    pub fn new(
        api_token: impl Into<String>,
        app_url: impl Into<String>,
        from_email: impl Into<String>,
    ) -> Self {
        EmailClient {
            c: Client::new(),
            api_token: Secret::new(api_token.into()),
            app_url: app_url.into(),
            from_email: from_email.into(),
        }
    }
    /// Send an email that.
    pub async fn send_auth_email(&self, user_email: &str) -> anyhow::Result<reqwest::Response> {
        let request = self
            .c
            .post("https://api.postmarkapp.com/email")
            .header("X-Postmark-Server-Token", self.api_token.expose_secret())
            .json(&PostMarkEmail::new_auth(
                user_email,
                &self.app_url,
                &self.from_email,
            )?)
            .build()?;
        println!("{:?}", request);
        let response = self.c.execute(request).await?;
        Ok(response)
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
    fn new_auth(
        user_email: &str,
        app_url: &str,
        from_email: &str,
    ) -> anyhow::Result<PostMarkEmail> {
        Ok(PostMarkEmail {
            html: render_email_html(&user_email, app_url)?,
            to: String::from(user_email),
            subject: "Date.rs Authentication".into(),
            from: from_email.into(),
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
    use toml;
    #[test]
    fn test_html_render() -> anyhow::Result<()> {
        let response_html = render_email_html("test@email.com", "test.com").unwrap();
        assert!(response_html.contains("test@email.com"));
        Ok(())
    }

    #[tokio::test]
    async fn test_client_construction() -> anyhow::Result<()> {
        let toml = toml::from_str::<toml::Value>(&fs::read_to_string("Secrets.dev.toml").unwrap())
            .unwrap();
        let key = toml.get("postmark_api_key").unwrap().as_str().unwrap();
        let email_from = toml.get("email_from").unwrap().as_str().unwrap();
        let url = toml.get("url").unwrap().as_str().unwrap();
        let _ = EmailClient::new(key, url, email_from);
        Ok(())
    }
}
