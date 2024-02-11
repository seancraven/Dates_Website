#[cfg(test)]
mod tests {
    use actix_web::http::StatusCode;
    use actix_web::test;
    use actix_web::web::{self, ServiceConfig};
    use actix_web::{web::Data, App};
    use chrono::{NaiveDate, NaiveTime};
    use date_rs::auth::user::GroupUser;
    use date_rs::auth::user::NoGroupUser;
    use date_rs::backend::postgres::PgRepo;
    use date_rs::domain::dates::Date;
    use date_rs::domain::repository::AppState;
    use date_rs::routes::landing::MainService;
    use sqlx::PgPool;
    use std::collections::HashMap;
    use uuid::Uuid;
    // TODO: Make tabular. At the moment this is much to long.
    fn start_tracing() {
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();
    }
    async fn get_pool() -> PgPool {
        PgPool::connect("postgres://postgres:assword@localhost:5432/postgres")
            .await
            .unwrap()
    }
    async fn mock_db() -> web::Data<AppState> {
        let state = AppState::new(Box::new(PgRepo {
            pool: get_pool().await,
        }));
        Data::new(state)
    }

    async fn mock_user(state: &AppState) -> anyhow::Result<GroupUser> {
        let mock_user = NoGroupUser {
            id: Uuid::new_v4(),
            email: String::from("integration@test.com"),
        };
        state.repo.create_user_and_group(mock_user).await
    }
    async fn mock_date(state: &AppState, user: &GroupUser) -> anyhow::Result<Date> {
        let mock_date = Date::new("test date");

        state.repo.add(mock_date.clone(), user.id).await?;
        state
            .repo
            .get(&mock_date.id, &user.id)
            .await
            .ok_or(anyhow::anyhow!("Date wans't found"))
    }

    async fn mock_db_user_date() -> anyhow::Result<(Data<AppState>, GroupUser, Date)> {
        let state = mock_db().await;
        let user = mock_user(&state).await?;
        let date = mock_date(&state, &user).await?;
        Ok((state, user, date))
    }
    fn get_mock_form() -> HashMap<String, String> {
        let mut form_data = HashMap::new();
        form_data.insert(
            "description_text".to_string(),
            "Test Description.".to_string(),
        );
        form_data.insert(
            "time".to_string(),
            NaiveTime::from_hms_opt(8, 0, 0).unwrap().to_string(),
        );
        form_data.insert(
            "day".to_string(),
            NaiveDate::from_ymd_opt(2020, 11, 1).unwrap().to_string(),
        );
        form_data
    }
    #[actix_web::test]
    async fn test_update_description_success() {
        // start_tracting();
        let (_, user, date) = mock_db_user_date().await.unwrap();

        let pool = get_pool().await;
        let app = test::init_service(App::new().configure(move |cfg: &mut ServiceConfig| {
            MainService::new(pool).service_configuration(cfg)
        }))
        .await;
        let form_data = get_mock_form();
        let uri = format!("/dates/{}/{}/description", user.id, date.id);
        let req = test::TestRequest::post().uri(&uri).set_form(form_data);
        let resp = test::call_service(&app, req.to_request()).await;
        assert!(resp.status().is_success());
    }
    #[actix_web::test]
    async fn test_update_description_contains_update() {
        // start_tracting();
        let (_, user, date) = mock_db_user_date().await.unwrap();
        let pool = get_pool().await;
        let app = test::init_service(App::new().configure(move |cfg: &mut ServiceConfig| {
            MainService::new(pool).service_configuration(cfg)
        }))
        .await;
        let form_data = get_mock_form();
        let uri = format!("/dates/{}/{}/description", user.id, date.id);
        let req = test::TestRequest::post().uri(&uri).set_form(form_data);
        let resp = test::call_and_read_body(&app, req.to_request()).await;
        let text = String::from_utf8(resp.to_vec()).unwrap();
        assert!(text.contains("Test Description."));
    }
    #[actix_web::test]
    async fn test_update_description_fails_with_empty_date() {
        // start_tracting();
        let (_, user, date) = mock_db_user_date().await.unwrap();
        let pool = get_pool().await;
        let app = test::init_service(App::new().configure(move |cfg: &mut ServiceConfig| {
            MainService::new(pool).service_configuration(cfg)
        }))
        .await;
        let mut form_data = get_mock_form();
        form_data.insert("day".to_string(), "".to_string());
        let uri = format!("/dates/{}/{}/description", user.id, date.id);
        let req = test::TestRequest::post().uri(&uri).set_form(form_data);
        assert!(test::call_service(&app, req.to_request())
            .await
            .status()
            .is_client_error());
    }
    #[actix_web::test]
    async fn test_update_description_fails_with_empty_time() {
        // start_tracting();
        let (_, user, date) = mock_db_user_date().await.unwrap();
        let pool = get_pool().await;
        let app = test::init_service(App::new().configure(move |cfg: &mut ServiceConfig| {
            MainService::new(pool).service_configuration(cfg)
        }))
        .await;
        let mut form_data = get_mock_form();
        form_data.insert("time".to_string(), "".to_string());
        let uri = format!("/dates/{}/{}/description", user.id, date.id);
        let req = test::TestRequest::post().uri(&uri).set_form(form_data);
        assert!(test::call_service(&app, req.to_request())
            .await
            .status()
            .is_client_error());
    }
    #[actix_web::test]
    async fn test_add_date_accept() {
        // start_tracting();
        let (_, user, _) = mock_db_user_date().await.unwrap();
        let pool = get_pool().await;
        let app = test::init_service(App::new().configure(move |cfg: &mut ServiceConfig| {
            MainService::new(pool).service_configuration(cfg)
        }))
        .await;
        let mut form = HashMap::new();
        form.insert("name".to_string(), "Test".to_string());
        let uri = format!("/dates/{}/new_date", user.id);
        let req = test::TestRequest::post().uri(&uri).set_form(form);
        assert!(test::call_service(&app, req.to_request())
            .await
            .status()
            .is_success());
    }
    #[actix_web::test]
    async fn test_add_date_fail() {
        // start_tracting();
        mock_db_user_date().await.unwrap();
        let pool = get_pool().await;
        let app = test::init_service(App::new().configure(move |cfg: &mut ServiceConfig| {
            MainService::new(pool).service_configuration(cfg)
        }))
        .await;
        let uri = format!("/dates/{}/new_date", Uuid::new_v4());
        let mut form = HashMap::new();
        form.insert("name".to_string(), "Test".to_string());
        let req = test::TestRequest::post().uri(&uri).set_form(form);
        assert_eq!(
            test::call_service(&app, req.to_request()).await.status(),
            StatusCode::UNAUTHORIZED
        );
    }
    #[actix_web::test]
    async fn test_add_date_forbidden() {
        // start_tracting();
        let (_, user, _) = mock_db_user_date().await.unwrap();
        let pool = get_pool().await;
        let app = test::init_service(App::new().configure(move |cfg: &mut ServiceConfig| {
            MainService::new(pool).service_configuration(cfg)
        }))
        .await;
        let mut form = HashMap::new();
        form.insert("name".to_string(), "".to_string());
        let uri = format!("/dates/{}/new_date", user.id);
        let req = test::TestRequest::post().uri(&uri).set_form(form);
        assert_eq!(
            test::call_service(&app, req.to_request()).await.status(),
            StatusCode::FORBIDDEN
        );
    }

