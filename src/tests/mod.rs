#[cfg(test)]
mod tests {
    use crate::network::api::root_handler;
    use crate::app_state::AppState;
    use crate::player::Player;
    use crate::network::routes::{lobby_handler, LobbyResponse};
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
        // assert that the response game ID is null since we only have one player
        assert_eq!(lobby_response.game, "waiting_for_players");
    }

    #[tokio::test]
    async fn test_lobby_with_multiple_players() {
        let app_state = AppState::new();
        let app = axum::Router::new()
            .route("/lobby", post(lobby_handler))
            .with_state(app_state.clone());

        let player1 = Player {
            name: "Player1".to_string(),
            team: "TeamA".to_string(),
            password: "secret1".to_string(),
            game_id: None,
        };

        let player2 = Player {
            name: "Player2".to_string(),
            team: "TeamA".to_string(),
            password: "secret2".to_string(),
            game_id: None,
        };

        let response1 = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/lobby")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::from(serde_json::to_string(&player1).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response1.status(), StatusCode::OK);

        let response2 = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/lobby")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::from(serde_json::to_string(&player2).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response2.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_state_endpoint() {
        let app_state = AppState::new();
        let app = axum::Router::new()
            .route("/state", get(crate::network::routes::state_handler))
            .with_state(app_state.clone());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/state")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
