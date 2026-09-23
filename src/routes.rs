use std::time::{SystemTime, UNIX_EPOCH};

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{
    Json,
    extract::{Query, State},
    http::{
        HeaderMap, HeaderValue, StatusCode,
        header::{COOKIE, SET_COOKIE},
    },
    response::{IntoResponse, Response},
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use mongodb::{
    bson::{Document, doc},
    error::{Error as MongoError, ErrorKind, WriteFailure},
};
use serde::{Deserialize, Serialize};

use crate::models::{
    AppState, LoginRequest, LoginResponse, MessageResponse, PublicUser, SignupRequest, UsersQuery,
    UsersResponse,
};

const MIN_PASSWORD_LENGTH: usize = 8;
const MAX_PASSWORD_LENGTH: usize = 128;
const DEFAULT_PAGE_SIZE: i64 = 50;
const MAX_PAGE_SIZE: i64 = 100;
const SESSION_COOKIE: &str = "session";

pub(crate) struct ApiError {
    status: StatusCode,
    message: &'static str,
}

impl ApiError {
    const fn bad_request(message: &'static str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message,
        }
    }

    const fn unauthorized() -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: "Authentication is required.",
        }
    }

    const fn conflict(message: &'static str) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            message,
        }
    }

    const fn internal() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "An internal server error occurred.",
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(MessageResponse {
                message: self.message.to_string(),
            }),
        )
            .into_response()
    }
}

pub async fn signup(
    State(state): State<AppState>,
    Json(data): Json<SignupRequest>,
) -> Result<(StatusCode, Json<MessageResponse>), ApiError> {
    let name = data.name.trim();
    if name.is_empty() || name.len() > 100 {
        return Err(ApiError::bad_request(
            "Name must be between 1 and 100 characters.",
        ));
    }

    let email = normalize_email(&data.email)
        .ok_or(ApiError::bad_request("Enter a valid email address."))?;

    if data.password.len() < MIN_PASSWORD_LENGTH || data.password.len() > MAX_PASSWORD_LENGTH {
        return Err(ApiError::bad_request(
            "Password must be between 8 and 128 characters.",
        ));
    }

    let password_hash = Argon2::default()
        .hash_password(data.password.as_bytes())
        .map_err(|_| ApiError::internal())?
        .to_string();

    let collection = state.database.collection::<Document>("users");
    let user = doc! {
        "name": name.to_owned(),
        "email": email,
        "password": password_hash,
    };

    match collection.insert_one(user).await {
        Ok(_) => Ok((
            StatusCode::CREATED,
            Json(MessageResponse {
                message: "Signup successful! You can now sign in.".to_string(),
            }),
        )),
        Err(error) if is_duplicate_key_error(&error) => Err(ApiError::conflict(
            "An account with that email already exists.",
        )),
        Err(_) => Err(ApiError::internal()),
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    email: String,
    exp: usize,
}

pub async fn login(
    State(state): State<AppState>,
    Json(data): Json<LoginRequest>,
) -> Result<Response, ApiError> {
    let email = normalize_email(&data.email).ok_or(ApiError::unauthorized())?;
    if data.password.is_empty() {
        return Err(ApiError::unauthorized());
    }

    let collection = state.database.collection::<Document>("users");
    let user = collection
        .find_one(doc! { "email": &email })
        .await
        .map_err(|_| ApiError::internal())?
        .ok_or(ApiError::unauthorized())?;

    let stored_hash = user.get_str("password").map_err(|_| ApiError::internal())?;
    let parsed_hash = PasswordHash::new(stored_hash).map_err(|_| ApiError::internal())?;
    Argon2::default()
        .verify_password(data.password.as_bytes(), &parsed_hash)
        .map_err(|_| ApiError::unauthorized())?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ApiError::internal())?
        .as_secs();
    let claims = Claims {
        email,
        exp: (now + 60 * 60) as usize,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    )
    .map_err(|_| ApiError::internal())?;

    let cookie = session_cookie(&token, state.cookie_secure).map_err(|_| ApiError::internal())?;
    let mut response = Json(LoginResponse {
        message: "Login successful!".to_string(),
    })
    .into_response();
    response.headers_mut().insert(SET_COOKIE, cookie);
    Ok(response)
}

