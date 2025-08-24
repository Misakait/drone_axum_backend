use bson::{doc, DateTime};
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
use bson::oid::ObjectId;
use crate::service::ai_service::{AIPaylod, AiService, Message};

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
        let find_options = FindOneOptions::builder().sort(doc! {"createdAt": -1}).build();
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
    pub async fn delete_by_id(&self, id: &str) -> Result<(),AppError> {
        let obj_id = ObjectId::parse_str(id).map_err(|e| {
            AppError::BadRequest(format!("Invalid ObjectId: {}", e))
        })?;
        self.collection.delete_one(doc! {"_id": obj_id}).await.map_err(|e| {
            AppError::InternalServerError(format!("Failed to delete report: {}", e))
        })?;
        Ok(())
    }

    pub async fn create_report_with_images(
        &self,
        report_data: ReportRawRequestDto,
        image_files: Vec<(String, String, bytes::Bytes)>, // (filename, content_type, data)
    ) -> Result<(serde_json::Value,ObjectId), AppError> {
        let mut image_paths = Vec::new();
        let mut relative_paths = Vec::new(); // 用于存储相对路径
        let base_upload_dir = "/var/uploads/images";
        
        // 生成日期文件夹名称 (YYYYMMDD)
        let date_folder = chrono::Utc::now().format("%Y%m%d").to_string();
        let upload_dir = format!("{}/{}", base_upload_dir, date_folder);

        // 确保上传目录存在
        if !Path::new(&upload_dir).exists() {
            fs::create_dir_all(&upload_dir).await.map_err(|e| {
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
            let unique_filename = format!("{}.{}", Uuid::new_v4(), extension);
            let file_path = format!("{}/{}", upload_dir, unique_filename);
            let relative_path = format!("/{}/{}", date_folder, unique_filename);

            // 保存文件
            let mut file = fs::File::create(&file_path).await.map_err(|e| {
                AppError::InternalServerError(format!("Failed to create file: {}", e))
            })?;

            file.write_all(&data).await.map_err(|e| {
                AppError::InternalServerError(format!("Failed to write file: {}", e))
            })?;

            info!("图片文件已保存: {}", file_path);
            image_paths.push(file_path);
            relative_paths.push(relative_path);
        }

        // let ai_service = AiService::new("https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions".into(), "sk-7d6d33a351fd4d2b9b14dda64062676f".into());
        // let mut messages = vec![];
        // messages.push(crate::service::ai_service::Message{ role: "system".to_string(), content: "你是一个报告智能报告生成体,用户会发送风机叶片检测之后的三个参数,分别为锈蚀情况,覆盖情况,损坏情况,这三个参数均在0到1之间代表百分数,你需要为其生成一份简短的报告,切记不要使用markdown格式".to_string() });
        // messages.push(crate::service::ai_service::Message{ role: "user".to_string(), content: format!("请为本次风机巡检生成报告，其中锈蚀严重度为{}，锈蚀覆盖度为{}, 损坏程度为{}", report_data.rust, report_data.covering, report_data.damage) });
        // let ai_payload = AIPaylod{
        //     messages,
        //     model: "qwen-plus".to_string(),
        // };
        // let result = ai_service.analyze_report(ai_payload).await?;
        // info!("AI 分析结果: {}", result);
        let report_id = ObjectId::new();

        // 保存报告到数据库
        let report_raw = ReportRaw{
            id: report_id,
            created_at: DateTime::now(),
            photo_path: relative_paths.join(", "), // 使用相对路径
            detail: report_data.detail,
            title: report_data.title,
            damage: report_data.damage,
            rust: report_data.rust,
            covering: report_data.covering,
            ai_report: None,
        };
        self.collection.insert_one(report_raw).await.map_err(|e| {
            AppError::InternalServerError(format!("Failed to save report: {}", e))
        })?;
        
        // 返回成功响应
        Ok((serde_json::json!({
            "status": "success",
            "message": "Report created successfully",
            "uploaded_images": image_paths,
            "relative_paths": relative_paths,
            "image_count": image_paths.len()
        }),report_id))
    }
    // 后台AI分析任务
    pub async fn generate_ai_report_background(
        &self,
        report_id: ObjectId,
        rust: f64,
        covering: f64,
        damage: f64,
    ) {
        let ai_service = AiService::new(
            "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions".into(),
            "sk-7d6d33a351fd4d2b9b14dda64062676f".into()
        );

        let mut messages = vec![];
        messages.push(Message {
            role: "system".to_string(),
            content: "你是一个报告智能报告生成体,用户会发送风机叶片检测之后的三个参数,分别为锈蚀情况,覆盖情况,损坏情况,这三个参数均在0到1之间代表百分数,你需要为其生成一份简短的报告以及维修建议,切记不要使用markdown格式".to_string()
        });
        messages.push(Message {
            role: "user".to_string(),
            content: format!(
                "请为本次风机巡检生成报告，其中锈蚀严重度为{}，锈蚀覆盖度为{}, 损坏程度为{}",
                rust,
                covering,
                damage
            )
        });

        let ai_payload = AIPaylod {
            messages,
            model: "qwen-plus".to_string(),
        };

        info!("开始后台AI分析，报告ID: {}", report_id.to_hex());
        match ai_service.analyze_report(ai_payload).await {
            Ok(result) => {
                info!("AI分析完成，报告ID: {}", report_id.to_hex());
                if let Err(e) = self.update_ai_report(report_id, result).await {
                    tracing::error!("更新AI报告失败: {:?}", e);
                }
            }
            Err(e) => {
                tracing::error!("AI分析失败: {:?}", e);
            }
        }
    }
    pub async fn update_ai_report(&self, report_id: ObjectId, ai_report: String) -> Result<(), AppError> {
        self.collection
            .update_one(
                doc! {"_id": report_id},
                doc! {
                    "$set": {
                        "aiReport": ai_report,
                    }
                }
            )
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to update AI report: {}", e)))?;

        info!("AI报告已更新到数据库，报告ID: {}", report_id.to_hex());
        Ok(())
    }
    pub async fn delete_all(&self) -> Result<u64, AppError> {
        let result = self.collection.delete_many(doc!{}).await.map_err(|e| {
            AppError::InternalServerError(format!("Failed to delete all reports: {}", e))
        })?;
        Ok(result.deleted_count)
    }
}