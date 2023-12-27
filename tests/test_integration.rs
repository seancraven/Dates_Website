#[cfg(test)]
mod tests {
    use actix_web::http::StatusCode;
    use actix_web::test;
    use actix_web::web;
    use actix_web::{web::Data, App};
    use chrono::{NaiveDate, NaiveTime};
    use dates::domain::dates::Date;
    use dates::domain::repository::{AppState, VecRepo};
    use dates::routes::dates_service::{add_new_date, update_description};
    use dates::routes::index::index;
    use std::collections::HashMap;
    use uuid::Uuid;
    async fn mock_db() -> (web::Data<AppState>, Uuid, Uuid) {
        let state = AppState::new(Box::new(VecRepo::new()));
        let mock_id = Uuid::new_v4();
        let date = Date::new("Test");
        let date_id = date.id;
        state.repo.add(date, mock_id).await.expect("Add");
        state.repo.get(&date_id, &mock_id).await.unwrap();
        (Data::new(state), mock_id, date_id)
    }
    fn start_tracing() {
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();
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
        start_tracing();
        let (state, user_id, date_id) = mock_db().await;
        let app = test::init_service(App::new().app_data(state).service(update_description)).await;
        let form_data = get_mock_form();
        let uri = format!("/{}/{}/description", user_id, date_id);
        let req = test::TestRequest::post().uri(&uri).set_form(form_data);
        let resp = test::call_service(&app, req.to_request()).await;
        assert!(resp.status().is_success());
    }
    #[actix_web::test]
    async fn test_update_description_contains_update() {
        start_tracing();
        let (state, user_id, date_id) = mock_db().await;
        let app = test::init_service(App::new().app_data(state).service(update_description)).await;
        let form_data = get_mock_form();
        let uri = format!("/{}/{}/description", user_id, date_id);
        let req = test::TestRequest::post().uri(&uri).set_form(form_data);
        let resp = test::call_and_read_body(&app, req.to_request()).await;
        let text = String::from_utf8(resp.to_vec()).unwrap();
        assert!(text.contains("Test Description."));
    }
    #[actix_web::test]
    async fn test_update_description_fails_with_empty_date() {
        start_tracing();
        let (state, user_id, date_id) = mock_db().await;
        let app = test::init_service(App::new().app_data(state).service(update_description)).await;
        let mut form_data = get_mock_form();
        form_data.insert("day".to_string(), "".to_string());
        let uri = format!("/{}/{}/description", user_id, date_id);
        let req = test::TestRequest::post().uri(&uri).set_form(form_data);
        assert!(test::call_service(&app, req.to_request())
            .await
            .status()
            .is_client_error());
    }
    #[actix_web::test]
    async fn test_update_description_fails_with_empty_time() {
        start_tracing();
        let (state, user_id, date_id) = mock_db().await;
        let app = test::init_service(App::new().app_data(state).service(update_description)).await;
        let mut form_data = get_mock_form();
        form_data.insert("time".to_string(), "".to_string());
        let uri = format!("/{}/{}/description", user_id, date_id);
        let req = test::TestRequest::post().uri(&uri).set_form(form_data);
        assert!(test::call_service(&app, req.to_request())
            .await
            .status()
            .is_client_error());
    }
    #[actix_web::test]
    async fn test_add_date_accept() {
        start_tracing();
        let (state, user_id, _) = mock_db().await;
        let app = test::init_service(App::new().app_data(state).service(add_new_date)).await;
        let mut form = HashMap::new();
        form.insert("new_date".to_string(), "Test".to_string());
        let uri = format!("/{}/new_date", user_id);
        let req = test::TestRequest::post().uri(&uri).set_form(form);
        assert!(test::call_service(&app, req.to_request())
            .await
            .status()
            .is_success());
    }
    #[actix_web::test]
    async fn test_add_date_fail() {
        start_tracing();
        let (state, _, _) = mock_db().await;
        let app = test::init_service(App::new().app_data(state).service(add_new_date)).await;
        let uri = format!("/{}/new_date", Uuid::new_v4());
        let mut form = HashMap::new();
        form.insert("new_date".to_string(), "Test".to_string());
        let req = test::TestRequest::post().uri(&uri).set_form(form);
        assert_eq!(
            test::call_service(&app, req.to_request()).await.status(),
            StatusCode::UNAUTHORIZED
        );
    }
    #[actix_web::test]
    async fn test_add_date_forbidden() {
        start_tracing();
        let (state, user_id, _) = mock_db().await;
        let app = test::init_service(App::new().app_data(state).service(add_new_date)).await;
        let mut form = HashMap::new();
        form.insert("new_date".to_string(), "".to_string());
        let uri = format!("/{}/new_date", user_id);
        let req = test::TestRequest::post().uri(&uri).set_form(form);
        assert_eq!(
            test::call_service(&app, req.to_request()).await.status(),
            StatusCode::FORBIDDEN
        );
    }

    #[actix_web::test]
    async fn test_index() {
        start_tracing();
        let (state, user_id, _) = mock_db().await;
        let app = test::init_service(App::new().app_data(state).service(index)).await;
        let req = test::TestRequest::get()
            .uri(&format!("/{}", user_id))
            .to_request();
        let resp = test::call_service(&app, req).await.status();
        assert_eq!(resp, StatusCode::OK);
    }
}
