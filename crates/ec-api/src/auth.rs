#![forbid(unsafe_code)]

//! مصادقة أساسية عبر مفتاح ثابت — ADR-024 (F2).

use crate::state::AppState;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;

/// اسم الـheader المطلوب.
pub const API_KEY_HEADER: &str = "x-api-key";

/// middleware يرفض أي طلب بلا مفتاح مطابق.
pub async fn require_api_key(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let provided = request
        .headers()
        .get(API_KEY_HEADER)
        .and_then(|v| v.to_str().ok());

    match provided {
        Some(key) if key == state.api_key.as_ref() => Ok(next.run(request).await),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_name_is_lowercase_and_stable() {
        assert_eq!(API_KEY_HEADER, "x-api-key");
    }
}
