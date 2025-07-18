use bson::DateTime;
use bson::oid::ObjectId;
use mongodb::bson::serde_helpers::serialize_object_id_as_hex_string;
use bson::serde_helpers::serialize_bson_datetime_as_rfc3339_string;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportRaw {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime,
    #[serde(rename = "photoPath")]
    pub photo_path: String,
    
    pub detail: String,
    
    pub title: String
}
impl From<ReportRawRequestDto> for ReportRaw {
    fn from(dto: ReportRawRequestDto) -> Self {
        ReportRaw {
            id: ObjectId::new(),
            created_at: DateTime::now(),
            photo_path: dto.photo_path,
            detail: dto.detail,
            title: dto.title,
        }
    }
}
#[derive(Debug, Deserialize,Serialize)]
pub struct ReportRawRequestDto {
    #[serde(rename = "photoPath")]
    pub photo_path: String,
    pub detail: String,
    pub title: String,
}
#[derive(Debug, Deserialize,Serialize)]
pub struct ReportRawResponseDto {
    #[serde(rename = "_id", serialize_with = "serialize_object_id_as_hex_string")]
    pub id: ObjectId,
    #[serde(rename = "createdAt", serialize_with = "serialize_bson_datetime_as_rfc3339_string")]
    pub created_at: DateTime,
    #[serde(rename = "photoPath")]
    pub photo_path: String,
    
    pub detail: String,
    
    pub title: String
}
impl From<ReportRaw> for ReportRawResponseDto {
    fn from(report_raw: ReportRaw) -> Self {
        ReportRawResponseDto {
            id: report_raw.id,
            created_at: report_raw.created_at,
            photo_path: report_raw.photo_path,
            detail: report_raw.detail,
            title: report_raw.title,
        }
    }
}