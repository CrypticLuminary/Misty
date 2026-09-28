use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use chrono::Duration;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    access_use_cases::{
        AccessError, authenticate_session, create_account_session_for_bootstrap, issue_invitation,
        join_with_invitation,
    },
    authorization::{Capability, Role},
    space_authorization::require_capability,
    space_use_cases::create_space,
};

#[derive(Clone)]
pub struct ApiState {
    pub database: PgPool,
}

#[derive(Deserialize)]
pub struct CreateSpaceRequest {
    pub name: String,
    pub display_name: String,
}

#[derive(Serialize)]
pub struct SpaceResponse {
    pub id: Uuid,
    pub name: String,
}

#[derive(Deserialize)]
pub struct CreateInvitationRequest {
    pub role: Role,
    pub expires_in_hours: i64,
}

#[derive(Serialize)]
pub struct InvitationResponse {
    pub id: Uuid,
    pub secret: String,
    pub expires_at: String,
}

#[derive(Deserialize)]
pub struct JoinRequest {
    pub invitation_secret: String,
    pub display_name: String,
}

#[derive(Serialize)]
pub struct JoinResponse {
    pub subject_id: Uuid,
    pub session_id: Uuid,
    pub session_secret: String,
    pub expires_at: String,
}

#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
}

pub fn router(state: ApiState) -> Router {
    let router = Router::new()
        .route("/v1/spaces", post(create_space_handler))
        .route("/v1/spaces/{space_id}", get(get_space_handler))
        .route("/v1/spaces/{space_id}/invitations", post(create_invitation_handler))
        .route("/v1/join", post(join_handler));

    let router = if std::env::var("MISTY_ENABLE_DEV_BOOTSTRAP").as_deref() == Ok("true") {
        router.route("/v1/dev/bootstrap-session", post(bootstrap_session_handler))
    } else {
        router
    };

    router.with_state(state)
}

async fn bootstrap_session_handler(
    State(state): State<ApiState>,
) -> Result<(StatusCode, Json<JoinResponse>), (StatusCode, Json<ErrorBody>)> {
    if std::env::var("MISTY_ENABLE_DEV_BOOTSTRAP").as_deref() != Ok("true") {
        return Err(not_found());
    }
    let session = create_account_session_for_bootstrap(&state.database)
        .await
        .map_err(map_access_error)?;
    Ok((
        StatusCode::CREATED,
        Json(JoinResponse {
            subject_id: session.subject_id,
            session_id: session.session_id,
            session_secret: session.secret,
            expires_at: session.expires_at.to_rfc3339(),
        }),
    ))
}

async fn create_space_handler(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(input): Json<CreateSpaceRequest>,
) -> Result<(StatusCode, Json<SpaceResponse>), (StatusCode, Json<ErrorBody>)> {
    let subject_id = subject_from_headers(&state.database, &headers).await?;
    let id = create_space(
        &state.database,
        subject_id,
        &input.display_name,
        &input.name,
        Uuid::new_v4(),
    )
    .await
    .map_err(|_| bad_request("INVALID_SPACE"))?;

    Ok((
        StatusCode::CREATED,
        Json(SpaceResponse {
            id,
            name: input.name.trim().to_owned(),
        }),
    ))
}

async fn get_space_handler(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(space_id): Path<Uuid>,
) -> Result<Json<SpaceResponse>, (StatusCode, Json<ErrorBody>)> {
    let subject_id = subject_from_headers(&state.database, &headers).await?;
    require_capability(&state.database, subject_id, space_id, Capability::View)
        .await
        .map_err(|_| unauthorized())?;

    let name: String = sqlx::query_scalar("SELECT name FROM spaces WHERE id = $1 AND status <> 'deleted'")
        .bind(space_id)
        .fetch_optional(&state.database)
        .await
        .map_err(|_| internal())?
        .ok_or_else(not_found)?;

    Ok(Json(SpaceResponse { id: space_id, name }))
}

async fn create_invitation_handler(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(space_id): Path<Uuid>,
    Json(input): Json<CreateInvitationRequest>,
) -> Result<(StatusCode, Json<InvitationResponse>), (StatusCode, Json<ErrorBody>)> {
    let subject_id = subject_from_headers(&state.database, &headers).await?;
    let invitation = issue_invitation(
        &state.database,
        subject_id,
        space_id,
        input.role,
        Duration::hours(input.expires_in_hours),
        Uuid::new_v4(),
    )
    .await
    .map_err(map_access_error)?;

    Ok((
        StatusCode::CREATED,
        Json(InvitationResponse {
            id: invitation.invitation_id,
            secret: invitation.secret,
            expires_at: invitation.expires_at.to_rfc3339(),
        }),
    ))
}

async fn join_handler(
    State(state): State<ApiState>,
    Json(input): Json<JoinRequest>,
) -> Result<(StatusCode, Json<JoinResponse>), (StatusCode, Json<ErrorBody>)> {
    let session = join_with_invitation(
        &state.database,
        &input.invitation_secret,
        &input.display_name,
        Uuid::new_v4(),
    )
    .await
    .map_err(map_access_error)?;

    Ok((
        StatusCode::CREATED,
        Json(JoinResponse {
            subject_id: session.subject_id,
            session_id: session.session_id,
            session_secret: session.secret,
            expires_at: session.expires_at.to_rfc3339(),
        }),
    ))
}

async fn subject_from_headers(
    pool: &PgPool,
    headers: &HeaderMap,
) -> Result<Uuid, (StatusCode, Json<ErrorBody>)> {
    let credential = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or_else(unauthorized)?;

    authenticate_session(pool, credential)
        .await
        .map_err(|_| unauthorized())
}

fn map_access_error(error: AccessError) -> (StatusCode, Json<ErrorBody>) {
    match error {
        AccessError::InvalidInput => bad_request("INVALID_INPUT"),
        AccessError::Unauthorized => unauthorized(),
        AccessError::Forbidden => (StatusCode::FORBIDDEN, Json(ErrorBody { code: "FORBIDDEN" })),
        AccessError::InviteUnavailable => (
            StatusCode::NOT_FOUND,
            Json(ErrorBody {
                code: "INVITE_UNAVAILABLE",
            }),
        ),
        AccessError::Database => internal(),
    }
}

fn bad_request(code: &'static str) -> (StatusCode, Json<ErrorBody>) {
    (StatusCode::BAD_REQUEST, Json(ErrorBody { code }))
}

fn unauthorized() -> (StatusCode, Json<ErrorBody>) {
    (
        StatusCode::UNAUTHORIZED,
        Json(ErrorBody {
            code: "UNAUTHORIZED",
        }),
    )
}

fn not_found() -> (StatusCode, Json<ErrorBody>) {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorBody { code: "NOT_FOUND" }),
    )
}

fn internal() -> (StatusCode, Json<ErrorBody>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorBody {
            code: "INTERNAL_ERROR",
        }),
    )
}