    #[actix_web::test]
    async fn test_index() {
        // start_tracting();
        let (_, user, _) = mock_db_user_date().await.unwrap();
        let pool = get_pool().await;
        let app = test::init_service(App::new().configure(move |cfg: &mut ServiceConfig| {
            MainService::new(pool).service_configuration(cfg)
        }))
        .await;
        let req = test::TestRequest::get()
            .uri(&format!("/dates/{}", user.id))
            .to_request();
        let resp = test::call_service(&app, req).await.status();
        assert_eq!(resp, StatusCode::OK);
    }
    #[actix_web::test]
    async fn test_login() {
        // start_tracting();
        let pool = get_pool().await;
        let app = test::init_service(App::new().configure(move |cfg: &mut ServiceConfig| {
            MainService::new(pool).service_configuration(cfg)
        }))
        .await;
        let mut form = HashMap::new();
        form.insert("email".to_string(), "test@test.com");
        form.insert("password".to_string(), "assword");
        let req = test::TestRequest::post()
            .uri("/login")
            .set_form(&form)
            .to_request();
        tracing::info!("Sending request.");
        let resp = test::call_service(&app, req).await.status();
        assert_eq!(resp, StatusCode::OK);
    }
    #[actix_web::test]
    async fn test_register() {
        // start_tracting();
        let pool = get_pool().await;
        let app = test::init_service(App::new().configure(move |cfg: &mut ServiceConfig| {
            MainService::new(pool).service_configuration(cfg)
        }))
        .await;
        let mut form = HashMap::new();
        form.insert("email".to_string(), "test@test.com");
        form.insert("password".to_string(), "assword");
        let req = test::TestRequest::post()
            .uri("/register")
            .set_form(&form)
            .to_request();
        let resp = test::call_service(&app, req).await.status();
        assert_eq!(resp, StatusCode::OK);
    }
}
