use crate::error::AppError;
use crate::model::report_raw::{ReportRawRequestDto, ReportRawResponseDto};
use crate::service::report_raw_service::ReportRawService;
use axum::extract::{State, Multipart, DefaultBodyLimit};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use std::sync::Arc;


pub fn report_routes() -> Router<Arc<ReportRawService>> {
    Router::new()
        .route("/report_raw", post(create_report_raw).get(get_report_raw_all))
        .route("/report_latest", get(get_latest_report_raw))
        .route("/report_with_image", post(create_report_with_image))
        .route("/report_raw/delete_all", delete(delete_all_report_raw))
        .layer(DefaultBodyLimit::max(1024*1024*10*5)) // 50MB limit
}
async fn get_report_raw_all(State(service): State<Arc<ReportRawService>>) -> Result<Json<Vec<ReportRawResponseDto>>, AppError> {
    let reports = service.get_all().await?;
    let response: Vec<ReportRawResponseDto> = reports.into_iter().map(ReportRawResponseDto::from).collect();
    Ok(Json(response))
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

async fn create_report_with_image(
    State(service): State<Arc<ReportRawService>>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut image_files = Vec::new();
    let mut report_data: Option<ReportRawRequestDto> = None;

    // 解析 multipart 表单数据
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::BadRequest(format!("Failed to read multipart field: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();

        if name == "image" {
            // 收集图片文件信息
            let file_name = field.file_name().unwrap_or("unknown").to_string();
            let content_type = field.content_type().unwrap_or("").to_string();
            
            let data = field.bytes().await.map_err(|e| {
                AppError::BadRequest(format!("Failed to read file data: {}", e))
            })?;

            image_files.push((file_name, content_type, data));

        } else if name == "report_data" {
            // 处理报告数据
            let data = field.text().await.map_err(|e| {
                AppError::BadRequest(format!("Failed to read report data: {}", e))
            })?;

            report_data = Some(serde_json::from_str::<ReportRawRequestDto>(&data).map_err(|e| {
                AppError::BadRequest(format!("Invalid report data format: {}", e))
            })?);
        }
    }

    // 验证报告数据是否存在
    let report = report_data.ok_or_else(|| {
        AppError::BadRequest("Missing report data".to_string())
    })?;

    // 委托给 service 层处理业务逻辑
    let result = service.create_report_with_images(report, image_files).await?;
    
    Ok(Json(result))
}

async fn delete_all_report_raw(State(service): State<Arc<ReportRawService>>) -> Result<Json<serde_json::Value>, AppError> {
    let deleted_count = service.delete_all().await?;
    Ok(Json(serde_json::json!({
        "status": "success",
        "deleted_count": deleted_count
    })))
}
