use async_std::stream::StreamExt;
use async_trait::async_trait;
use mongodb::{
    bson::{self, doc},
    options::{ClientOptions, FindOptions},
    Client, Collection, Database,
};
use serde::{de::DeserializeOwned, Serialize};
use std::borrow::Borrow;
pub struct Data {
    pub client: Client,
    pub db: Database,
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
            client,
            db,
        })
    }
    pub fn get_repo<T: Send + Sync + Clone + Serialize + DeserializeOwned + Unpin + 'static>
    (&self, collection: &str, key_field:String)-> Result<Repository<T>, String> {
        let collection = self.db.collection(collection);
        Ok(Repository {
            key_field,
            collection,
        })
    }
}
pub struct Repository<T> 
where T: Send + Sync + Clone + Serialize + DeserializeOwned + Unpin + 'static{
    key_field: String,
    collection: Collection<T>,
}
#[async_trait]
pub trait Crud<T>: Send + Sync {
    async fn create_many(&self, new_entities: Vec<T>) -> Result<Vec<String>, String>;
    async fn create(&self, new_entity: T) -> Result<String, String>;
    async fn get_all(&self, skip:usize, limit:usize) -> Result<Vec<T>, String>;
    async fn get_by_id(&self, id: &str) -> Result<T, String>;
    async fn update_by_id(&self, id: &str, edit_entity: T) -> Result<T, String>;
    async fn delete_by_id(&self, id: &str) -> Result<bool, String>;
    async fn get_by_fields(&self, field: Vec<String>, value: Vec<String>) -> Result<Vec<T>, String>;
    async fn count(&self)->u64;
}

#[async_trait]
impl<T> Crud<T> for Repository<T>
where
    T: Send + Sync + Clone + Serialize + DeserializeOwned + Unpin + 'static,
{
    async fn create_many(&self, entities: Vec<T>) -> Result<Vec<String>, String> {
        let result = match self.collection.insert_many(entities, None).await{
            Ok(result) => result,
            Err(e) => return Err(format!("Error creating entities: {}", e))
        };
        let ids = result.inserted_ids.iter().map(|id| id.1.to_string()).collect();
        Ok(ids)
    }
    async fn create(&self, new_entity: T) -> Result<String, String> {
        let entity = match self
            .collection
            .insert_one(new_entity.borrow(), None)
            .await{
                Ok(entity) => entity,
                Err(e) => return Err(format!("Error creating entity: {}", e))
            };
            Ok(entity.inserted_id.to_string())
        }

    async fn get_by_id(&self, id: &str) -> Result<T, String> {
        let filter = doc! {&self.key_field: id};
        let entity_detail = match self
            .collection
            .find_one(filter, None)
            .await{
                Ok(entity_detail) => match entity_detail{
                    Some(entity_detail) => entity_detail,
                    None => return Err("Entity not found".to_uppercase())
                },
                Err(e) => return Err(format!("Error getting entity: {}", e))
            };
        Ok(entity_detail)
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
            let element = match self.get_by_id(id).await {
                Ok(id) => id,
                Err(e) => return Err(format!("Error updating entity: {}", e))
            };
            Ok(element)
        } else {
            Err("Error updating entity".to_uppercase())
        }
    }

    async fn delete_by_id(&self, id: &str) -> Result<bool, String> {
        let filter = doc! {&self.key_field: id};

        let entity_detail = match self
            .collection
            .delete_one(filter, None)
            .await{
                Ok(entity_detail) => entity_detail,
                Err(e) => return Err(format!("Error deleting entity: {}", e))
            };
        if entity_detail.deleted_count > 0 {
            Ok(true)
        } else {
            Err("Error deleting entity".to_uppercase())
        }
    }

    async fn get_all(&self, skip:usize, limit:usize) -> Result<Vec<T>, String> {
        let find_options = FindOptions::builder()
        .skip(skip as u64)
        .limit(limit as i64)
        .build();
    
        let mut cursors = match self.collection.find(None, find_options).await {
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
    async fn count(&self)->u64{
        match self.collection.count_documents(None, None).await {
            Ok(count) => count,
            _ => return 0,
        }
    }
}
