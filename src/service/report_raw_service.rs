use bson::doc;
use futures::StreamExt;
use mongodb::Collection;
use mongodb::options::FindOneOptions;
use crate::model::report_raw::{ReportRaw, ReportRawRequestDto, ReportRawResponseDto};
use crate::error::AppError;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;
use std::path::Path;
use tracing::info;
use axum::extract::multipart::Field;

pub struct ReportRawService{
    pub collection: Collection<ReportRaw>,
}

impl ReportRawService {
    pub fn new(collection: Collection<ReportRaw>) -> Self {
        ReportRawService { collection }
    }

    pub async fn insert_one(&self, report_raw_request: ReportRawRequestDto) -> mongodb::error::Result<()> {
        let report_raw = ReportRaw::from(report_raw_request);
        self.collection.insert_one(report_raw).await?;
        Ok(())
    }

    pub async fn get_latest(&self) -> mongodb::error::Result<Option<ReportRaw>> {
        let find_options = FindOneOptions::builder().sort(doc! {"lastUpdate": -1}).build();
        self.collection.find_one(doc! {}).with_options(find_options).await
    }
    pub async fn get_by_id(&self, id: &str) -> mongodb::error::Result<Option<ReportRaw>> {
        let obj_id = bson::oid::ObjectId::parse_str(id).map_err(|e| mongodb::error::Error::custom(format!("Invalid ObjectId: {}", e)))?;
        self.collection.find_one(doc! {"_id": obj_id}).await
    }
    pub async fn get_all(&self) -> mongodb::error::Result<Vec<ReportRaw>> {
        let mut cursor = self.collection.find(doc! {}).await?;
        let mut results = Vec::new();
        while let Some(doc) = cursor.next().await {
            match doc {
                Ok(report_raw) => results.push(report_raw),
                Err(e) => return Err(e.into()),
            }
        }
        Ok(results)
    }

    pub async fn create_report_with_images(
        &self,
        report_data: ReportRawRequestDto,
        image_files: Vec<(String, String, bytes::Bytes)>, // (filename, content_type, data)
    ) -> Result<serde_json::Value, AppError> {
        let mut image_paths = Vec::new();
        let upload_dir = "/var/uploads/images";

        // 确保上传目录存在
        if !Path::new(upload_dir).exists() {
            fs::create_dir_all(upload_dir).await.map_err(|e| {
                AppError::InternalServerError(format!("Failed to create upload directory: {}", e))
            })?;
        }

        // 处理图片文件上传
        for (file_name, content_type, data) in image_files {
            // 验证是否为图片文件
            if !content_type.starts_with("image/") {
                return Err(AppError::BadRequest("Only image files are allowed".to_string()));
            }

            // 生成唯一文件名
            let extension = file_name.split('.').last().unwrap_or("jpg");
            let unique_filename = format!("{}_{}.{}", Uuid::new_v4(), chrono::Utc::now().timestamp(), extension);
            let file_path = format!("{}/{}", upload_dir, unique_filename);

            // 保存文件
            let mut file = fs::File::create(&file_path).await.map_err(|e| {
                AppError::InternalServerError(format!("Failed to create file: {}", e))
            })?;

            file.write_all(&data).await.map_err(|e| {
                AppError::InternalServerError(format!("Failed to write file: {}", e))
            })?;

            info!("图片文件已保存: {}", file_path);
            image_paths.push(file_path);
        }

        // 保存报告到数据库
        self.insert_one(report_data).await.map_err(|e| {
            AppError::InternalServerError(format!("Failed to save report: {}", e))
        })?;

        // 返回成功响应
        Ok(serde_json::json!({
            "status": "success",
            "message": "Report created successfully",
            "uploaded_images": image_paths,
            "image_count": image_paths.len()
        }))
    }
}