use std::fs;

use anyhow::Context;
// File to manage accepting email_confirmation.
// I can use this an an excuse to make an email microservice.
use reqwest::Client;
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};

use crate::auth::user::UnauthorizedUser;

struct EmailClient {
    c: Client,
    api_token: Secret<String>,
    app_url: String,
}
impl EmailClient {
    fn new(api_token: impl Into<String>, app_url: impl Into<String>) -> Self {
        EmailClient {
            c: Client::new(),
            api_token: Secret::new(api_token.into()),
            app_url: app_url.into(),
        }
    }
    /// Send an email that.
    async fn send_auth_email(
        &self,
        user: UnauthorizedUser,
        app_url: &str,
    ) -> anyhow::Result<reqwest::Response> {
        let request = self
            .c
            .post("https://api.postmarkapp.com/email")
            .header("X-Postmark-Server-Token", self.api_token.expose_secret())
            .json(&PostMarkEmail::new_auth(user, app_url)?)
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
    use secrecy::Secret;
    use toml;
    #[test]
    fn test_html_render() -> anyhow::Result<()> {
        let response_html = render_email_html("test@email.com", "test.com").unwrap();
        assert!(response_html.contains("test@email.com"));
        Ok(())
    }

    #[tokio::test]
    async fn test_client() -> anyhow::Result<()> {
        let toml = toml::from_str::<toml::Value>(&fs::read_to_string("Secrets.dev.toml").unwrap())
            .unwrap();
        let key = toml.get("postmark_api_key").unwrap().as_str().unwrap();
        let email_from = toml.get("email_from").unwrap().as_str().unwrap();
        let url = toml.get("url").unwrap().as_str().unwrap();
        let client = EmailClient::new(key, url);
        let user = UnauthorizedUser {
            email: String::from(email_from),
            password: Secret::new(String::from("assword")),
        };
        let response = client.send_auth_email(user, email_from).await?;
        println!("{:?}", response);
        assert!(response.status().is_success());
        Ok(())
    }
}
