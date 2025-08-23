use std::sync::Arc;
use axum::extract::State;
use axum::{Json, Router};
use axum::routing::{delete, get, post, put};
use mongodb::bson::oid::ObjectId;
use tracing::log::error;
use crate::error::AppError;
use crate::service::flight_service::FlightService;
use crate::model::flight::{Flight, FlightDto};
use crate::service::ship_track_service::ShipTrackService;

pub fn flight_routes() -> Router<Arc<FlightService>> {
    Router::new()
        .route("/flight", post(create_empty_flight))
}
async fn create_empty_flight(
    State(service): State<Arc<FlightService>>,
    Json(track_id_raw): Json<String>,
) ->Result<Json<String>,AppError>{
    let new_id = ObjectId::new(); // 服务器生成 _id
    let track_id = ObjectId::parse_str(&track_id_raw).map_err(|e| {
        error!("{:?}",e);
       AppError::BadRequest("Invalid track ID".to_string())})?;
    // 从 payload 和服务器生成的值构建 Flight 实例
    let flight = Flight {
        id: new_id,
        track_id,
        battery_capacity: vec![],
        estimated_remaining_usage_time: vec![],
        cabin_temperature: vec![],
        aircraft_altitude: vec![],
        distance_to_fan: vec![],
        air_pressure: vec![],
    };
    service.create(flight).await?;
    Ok(Json(new_id.to_hex()))
}