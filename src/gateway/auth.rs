use std::env;

use axum::body::Body;
use axum::extract::Request;
use axum::http::{StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use subtle::ConstantTimeEq;

use crate::errors::AppError;

const USER_ENV: &str = "POIS_ADMIN_USER";
const PASS_ENV: &str = "POIS_ADMIN_PASS";
const REALM: &str = "pois";

#[derive(Clone, Debug)]
pub struct BasicAuth {
    user: String,
    pass: String,
}

impl BasicAuth {
    pub fn from_env() -> Result<Self, AppError> {
        let user = non_empty_env(USER_ENV)?;
        let pass = non_empty_env(PASS_ENV)?;
        Ok(Self { user, pass })
    }

    fn verify(&self, candidate_user: &[u8], candidate_pass: &[u8]) -> bool {
        // ConstantTimeEq returns false on length mismatch, so we xor both
        // comparisons and require both to be true without early exit.
        let user_ok: bool = self.user.as_bytes().ct_eq(candidate_user).into();
        let pass_ok: bool = self.pass.as_bytes().ct_eq(candidate_pass).into();
        user_ok & pass_ok
    }
}

fn non_empty_env(key: &'static str) -> Result<String, AppError> {
    match env::var(key) {
        Ok(value) if !value.is_empty() => Ok(value),
        _ => Err(AppError::MissingEnv(key)),
    }
}

pub async fn middleware(
    axum::extract::State(auth): axum::extract::State<BasicAuth>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if authorised(&auth, &request) {
        return next.run(request).await;
    }
    unauthorized()
}

fn authorised(auth: &BasicAuth, request: &Request<Body>) -> bool {
    let Some(header_value) = request.headers().get(header::AUTHORIZATION) else {
        return false;
    };
    let Ok(header_str) = header_value.to_str() else {
        return false;
    };
    let Some(encoded) = header_str.strip_prefix("Basic ") else {
        return false;
    };
    let Ok(decoded) = STANDARD.decode(encoded.trim()) else {
        return false;
    };
    let Some(separator) = decoded.iter().position(|b| *b == b':') else {
        return false;
    };
    let (user, pass_with_colon) = decoded.split_at(separator);
    let pass = &pass_with_colon[1..];
    auth.verify(user, pass)
}

fn unauthorized() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        [(header::WWW_AUTHENTICATE, format!("Basic realm=\"{REALM}\""))],
        "unauthorized",
    )
        .into_response()
}
