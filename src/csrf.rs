use axum_csrf::{CsrfConfig, CsrfLayer, CsrfToken};
use tower_sessions::Session;
use tracing::error;

pub fn create_layer() -> CsrfLayer {
    let config = CsrfConfig::default();
    CsrfLayer::new(config)
}

async fn set_token(session: &Session, token: &str) {
    session
        .insert("cfsr_authenticity_token", &token)
        .await
        .unwrap();
}

pub async fn regenerate_token(session: &Session, token: &CsrfToken) -> String {
    match token.authenticity_token() {
        Ok(t) => {
            set_token(session, &t).await;
            t
        }
        Err(err) => {
            // TODO should we panic here?
            error!("Unable to retrieve csrf token: '{:?}'", err);
            set_token(session, "").await;
            "".to_string()
        }
    }
}

pub async fn remove_token(session: &Session) {
    let _ = session
        .remove::<String>("cfsr_authenticity_token")
        .await
        .unwrap();
}

async fn get_token(session: &Session) -> Result<String, Box<dyn std::error::Error>> {
    match session
        .get::<String>("cfsr_authenticity_token")
        .await
        .unwrap()
    {
        Some(t) => Ok(t),
        None => Err("CFSR token not found in session!".into()),
    }
}

pub async fn verify(session: &Session, token: &CsrfToken, payload_token: &str) -> bool {
    let Ok(stored_token) = get_token(session).await else {
        return false;
    };

    token.verify(payload_token).is_ok() && token.verify(&stored_token).is_ok()
}
