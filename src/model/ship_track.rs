use chrono::Utc;
use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};
use mongodb::bson::oid::{ObjectId};
use mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime;
use mongodb::bson::serde_helpers::serialize_object_id_as_hex_string;
use bson::serde_helpers::serialize_bson_datetime_as_rfc3339_string;
// 自定义序列化模块，用于将 chrono::DateTime<Utc> 序列化为 RFC 3339 字符串
mod rfc3339_date_format {
    use chrono::{DateTime, Utc};
    use serde::{Serializer, Serialize};

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // 委托给 chrono::DateTime<Utc> 的默认 Serialize 实现，
        // 当 chrono 的 "serde" 特性启用时，它会生成 RFC 3339 字符串。
        date.serialize(serializer)
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ShipTrack {
    #[serde(rename = "_id",serialize_with = "serialize_object_id_as_hex_string")]
    pub id: ObjectId,
    #[serde(rename = "startTime", deserialize_with = "chrono_datetime_as_bson_datetime::deserialize")]
    pub start_time: chrono::DateTime<Utc>,
   #[serde(rename = "lastUpdate",serialize_with = "serialize_bson_datetime_as_rfc3339_string")]
    pub last_update: DateTime,
    pub track: Track,
    #[serde(rename = "totalPoints")]
    pub total_points: u32,
}

// 新增：用于创建操作的请求体结构体
#[derive(Debug, Deserialize)] // 只需要 Deserialize，因为这是输入载荷
pub struct ShipTrackDto{
    pub track: Track, // 客户端提供 track 数据
    #[serde(rename = "totalPoints")]
    pub total_points: u32, // 客户端提供 total_points
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Track {
    #[serde(rename = "type")]
    pub track_type: String,
    pub coordinates: Vec<[f64; 2]>,
}
