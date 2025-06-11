mod model;
mod controller;
mod service;

use axum::{
	routing::get,
	Router,
};
use mongodb::Client;
use mongodb::options::ClientOptions;
use std::sync::Arc;
use crate::controller::track::track_routes;
use crate::service::ship_track_service::ShipTrackService;

#[tokio::main]
async fn main() {
    let client_options = ClientOptions::parse("mongodb://localhost:27017").await.unwrap();
    let client = Client::with_options(client_options).unwrap();
    let db = client.database("shipTracking");
    let collection = db.collection::<model::ship_track::ShipTrack>("trackSegments");
    let service = Arc::new(ShipTrackService::new(collection));
    let app = Router::new()
        .route("/", get(|| async { "Hello World!" }))
        .merge(track_routes())
        .with_state(service);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    let addr = listener.local_addr().unwrap();
    println!("The service is listening http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}
