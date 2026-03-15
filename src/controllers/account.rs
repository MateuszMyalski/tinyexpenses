use crate::auth::require_auth;
use axum::{
    Router,
    routing::{get, post},
};

pub fn router() -> Router {
    let protected = Router::new()
        .route("/logout", get(self::get::logout))
        .route("/view", get(self::get::view))
        .route("/change/details", post(self::post::change_details))
        .route("/change/password", post(self::post::change_password))
        .route("/change/token", post(self::post::change_token))
        .route("/change/color", post(self::post::change_color_scheme))
        .route_layer(axum::middleware::from_fn(require_auth));

    let unprotected = Router::new()
        .route("/login", get(self::get::login))
        .route("/login", post(self::post::login));

    Router::new().merge(protected).merge(unprotected)
}

pub mod get {
    use crate::account::Account;
    use crate::auth::AuthSession;
    use crate::models::templates::{WebPageContext, WebPageToHtml, account};
    use crate::redirection;
    use axum::{Extension, extract::OriginalUri, response::IntoResponse};
    use axum_csrf::CsrfToken;
    use axum_messages::Messages;
    use tower_sessions::Session;
    use tracing::log::{debug, error};

    pub async fn login(
        csrf_token: CsrfToken,
        messages: Messages,
        session: Session,
    ) -> impl IntoResponse {
        let view = account::LoginTmpl {
            ctx: &WebPageContext::new("- Login", &session)
                .await
                .with_messages(messages)
                .with_csrf(&csrf_token)
                .await,
        }
        .to_html();

        (csrf_token, view).into_response()
    }

    pub async fn logout(messages: Messages, mut auth_session: AuthSession) -> impl IntoResponse {
        if let Err(err) = auth_session.logout().await {
            error!("Cannot logout user.");
            debug!("{}", err);
            messages.error("Unable to logout user.");
        }

        redirection::to_index().into_response()
    }

    pub async fn view(
        messages: Messages,
        csrf_token: CsrfToken,
        session: Session,
        auth_session: AuthSession,
        OriginalUri(hostname): OriginalUri,
        Extension(user): Extension<Account>,
    ) -> impl IntoResponse {
        let api_token = auth_session
            .session
            .get::<String>("api-token")
            .await
            .unwrap_or(None);

        let view = account::ViewTmpl {
            ctx: &WebPageContext::new("- View account", &session)
                .await
                .with_account(&user)
                .with_messages(messages)
                .with_csrf(&csrf_token)
                .await,
            api_token: api_token.as_deref(),
            host_url: hostname.host().unwrap_or("https://(your domain)"),
        };

        view.to_html().into_response()
    }
}

pub mod post {
    use crate::account::Account;
    use crate::api_token;
    use crate::auth::AuthSession;
    use crate::csrf;
    use crate::models::templates::{WebPageContext, WebPageToHtml, account};
    use crate::redirection;
    use crate::storage::BasicRepository;
    use axum::{Extension, Form, response::IntoResponse};
    use axum_csrf::CsrfToken;
    use axum_messages::Messages;
    use tower_sessions::Session;
    use tracing::{debug, error};

