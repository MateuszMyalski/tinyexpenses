use crate::api_token;
use crate::{account::Account, models};
use axum::{
    extract::{self, Request},
    http::{StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use axum_login::{AuthUser, AuthnBackend, UserId};
use std::{
    error::Error,
    path::{Path, PathBuf},
};
use tracing::log::{debug, error, info, warn};

#[derive(Debug, Clone)]
pub struct UsersBackend {
    pub users_path: PathBuf,
}

pub type AuthSession = axum_login::AuthSession<UsersBackend>;

impl UsersBackend {
    pub fn new(users_path: &Path) -> Result<Self, Box<dyn Error>> {
        if !users_path.is_dir() {
            error!("Provided path for users db must be a directory.");
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotADirectory,
                "Proveided user_path does not exists.",
            )
            .into());
        }

        info!("Registered backend directory.");

        Ok(UsersBackend {
            users_path: users_path.to_path_buf(),
        })
    }

    pub fn load(&self, username: &str) -> Option<Account> {
        let user_dir = self.users_path.join(username);
        match Account::new(user_dir.as_path()) {
            Ok(a) => Some(a),
            Err(err) => {
                warn!("Cannot read user {username}");
                debug!("{}", err);
                None
            }
        }
    }
}

impl AuthUser for Account {
    type Id = String;

    fn id(&self) -> Self::Id {
        String::from(self.config.user_username())
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.config.user_password_hash().as_bytes()
    }
}

impl AuthnBackend for UsersBackend {
    type User = Account;
    type Credentials = models::templates::account::LoginForm;
    type Error = std::convert::Infallible;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let account = match self
            .get_user(&creds.username)
            .await
            .expect("The error is infallible.")
        {
            Some(a) => a,
            None => return Ok(None),
        };

        if !account.config.user_active() {
            warn!("User {} inactive.", creds.username);
            return Ok(None);
        }

        if account.config.user_password_verify(&creds.password) {
            Ok(Some(account.clone()))
        } else {
            warn!("Incorrect password for {}.", creds.username);
            Ok(None)
        }
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        Ok(self.load(user_id))
    }
}

pub async fn require_auth(
    auth_session: AuthSession,
    mut request: Request,
    next: Next,
) -> Result<Response, Redirect> {
    match auth_session.user {
        Some(user) => {
            request.extensions_mut().insert(user);
            Ok(next.run(request).await)
        }
        None => Err(Redirect::to("/account/login")),
    }
}

pub struct ApiResponsePayload {
    msg: &'static str,
    code: StatusCode,
}

impl ApiResponsePayload {
    const OK_MSG: &str = "OK";
    const NOK_MSG: &str = "NOK";

    pub fn new() -> Self {
        ApiResponsePayload {
            msg: "",
            code: StatusCode::IM_A_TEAPOT,
        }
    }
    pub fn as_ok(mut self) -> Self {
        self.msg = &ApiResponsePayload::OK_MSG;
        self.code = StatusCode::OK;

        self
    }

    pub fn as_unauthorized(mut self) -> Self {
        self.msg = &ApiResponsePayload::NOK_MSG;
        self.code = StatusCode::UNAUTHORIZED;

        self
    }

    pub fn as_nok(mut self) -> Self {
        self.msg = &ApiResponsePayload::NOK_MSG;
        self.code = StatusCode::INTERNAL_SERVER_ERROR;

        self
    }
}

impl IntoResponse for ApiResponsePayload {
    fn into_response(self) -> Response {
        let body = format!("{{ \"status\" = \"{}\" }}", self.msg);
        (self.code, body).into_response()
    }
}

pub async fn require_api_auth(
    headers: header::HeaderMap,
    auth_session: AuthSession,
    extract::Path(username): extract::Path<String>,
    mut request: Request,
    next: Next,
) -> Response {
    let user = match auth_session.backend.load(&username) {
        Some(u) => u,
        None => return ApiResponsePayload::new().as_unauthorized().into_response(),
    };

    let x_api_key = match headers.get("X-API-Key") {
        Some(x) => x,
        None => return ApiResponsePayload::new().as_unauthorized().into_response(),
    }
    .to_str();

    let x_api_key = match x_api_key {
        Ok(x) => x,
        Err(err) => {
            warn!("Unable to read X-API-Key");
            debug!("{err}");
            return ApiResponsePayload::new().as_unauthorized().into_response();
        }
    };

    if !api_token::verify(x_api_key, user.config.api_token()) {
        warn!("Invalid verification to {username}");
        return ApiResponsePayload::new().as_unauthorized().into_response();
    }

    request.extensions_mut().insert(user);
    next.run(request).await
}
