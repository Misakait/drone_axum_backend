use chrono::Utc;
use axum::{extract::{State, Path}, Json, Router, routing::{get, post, put, delete}};
use crate::model::ship_track::{ShipTrack, UpdateShipTrackPayload};
use crate::service::ship_track_service::ShipTrackService;
use std::sync::Arc;
use crate::model::ship_track::ShipTrackRequestDto;
use crate::model::ship_track::ShipTrackResponseDto;
use bson::oid::ObjectId;
pub fn track_routes() -> Router<Arc<ShipTrackService>> {
    Router::new()
        .route("/track", post(create_track))
        .route("/track/{id}", get(get_track))
        .route("/track/{id}", put(update_track))
        .route("/track/{id}", delete(delete_track))
        .route("/track_latest", get(get_latest_track))
        .route("/append_track/{id}", put(append_track))
}

async fn create_track(State(service): State<Arc<ShipTrackService>>, Json(track_dto): Json<ShipTrackRequestDto>) -> Json<String> {
    let new_id = ObjectId::new(); // 服务器生成 _id
    let current_time = Utc::now(); // 服务器生成时间戳

    // 从 payload 和服务器生成的值构建 ShipTrack 实例
    let track = ShipTrack {
        id: new_id,
        start_time: current_time.into(),
        last_update: current_time.into(),
        coordinates: track_dto.coordinates,
        total_points: track_dto.total_points,
    };
    service.create(track).await.unwrap();
    Json(new_id.to_hex())
}

async fn get_track(State(service): State<Arc<ShipTrackService>>, Path(id): Path<String>) -> Json<Option<ShipTrackResponseDto>> {
    let res = service.get(&id).await.unwrap();
    Json(res.map(ShipTrackResponseDto::from))
}

async fn update_track(State(service): State<Arc<ShipTrackService>>, Path(id): Path<String>, Json(track): Json<ShipTrack>) -> Json<&'static str> {
    service.update(&id, track).await.unwrap();
    Json("ok")
}
async fn delete_track (State(service): State<Arc<ShipTrackService>>, Path(id): Path<String>) -> Json<&'static str> {
    service.delete(&id).await.unwrap();
    Json("ok")
}
async fn get_latest_track (State(service): State<Arc<ShipTrackService>>) -> Json<Option<ShipTrackResponseDto>> {
    let res = service.get_latest().await.unwrap();
    Json(res.map(ShipTrackResponseDto::from))
}

async fn append_track(
    State(service): State<Arc<ShipTrackService>>,
    Path(id): Path<String>, // 从路径获取 ID
    Json(payload): Json<UpdateShipTrackPayload> // 使用新的 Payload
) -> Json<Option<ShipTrackResponseDto>> { // 返回更新后的轨迹或 None
    match service.append_coordinates_and_update(&id, payload.coordinates_to_add).await {
        Ok(Some(updated_track_model)) => Json(Some(ShipTrackResponseDto::from(updated_track_model))),
        Ok(None) => Json(None), // 文档未找到或未修改 (但 ReturnDocument::After 应该会返回)
        Err(_e) => {
            // 在实际应用中应记录错误 _e
            // 并返回更合适的 HTTP 错误状态码
            eprintln!("Error updating track: {:?}", _e); // 简单打印错误
            Json(None) // 简化错误处理
        }
    }
}