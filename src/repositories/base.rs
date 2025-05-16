use anyhow::Error;
use async_trait::async_trait;
use mongodb::Collection;
use mongodb::bson::Document;
use mongodb::bson::uuid::Uuid;
use serde::Serialize;
use serde::de::DeserializeOwned;

#[async_trait]
pub trait Repository<T: Send + Sync + Serialize + DeserializeOwned + 'static> {
    type UpdatePayload: Send + Sync + Serialize + DeserializeOwned + 'static;

    async fn create(&self, mut item: T) -> Result<T, Error>;

    async fn read(&self, id: Uuid) -> Result<Option<T>, Error>;

    async fn replace(&self, id: Uuid, mut item: T) -> Result<T, Error>;

    async fn update(&self, id: Uuid, update: Self::UpdatePayload) -> Result<T, Error>;

    async fn delete(&self, id: Uuid) -> Result<bool, Error>;

    async fn find(&self, filter: Document) -> Result<Vec<T>, Error>;

    fn collection(&self) -> Result<Collection<T>, Error>;
}
