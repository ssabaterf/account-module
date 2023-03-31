use async_std::stream::StreamExt;
use async_trait::async_trait;
use mongodb::{
    bson::{self, doc},
    options::ClientOptions,
    Client, Collection, Database,
};
use serde::{de::DeserializeOwned, Serialize};
use std::borrow::Borrow;
pub struct Data {
    pub client: Client,
    pub db: Database,
    uri: String,
}

impl Data {
    pub async fn new(uri: &str, app: &str, database: &str) -> Result<Data, String> {
        let mut client_options = match ClientOptions::parse(uri).await {
            Ok(client_options) => client_options,
            Err(e) => return Err(format!("Error parsing uri: {}", e)),
        };
        client_options.app_name = Some(app.to_string());
        let client = match Client::with_options(client_options) {
            Ok(client) => client,
            Err(e) => return Err(format!("Error creating client: {}", e)),
        };
        let db = client.database(database);
        Ok(Data {
            client: client,
            db: db,
            uri: uri.to_string(),
        })
    }
    pub fn get_repo<T: Send + Sync + Clone + Serialize + DeserializeOwned + Unpin + 'static>
    (&self, collection: &str, key_field:String)-> Result<Repository<T>, String> {
        let collection = self.db.collection(collection);
        Ok(Repository {
            key_field: key_field,
            collection: collection,
            uri: self.uri.clone(),
        })
    }
}
pub struct Repository<T> 
where T: Send + Sync + Clone + Serialize + DeserializeOwned + Unpin + 'static{
    key_field: String,
    collection: Collection<T>,
    uri: String,
}
#[async_trait]
pub trait Crud<T>: Send + Sync {
    async fn create_many(&self, new_entities: Vec<T>) -> Result<Vec<String>, String>;
    async fn create(&self, new_entity: T) -> Result<String, String>;
    async fn get_all(&self) -> Result<Vec<T>, String>;
    async fn get_by_id(&self, id: &str) -> Result<T, String>;
    async fn update_by_id(&self, id: &str, edit_entity: T) -> Result<T, String>;
    async fn delete_by_id(&self, id: &str) -> Result<bool, String>;
    async fn get_by_fields(&self, field: Vec<String>, value: Vec<String>) -> Result<Vec<T>, String>;
}
#[async_trait]
impl<T> Crud<T> for Repository<T>
where
    T: Send + Sync + Clone + Serialize + DeserializeOwned + Unpin + 'static,
{
    async fn create_many(&self, entities: Vec<T>) -> Result<Vec<String>, String> {
        let result = self.collection.insert_many(entities, None).await.ok().expect("Error creating entities");
        let ids = result.inserted_ids.iter().map(|id| id.1.to_string()).collect();
        Ok(ids)
    }
    async fn create(&self, new_entity: T) -> Result<String, String> {
        let entity = self
            .collection
            .insert_one(new_entity.borrow(), None)
            .await
            .ok()
            .expect("Error creating entity")
            .inserted_id
            .to_string();
        Ok(entity)
    }

    async fn get_by_id(&self, id: &str) -> Result<T, String> {
        let filter = doc! {&self.key_field: id};
        let entity_detail = self
            .collection
            .find_one(filter, None)
            .await
            .ok()
            .expect("Error getting entity's detail");
        Ok(entity_detail.expect("Entity not found"))
    }

    async fn update_by_id(&self, id: &str, edit_entity: T) -> Result<T, String> {
        let filter = doc! {&self.key_field: id};
        let edit_doc = bson::to_document(&edit_entity).map_err(|e| format!("Error serializing entity: {}", e))?;

        let updated_doc = match self
            .collection
            .update_one(filter, doc! {"$set": edit_doc}, None)
            .await{
                Ok(updated_doc) => updated_doc,
                Err(e) => return Err(format!("Error updating entity: {}", e))
            };
        if updated_doc.matched_count > 0 {
            Ok(self.get_by_id(id).await.unwrap())
        } else {
            Err("Error updating entity".to_uppercase())
        }
    }

    async fn delete_by_id(&self, id: &str) -> Result<bool, String> {
        let filter = doc! {&self.key_field: id};

        let entity_detail = self
            .collection
            .delete_one(filter, None)
            .await
            .ok()
            .expect("Error deleting entity");
        if entity_detail.deleted_count > 0 {
            Ok(true)
        } else {
            Err("Error deleting entity".to_uppercase())
        }
    }

    async fn get_all(&self) -> Result<Vec<T>, String> {
        let mut cursors = match self.collection.find(None, None).await {
            Ok(cursors) => cursors,
            Err(e) => return Err(format!("Error getting entities: {}", e)),
        };
        let mut entities: Vec<T> = Vec::new();

        while let Some(entity) = cursors.next().await {
            let entity = match entity {
                Ok(entity) => entity,
                Err(e) => return Err(format!("Error getting entities: {}", e)),
            };
            entities.push(entity);
        }
        Ok(entities)
    }

    async fn get_by_fields(&self, field: Vec<String>, value: Vec<String>) -> Result<Vec<T>, String> {
        let mut filter_doc = doc! {};
        for (i, field_name) in field.iter().enumerate() {
            let field_value = &value[i];
            filter_doc.extend(doc! {field_name: field_value});
        }

        let mut cursors = match self.collection.find(filter_doc, None).await {
            Ok(cursors) => cursors,
            Err(e) => return Err(format!("Error getting entities by fields: {}", e)),
        };
        let mut entities: Vec<T> = Vec::new();

        while let Some(entity) = cursors.next().await {
            let entity = match entity {
                Ok(entity) => entity,
                Err(e) => return Err(format!("Error getting entities: {}", e)),
            };
            entities.push(entity);
        };
        Ok(entities)
    }
}
