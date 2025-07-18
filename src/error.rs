use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

// 定义我们的自定义错误类型
#[derive(Debug)]
pub enum AppError {
    Mongo(mongodb::error::Error),
    // 在此添加其他错误变体
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Mongo(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("数据库错误: {}", err),
            ),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

// 这使得在返回 mongodb::error::Result 的函数上可以使用 `?`
impl From<mongodb::error::Error> for AppError {
    fn from(err: mongodb::error::Error) -> Self {
        AppError::Mongo(err)
    }
}