    pub async fn login(
        csrf_token: CsrfToken,
        messages: Messages,
        session: Session,
        mut auth_session: AuthSession,
        Form(form): Form<account::LoginForm>,
    ) -> impl IntoResponse {
        if !csrf::verify(&session, &csrf_token, &form.csrf_token).await {
            messages.error("CSRF token is invalid.");
            return redirection::to_index().into_response();
        }

        csrf::remove_token(&session).await;

        let user = match auth_session.authenticate(form.clone()).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                let view = account::InvalidLoginTmpl {
                    ctx: &WebPageContext::new("- Error login", &session)
                        .await
                        .with_messages(messages),
                };
                return view.to_html().into_response();
            }
            Err(_) => {
                messages.error("Cannot authenticate user.");
                return redirection::to_index().into_response();
            }
        };

        if let Err(err) = auth_session.login(&user).await {
            error!("Cannot login user.");
            debug!("{}", err);
            messages.error("Cannot login user.");
        }

        let _ = auth_session
            .session
            .insert("dark_color_scheme", user.config.dark_color_scheme())
            .await;

        redirection::to_index().into_response()
    }

    pub async fn change_details(
        messages: Messages,
        csrf_token: CsrfToken,
        session: Session,
        Extension(mut user): Extension<Account>,
        Form(form): Form<account::ViewDetailsForm>,
    ) -> impl IntoResponse {
        if !csrf::verify(&session, &csrf_token, &form.csrf_token).await {
            messages.error("CSRF token is invalid.");
            return redirection::to_account_view().into_response();
        }

        csrf::remove_token(&session).await;

        user.config.set_currency(&form.currency);
        user.config.set_user_full_name(&form.user_full_name);

        if let Err(err) = user.config.write() {
            messages.error("Cannot edit details - cannot store config.");
            debug!("{}", err);
            return redirection::to_account_view().into_response();
        }

        messages.success("Details changed.");
        redirection::to_account_view().into_response()
    }

    pub async fn change_color_scheme(
        messages: Messages,
        csrf_token: CsrfToken,
        session: Session,
        Extension(mut user): Extension<Account>,
        Form(form): Form<account::ViewChangeColorSchemeForm>,
    ) -> impl IntoResponse {
        if !csrf::verify(&session, &csrf_token, &form.csrf_token).await {
            messages.error("CSRF token is invalid.");
            return redirection::to_account_view().into_response();
        }

        csrf::remove_token(&session).await;

        if form.dark_color_scheme {
            user.config.set_dark_color_scheme();
        } else {
            user.config.set_light_color_scheme();
        }

        if let Err(err) = user.config.write() {
            messages.error("Failed changing color scheme - cannot store config.");
            debug!("{}", err);
            return redirection::to_account_view().into_response();
        }

        if form.dark_color_scheme {
            messages.success("Color scheme changed to dark.");
        } else {
            messages.success("Color scheme changed to light.");
        }

        let _ = session
            .insert("dark_color_scheme", user.config.dark_color_scheme())
            .await;

        redirection::to_account_view().into_response()
    }

    pub async fn change_password(
        messages: Messages,
        csrf_token: CsrfToken,
        session: Session,
        Extension(mut user): Extension<Account>,
        Form(form): Form<account::ViewChangePasswordForm>,
    ) -> impl IntoResponse {
        if !csrf::verify(&session, &csrf_token, &form.csrf_token).await {
            messages.error("CSRF token is invalid.");
            return redirection::to_account_view().into_response();
        }

        csrf::remove_token(&session).await;

        if !user.config.user_password_hash().is_empty()
            && !user.config.user_password_verify(&form.password)
        {
            messages.error("Incorrect password.");
            return redirection::to_account_view().into_response();
        }

        if form.new_password != form.confirm_password {
            messages.error("New password does not match.");
            return redirection::to_account_view().into_response();
        }

        if let Err(err) = user.config.set_user_password_hash(&form.new_password) {
            error!("Cannot set user password.");
            debug!("{}", err);
            messages.error("Failed changing password - hashing error.");
            return redirection::to_account_view().into_response();
        }

        if let Err(err) = user.config.write() {
            messages.error("Failed changing password - cannot store config.");
            debug!("{}", err);
            return redirection::to_account_view().into_response();
        }

        messages.success("Password changed.");

        redirection::to_account_view().into_response()
    }

    pub async fn change_token(
        messages: Messages,
        csrf_token: CsrfToken,
        session: Session,
        auth_session: AuthSession,
        Extension(mut user): Extension<Account>,
        Form(form): Form<account::ViewApiTokenForm>,
    ) -> impl IntoResponse {
        if !crate::csrf::verify(&session, &csrf_token, &form.csrf_token).await {
            messages.error("CSRF token is invalid.");
            return redirection::to_account_view().into_response();
        }

        csrf::remove_token(&session).await;

        if let Err(err) = user.config.generate_token() {
            error!("Cannot generate api token.");
            debug!("{}", err);
            messages.error("Failed generating token.");
            return redirection::to_account_view().into_response();
        }

        if let Err(err) = user.config.write() {
            messages.error("Failed generating token - cannot store config.");
            debug!("{}", err);
            return redirection::to_account_view().into_response();
        }

        if let Err(err) = auth_session
            .session
            .insert("api-token", api_token::sign(user.config.api_token()))
            .await
        {
            error!("Cannot sign user api token.");
            debug!("{}", err);
            messages.warning("Error while accessing session.");
            return redirection::to_account_view().into_response();
        }

        messages.success("Token generated.");
        redirection::to_account_view().into_response()
    }
}
