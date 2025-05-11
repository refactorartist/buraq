use mongodb::Collection;
use mongodb::bson::{Document, doc};
use mongodb::bson::oid::ObjectId;
use anyhow::Error;
use mongodb::results::{InsertOneResult, UpdateResult, DeleteResult};
use serde::Serialize;
use serde::de::DeserializeOwned;
use futures::TryStreamExt;
use async_trait::async_trait;

#[async_trait]
pub trait Repository<T: Send + Sync + Serialize + DeserializeOwned + 'static> {
    async fn create(&self, item: T) -> Result<InsertOneResult, Error> {
        let collection = self.collection()?;
        let result = collection.insert_one(item).await?;
        Ok(result)
    }

    async fn read(&self, id: ObjectId) -> Result<Option<T>, Error> {
        let collection = self.collection()?;
        let filter = doc! { "_id": id };
        let result = collection.find_one(filter).await?;
        Ok(result)
    }

    async fn update(&self, id: ObjectId, item: T) -> Result<UpdateResult, Error> {
        let collection = self.collection()?;
        let filter = doc! { "_id": id };
        
        // Convert item to document
        let mut doc = mongodb::bson::to_document(&item)?;
        
        // Preserve the _id field
        doc.insert("_id", id);
        
        // Convert back to T
        let item = mongodb::bson::from_document::<T>(doc)?;
        
        let result = collection.replace_one(filter, item).await?;
        Ok(result)
    }

    async fn delete(&self, id: ObjectId) -> Result<DeleteResult, Error> {
        let collection = self.collection()?;
        let filter = doc! { "_id": id };
        let result = collection.delete_one(filter).await?;
        Ok(result)
    }

    async fn find(&self, filter: Document) -> Result<Vec<T>, Error> {
        let collection = self.collection()?;

        let mut cursor = collection.find(filter).await?; 
        let mut results = Vec::new();

        while let Some(item) = cursor.try_next().await? {
            results.push(item);
        }

        Ok(results) 
    }

    fn collection(&self) -> Result<Collection<T>, Error>;
}

