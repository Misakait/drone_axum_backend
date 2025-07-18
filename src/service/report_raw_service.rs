use bson::doc;
use futures::StreamExt;
use mongodb::Collection;
use mongodb::options::FindOneOptions;
use crate::model::report_raw::{ReportRaw, ReportRawRequestDto};

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
        let find_options = FindOneOptions::builder().sort(doc! {"lastUpdate": -1}).build();
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
}