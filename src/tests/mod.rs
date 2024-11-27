#[cfg(test)]
mod tests {
    use crate::app_state::AppState;
    use crate::player::Player;
    use crate::routes::{lobby_handler, root_handler, LobbyResponse};
    use axum::routing::{get, post};
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_root_handler() {
        let app = axum::Router::new().route("/", get(root_handler));

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"Welcome to Space Race!");
    }

    #[tokio::test]
    async fn test_lobby_handler() {
        let app_state = AppState::new();
        let app = axum::Router::new()
            .route("/lobby", post(lobby_handler))
            .with_state(app_state.clone());

        let player = Player {
            name: "TestPlayer".to_string(),
            team: "TestTeam".to_string(),
            password: "secret".to_string(),
            game_id: None,
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/lobby")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::from(serde_json::to_string(&player).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let lobby_response: LobbyResponse = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(lobby_response.name, "TestPlayer");
        assert_eq!(lobby_response.map, "default_map");
        assert!(Uuid::parse_str(&lobby_response.game).is_ok());
    }
}
