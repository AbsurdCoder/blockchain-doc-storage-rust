use actix_web::{
    dev::Payload,
    error::ErrorUnauthorized,
    http::header,
    web::Data,
    Error as ActixError, FromRequest, HttpRequest,
};
use futures::future::{ready, Ready};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

use crate::models::Claims;
use crate::AppState;

/// Extractor representing an authenticated user based on a validated JWT.
///
/// Handlers can require authentication by adding this as an argument:
/// `async fn protected_route(user: AuthenticatedUser, ...) -> impl Responder { ... }`
#[derive(Debug, Clone)]
pub struct AuthenticatedUser(pub Claims);

impl AuthenticatedUser {
    pub fn user_id(&self) -> &str {
        &self.0.sub
    }

    pub fn role(&self) -> &str {
        &self.0.role
    }

    pub fn claims(&self) -> &Claims {
        &self.0
    }

    pub fn is_admin(&self) -> bool {
        self.0.role.eq_ignore_ascii_case("admin")
    }
}

impl FromRequest for AuthenticatedUser {
    type Error = ActixError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        // Get the Authorization header
        let auth_header = match req.headers().get(header::AUTHORIZATION) {
            Some(h) => h,
            None => {
                return ready(Err(ErrorUnauthorized("Missing Authorization header")));
            }
        };

        let auth_str = match auth_header.to_str() {
            Ok(s) => s,
            Err(_) => {
                return ready(Err(ErrorUnauthorized("Invalid Authorization header")));
            }
        };

        // Expect a Bearer token: "Bearer <jwt>"
        let token = match auth_str.strip_prefix("Bearer ") {
            Some(t) if !t.is_empty() => t,
            _ => {
                return ready(Err(ErrorUnauthorized(
                    "Authorization header must be in the format: Bearer <token>",
                )));
            }
        };

        // Get application state to access the JWT secret
        let state = match req.app_data::<Data<AppState>>() {
            Some(data) => data.clone(),
            None => {
                return ready(Err(ErrorUnauthorized("Application state not available")));
            }
        };

        let jwt_secret = &state.jwt_secret;

        // Validate and decode the JWT
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;

        let token_data = match decode::<Claims>(
            token,
            &DecodingKey::from_secret(jwt_secret.as_bytes()),
            &validation,
        ) {
            Ok(data) => data,
            Err(_) => {
                return ready(Err(ErrorUnauthorized("Invalid or expired token")));
            }
        };

        ready(Ok(AuthenticatedUser(token_data.claims)))
    }
}

