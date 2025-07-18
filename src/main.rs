mod model;
mod controller;
mod service;
mod error;

use axum::{
	routing::get,
	Router,
};
use mongodb::Client;
use mongodb::options::ClientOptions;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing::{info, Level};
use crate::controller::report::report_routes;
use crate::controller::track::track_routes;
use crate::service::ship_track_service::ShipTrackService;

#[tokio::main]
async fn main() {
    // 初始化日志记录器
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    let client_options = ClientOptions::parse("mongodb://localhost:27017").await.unwrap();
    let client = Client::with_options(client_options).unwrap();
    let db = client.database("shipTracking");
    // Initialize the ShipTrackService with the MongoDB collection
    let ship_track_collection = db.collection::<model::ship_track::ShipTrack>("trackSegments");
    let ship_track_service = Arc::new(ShipTrackService::new(ship_track_collection));
    // Initialize the ReportRawService with the MongoDB collection
    let report_collection = db.collection::<model::report_raw::ReportRaw>("reportRaw");
    let report_raw_service = Arc::new(service::report_raw_service::ReportRawService::new(report_collection));
    // Create the Axum application with the routes and services
    let app = Router::new()
        .route("/", get(|| async { "Hello World!" }))
        .merge(track_routes().with_state(ship_track_service))
        .merge(report_routes().with_state(report_raw_service))
        .layer(TraceLayer::new_for_http());
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    let addr = listener.local_addr().unwrap();
    info!("The service is listening http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}
