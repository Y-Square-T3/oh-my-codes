use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use omc_core::error::OmcError;

pub struct AppError(pub OmcError);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self.0 {
            OmcError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            OmcError::Storage(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            OmcError::Api(msg) => (StatusCode::BAD_GATEWAY, msg.clone()),
            OmcError::Config(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            OmcError::Io(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            OmcError::Auth(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            OmcError::TokenExpired => (StatusCode::UNAUTHORIZED, "Token expired".to_string()),
            OmcError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            OmcError::PortInUse { .. } => (StatusCode::SERVICE_UNAVAILABLE, self.0.to_string()),
        };

        let body = serde_json::json!({ "error": message });
        (status, axum::Json(body)).into_response()
    }
}

impl From<OmcError> for AppError {
    fn from(err: OmcError) -> Self {
        AppError(err)
    }
}
