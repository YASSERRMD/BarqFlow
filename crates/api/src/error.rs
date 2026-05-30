use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

/// Structured API error returned by all handlers.
///
/// Serialises to `{"code": "...", "message": "..."}` so clients can react to
/// machine-readable codes without parsing human-readable strings.
#[derive(Debug, Serialize)]
pub struct ApiError {
    pub code: &'static str,
    pub message: String,
}

impl ApiError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> (StatusCode, Self) {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Self::new("INTERNAL_ERROR", message),
        )
    }

    pub fn not_found(message: impl Into<String>) -> (StatusCode, Self) {
        (StatusCode::NOT_FOUND, Self::new("NOT_FOUND", message))
    }

    pub fn bad_request(message: impl Into<String>) -> (StatusCode, Self) {
        (StatusCode::BAD_REQUEST, Self::new("BAD_REQUEST", message))
    }

    pub fn unauthorized(message: impl Into<String>) -> (StatusCode, Self) {
        (StatusCode::UNAUTHORIZED, Self::new("UNAUTHORIZED", message))
    }

    pub fn forbidden(message: impl Into<String>) -> (StatusCode, Self) {
        (StatusCode::FORBIDDEN, Self::new("FORBIDDEN", message))
    }

    pub fn service_unavailable(message: impl Into<String>) -> (StatusCode, Self) {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Self::new("SERVICE_UNAVAILABLE", message),
        )
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

/// Convenience type alias used by handlers.
pub type ApiResult<T> = Result<T, (StatusCode, ApiError)>;

/// Convert a plain `(StatusCode, String)` tuple — the old error shape used
/// throughout the codebase — into the new typed `(StatusCode, ApiError)`.
pub fn from_string_error(
    code: &'static str,
) -> impl Fn((StatusCode, String)) -> (StatusCode, ApiError) {
    move |(status, message)| (status, ApiError::new(code, message))
}

// `(StatusCode, ApiError)` is already usable as an Axum response via axum's
// blanket `IntoResponse for (StatusCode, T) where T: IntoResponse` impl, since
// `ApiError` implements `IntoResponse` above.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_error_serialises_with_code_and_message() {
        let error = ApiError::new("NOT_FOUND", "workflow not found");
        let json = serde_json::to_value(&error).unwrap();

        assert_eq!(json["code"], "NOT_FOUND");
        assert_eq!(json["message"], "workflow not found");
    }

    #[test]
    fn helpers_return_correct_status_codes() {
        assert_eq!(
            ApiError::internal("oops").0,
            StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(ApiError::not_found("missing").0, StatusCode::NOT_FOUND);
        assert_eq!(ApiError::bad_request("bad").0, StatusCode::BAD_REQUEST);
        assert_eq!(ApiError::unauthorized("denied").0, StatusCode::UNAUTHORIZED);
        assert_eq!(ApiError::forbidden("no").0, StatusCode::FORBIDDEN);
        assert_eq!(
            ApiError::service_unavailable("busy").0,
            StatusCode::SERVICE_UNAVAILABLE
        );
    }
}
