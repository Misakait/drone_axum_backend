use crate::model::ship_track::ShipTrack;
use mongodb::{bson::{doc, oid::ObjectId}, options::FindOneOptions, Collection};


pub struct ShipTrackService {
    pub collection: Collection<ShipTrack>,
}

impl ShipTrackService{
    pub fn new(collection: Collection<ShipTrack>) -> Self {
        Self { collection }
    }

    pub async fn create(&self, track: ShipTrack) -> mongodb::error::Result<()> {
        self.collection.insert_one(track).await?;
        Ok(())
    }

    pub async fn get(&self, id: &str) -> mongodb::error::Result<Option<ShipTrack>> {
        let obj_id = ObjectId::parse_str(id).unwrap();
        self.collection.find_one(doc! {"_id": obj_id}).await
    }

    pub async fn update(&self, id: &str, track: ShipTrack) -> mongodb::error::Result<()> {
        let obj_id = ObjectId::parse_str(id).unwrap();
        self.collection.replace_one(doc! {"_id": obj_id}, track).await?;
        Ok(())
    }

    pub async fn delete(&self, id: &str) -> mongodb::error::Result<()> {
        let obj_id = ObjectId::parse_str(id).unwrap();
        self.collection.delete_one(doc! {"_id": obj_id}).await?;
        Ok(())
    }

    pub async fn get_latest(&self) -> mongodb::error::Result<Option<ShipTrack>> {
        let find_options = FindOneOptions::builder().sort(doc! {"last_update": -1}).build();
        self.collection.find_one(doc! {}).with_options(find_options).await
    }
}





