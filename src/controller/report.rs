use crate::error::AppError;
use crate::model::report_raw::{ReportRaw, ReportRawRequestDto, ReportRawResponseDto};
use crate::service::report_raw_service::ReportRawService;
use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use std::sync::Arc;


pub fn report_routes() -> Router<Arc<ReportRawService>> {
    Router::new()
        .route("/report_raw", post(create_report_raw))
        .route("/report_latest", get(get_latest_report_raw))
}
async fn get_latest_report_raw(State(service): State<Arc<ReportRawService>>) -> Result<Json<Option<ReportRawResponseDto>>, AppError> {
    let res = service.get_latest().await?;
    match res {
        Some(report) => Ok(Json(Some(ReportRawResponseDto::from(report)))),
        None => Ok(Json(None)),
    }
}
async fn create_report_raw(
    State(service): State<Arc<ReportRawService>>,
    Json(report): Json<ReportRawRequestDto>
) -> Result<Json<&'static str>, AppError> {
    service.insert_one(report).await?;
    Ok(Json("ok"))
}