use std::collections::HashMap;

use async_trait::async_trait;
use futures::stream::StreamExt;
use mongodb::{bson::{self, doc, Document}, Collection};
use uuid::Uuid;

use crate::models::{Retro, ServiceConfig, SharedRetros, SharedUsers, User};

#[async_trait]
trait PersistenceHandler: Clone {
    async fn get_retro(&self, retro_id: &Uuid) -> Result<Retro, String>;
    async fn get_retros(&self) -> Result<Vec<Retro>, String>;
    async fn get_user(&self, user_id: &Uuid) -> Result<User, String>;
    async fn get_users(&self) -> Result<Vec<User>, String>;
    async fn create_user(&self, user: User) -> Result<User, String>;
    async fn create_retro(&self, retro: Retro) -> Result<Retro, String>;
    async fn update_retro(&self, retro: Retro) -> Result<Retro, String>;
    async fn update_user(&self, user: User) -> Result<User, String>;
}


#[derive(Clone)]
pub struct MemoryHandler {
    retros: SharedRetros,
    users: SharedUsers,
}

impl MemoryHandler {
    pub fn new(retros: SharedRetros, users: SharedUsers) -> MemoryHandler {
        MemoryHandler {
            retros,
            users,
        }
    }
}

#[async_trait]
impl PersistenceHandler for MemoryHandler {
    async fn get_retro(&self, retro_id: &Uuid) -> Result<Retro, String> {
        let retros = self.retros.read().unwrap();
        match retros.get(retro_id) {
            Some(retro) => Ok(retro.clone()),
            None => Err("Retro not found".to_string()),
        }
    }

    async fn get_retros(&self) -> Result<Vec<Retro>, String> {
        let retros = self.retros.read().unwrap();
        let retros: Vec<Retro> = retros.values().cloned().collect();
        Ok(retros)
    }

    async fn get_user(&self, user_id: &Uuid) -> Result<User, String> {
        let users = self.users.read().unwrap();
        match users.get(user_id) {
            Some(user) => Ok(user.clone()),
            None => Err("User not found".to_string()),
        }
    }

    async fn get_users(&self) -> Result<Vec<User>, String> {
        let users = self.users.read().unwrap();
        let users: Vec<User> = users.values().cloned().collect();
        Ok(users)
    }

    async fn create_user(&self, user: User) -> Result<User, String> {
        let mut users = self.users.write().unwrap();
        users.insert(user.id, user.clone());
        Ok(user)
    }

    async fn create_retro(&self, retro: Retro) -> Result<Retro, String> {
        let mut retros = self.retros.write().unwrap();
        retros.insert(retro.id, retro.clone());
        Ok(retro)
    }

    async fn update_retro(&self, retro: Retro) -> Result<Retro, String> {
        let mut retros = self.retros.write().unwrap();
        retros.insert(retro.id, retro.clone());
        Ok(retro)
    }

    async fn update_user(&self, user: User) -> Result<User, String> {
        let mut users = self.users.write().unwrap();
        users.insert(user.id, user.clone());
        Ok(user)
    }
}

#[derive(Clone)]
pub struct MongoHandler {
    client: mongodb::Client,
    db: mongodb::Database,
}

impl MongoHandler {
    pub async fn new() -> MongoHandler {
        let client = mongodb::Client::with_uri_str("mongodb://localhost:27017").await.unwrap();
        let db = client.database("retro");
        MongoHandler { client, db }
    }
}

#[async_trait]
impl PersistenceHandler for MongoHandler {
    async fn get_retro(&self, retro_id: &Uuid) -> Result<Retro, String> {
        let retros = self.db.collection("retros");
        let filter = doc! { "id": retro_id.to_string() };
        let result = retros.find_one(filter).await.unwrap();
        match result {
            Some(doc) => {
                let retro: Retro = bson::from_bson(bson::Bson::Document(doc)).unwrap();
                Ok(retro)
            }
            None => Err("Retro not found".to_string()),
        }
    }

    async fn get_retros(&self) -> Result<Vec<Retro>, String> {
        let retros = self.db.collection("retros");
        let mut cursor = retros.find(doc! {}).await.unwrap();
        let mut result = vec![];
        while let Some(doc) = cursor.next().await {
            let retro: Retro = bson::from_bson(bson::Bson::Document(doc.unwrap())).unwrap();
            result.push(retro);
        }
        Ok(result)
    }

