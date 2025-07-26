use bson::doc;

use mongodb::bson::oid::ObjectId;
use mongodb::Collection;
use tracing::log::error;
use crate::error::AppError;
use crate::model::flight::Flight;

pub struct FlightService{
    pub collection: Collection<Flight>,
}
impl FlightService {
    pub fn new(collection: Collection<Flight>) -> Self {
        Self { collection }
    }

    pub async fn create(&self, flight: Flight) -> Result<(), AppError> {
        self.collection.insert_one(flight).await.map_err(|e| {
            AppError::InternalServerError(format!("Failed to save report: {}", e))
        })?;
        Ok(())
    }

    pub async fn get(&self, id: &str) -> mongodb::error::Result<Option<Flight>> {
        let obj_id = ObjectId::parse_str(id).map_err(|e| {
            error!("{:?}", e);
            mongodb::error::Error::custom(e)
        })?;
        self.collection.find_one(doc! {"_id": obj_id}).await
    }

    pub async fn update(&self, id: &str, flight: Flight) -> mongodb::error::Result<()> {
        let obj_id = ObjectId::parse_str(id).map_err(|e| {
            error!("{:?}", e);
            mongodb::error::Error::custom(e)
        })?;
        self.collection.replace_one(doc! {"_id": obj_id}, flight).await?;
        Ok(())
    }
}