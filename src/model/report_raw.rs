use bson::DateTime;
use bson::oid::ObjectId;
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