    async fn get_user(&self, user_id: &Uuid) -> Result<User, String> {
        let users = self.db.collection("users");
        let filter = doc! { "id": user_id.to_string() };
        let result = users.find_one(filter).await.unwrap();
        match result {
            Some(doc) => {
                let user: User = bson::from_bson(bson::Bson::Document(doc)).unwrap();
                Ok(user)
            }
            None => Err("User not found".to_string()),
        }
    }

    async fn get_users(&self) -> Result<Vec<User>, String> {
        let users = self.db.collection("users");
        let mut cursor = users.find(doc! {}).await.unwrap();
        let mut result = vec![];
        while let Some(doc) = cursor.next().await {
            let user: User = bson::from_bson(bson::Bson::Document(doc.unwrap())).unwrap();
            result.push(user);
        }
        Ok(result)
    }

    async fn create_user(&self, user: User) -> Result<User, String> {
        println!("Creating user: {:?}", user);
        let users = self.db.collection("users");
        let doc = bson::to_document(&user).unwrap();
        users.insert_one(doc).await.unwrap();
        Ok(user)
    }

    async fn create_retro(&self, retro: Retro) -> Result<Retro, String> {
        let retros = self.db.collection("retros");
        let doc = bson::to_document(&retro).unwrap();
        retros.insert_one(doc).await.unwrap();
        Ok(retro)
    }

    async fn update_retro(&self, retro: Retro) -> Result<Retro, String> {
        let retros: Collection<Document> = self.db.collection("retros");
        let filter = doc! { "id": retro.id.to_string() };
        let doc = bson::to_document(&retro).unwrap();
        retros.update_one(filter, doc).await.unwrap();
        Ok(retro)
    }

    async fn update_user(&self, user: User) -> Result<User, String> {
        let users: Collection<Document> = self.db.collection("users");
        let filter = doc! { "id": user.id.to_string() };
        let doc = bson::to_document(&user).unwrap();
        users.update_one(filter, doc).await.unwrap();
        Ok(user)
    }
}

#[derive(Clone)]
pub enum PersistenceManager {
    Memory(MemoryHandler),
    Mongo(MongoHandler),
}

impl PersistenceManager {
    pub fn new_memory(config: &ServiceConfig, retros: SharedRetros, users: SharedUsers) -> PersistenceManager {
        let handler = MemoryHandler::new(retros, users);
        PersistenceManager::Memory(handler)
    }

    pub async fn new_mongo(config: &ServiceConfig) -> PersistenceManager {
        let handler = MongoHandler::new().await;
        PersistenceManager::Mongo(handler)
    }

    pub async fn get_retro(&self, retro_id: &Uuid) -> Result<Retro, String> {
        match self {
            PersistenceManager::Memory(handler) => handler.get_retro(retro_id).await,
            PersistenceManager::Mongo(handler) => handler.get_retro(retro_id).await,
        }
    }

    pub async fn get_retros(&self) -> Result<Vec<Retro>, String> {
        match self {
            PersistenceManager::Memory(handler) => handler.get_retros().await,
            PersistenceManager::Mongo(handler) => handler.get_retros().await,
        }
    }

    pub async fn get_user(&self, user_id: &Uuid) -> Result<User, String> {
        match self {
            PersistenceManager::Memory(handler) => handler.get_user(user_id).await,
            PersistenceManager::Mongo(handler) => handler.get_user(user_id).await,
        }
    }

    pub async fn get_users(&self) -> Result<Vec<User>, String> {
        match self {
            PersistenceManager::Memory(handler) => handler.get_users().await,
            PersistenceManager::Mongo(handler) => handler.get_users().await,
        }
    }

    pub async fn create_user(&self, user: User) -> Result<User, String> {
        match self {
            PersistenceManager::Memory(handler) => handler.create_user(user).await,
            PersistenceManager::Mongo(handler) => handler.create_user(user).await,
        }
    }

    pub async fn create_retro(&self, retro: Retro) -> Result<Retro, String> {
        match self {
            PersistenceManager::Memory(handler) => handler.create_retro(retro).await,
            PersistenceManager::Mongo(handler) => handler.create_retro(retro).await,
        }
    }

    pub async fn update_retro(&self, retro: Retro) -> Result<Retro, String> {
        match self {
            PersistenceManager::Memory(handler) => handler.update_retro(retro).await,
            PersistenceManager::Mongo(handler) => handler.update_retro(retro).await,
        }
    }

    pub async fn update_user(&self, user: User) -> Result<User, String> {
        match self {
            PersistenceManager::Memory(handler) => handler.update_user(user).await,
            PersistenceManager::Mongo(handler) => handler.update_user(user).await,
        }
    }
}