pub async fn logout() -> Response {
    let mut response = StatusCode::NO_CONTENT.into_response();
    response.headers_mut().insert(
        SET_COOKIE,
        HeaderValue::from_static("session=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0"),
    );
    response
}

pub async fn users(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<UsersQuery>,
) -> Result<Json<UsersResponse>, ApiError> {
    authenticate(&headers, &state)?;

    let page = query.page.unwrap_or(1).max(1);
    let limit = query
        .limit
        .unwrap_or(DEFAULT_PAGE_SIZE)
        .clamp(1, MAX_PAGE_SIZE);
    let skip = page.saturating_sub(1).saturating_mul(limit as u64);
    let collection = state.database.collection::<Document>("users");
    let mut cursor = collection
        .find(doc! {})
        .projection(doc! { "name": 1, "email": 1, "_id": 0 })
        .sort(doc! { "email": 1 })
        .skip(skip)
        .limit(limit + 1)
        .await
        .map_err(|_| ApiError::internal())?;

    let mut users = Vec::new();
    while cursor.advance().await.map_err(|_| ApiError::internal())? {
        let user: Document = cursor
            .deserialize_current()
            .map_err(|_| ApiError::internal())?;
        if let Some(user) = public_user(&user) {
            users.push(user);
        }
    }

    let has_more = users.len() > limit as usize;
    if has_more {
        users.pop();
    }

    Ok(Json(UsersResponse { users, has_more }))
}

pub async fn profile(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<PublicUser>, ApiError> {
    let email = authenticate(&headers, &state)?;
    let collection = state.database.collection::<Document>("users");
    let user = collection
        .find_one(doc! { "email": email })
        .await
        .map_err(|_| ApiError::internal())?
        .ok_or(ApiError::unauthorized())?;

    public_user(&user).map(Json).ok_or(ApiError::internal())
}

fn authenticate(headers: &HeaderMap, state: &AppState) -> Result<String, ApiError> {
    let token = session_token(headers)?;
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims.email)
    .map_err(|_| ApiError::unauthorized())
}

fn session_token(headers: &HeaderMap) -> Result<&str, ApiError> {
    headers
        .get(COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(|cookies| {
            cookies
                .split(';')
                .map(str::trim)
                .find_map(|cookie| cookie.strip_prefix("session="))
        })
        .filter(|token| !token.is_empty())
        .ok_or(ApiError::unauthorized())
}

fn session_cookie(
    token: &str,
    secure: bool,
) -> Result<HeaderValue, axum::http::header::InvalidHeaderValue> {
    let secure_attribute = if secure { "; Secure" } else { "" };
    HeaderValue::from_str(&format!(
        "{SESSION_COOKIE}={token}; HttpOnly; SameSite=Strict; Path=/; Max-Age=3600{secure_attribute}"
    ))
}

fn normalize_email(input: &str) -> Option<String> {
    let email = input.trim().to_ascii_lowercase();
    let mut parts = email.split('@');
    let (local, domain) = (parts.next()?, parts.next()?);

    if parts.next().is_some()
        || local.is_empty()
        || domain.len() < 3
        || !domain.contains('.')
        || email.chars().any(char::is_whitespace)
    {
        return None;
    }

    Some(email)
}

fn public_user(document: &Document) -> Option<PublicUser> {
    Some(PublicUser {
        name: document.get_str("name").ok()?.to_owned(),
        email: document.get_str("email").ok()?.to_owned(),
    })
}

fn is_duplicate_key_error(error: &MongoError) -> bool {
    matches!(
        error.kind.as_ref(),
        ErrorKind::Write(WriteFailure::WriteError(write_error)) if write_error.code == 11000
    )
}
