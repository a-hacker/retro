use juniper::{GraphQLUnion, GraphQLEnum};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use crate::context::Context;
use std::{collections::{HashMap, HashSet}, sync::{Arc, RwLock}};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceMode {
    MEMORY,
    MONGO
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub mode: ServiceMode,
    pub db: Option<DbConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConfig {
    pub host: String,
    pub username: String,
    pub password: String,
    pub database: String,
    pub auth_source: String,
    pub replica_set: Option<String>,
}

impl Into<mongodb::options::ClientOptions> for DbConfig {
    fn into(self) -> mongodb::options::ClientOptions {
        let address = mongodb::options::ServerAddress::parse(&self.host).unwrap();
        let credentials = mongodb::options::Credential::builder()
            .username(self.username)
            .password(self.password)
            .source(self.auth_source)
            .build();
        let tls = mongodb::options::Tls::Enabled(mongodb::options::TlsOptions::default());

        let client_options = mongodb::options::ClientOptions::builder()
            .hosts(vec![address])
            .credential(credentials)
            .tls(tls)
            .repl_set_name(self.replica_set)
            .build();
        client_options
    }
}

impl Default for ServiceConfig {
    fn default() -> Self {
        ServiceConfig {
            mode: ServiceMode::MEMORY,
            db: None,
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub _id: ObjectId,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoginRequest {
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Card {
    pub id: ObjectId,
    pub creator_id: ObjectId,
    pub retro_id: ObjectId,
    pub text: String,
    pub subcards: Vec<Card>,
    pub votes: HashSet<ObjectId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, GraphQLEnum)]
#[graphql(rename_all = "none")]
pub enum RetroStep {
    Writing,
    Grouping,
    Voting,
    Reviewing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetroParticipant {
    pub user: ObjectId,
    pub retro_id: ObjectId,
}

// Represents a Retro
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Retro {
    pub _id: ObjectId,
    pub retro_name: String,
    pub creator_id: ObjectId,
    pub step: RetroStep,
    pub created_at: String, // ISO 8601 format
    pub participants: Vec<RetroParticipant>,
    pub lanes: Vec<Lane>,
}

// Categorized Cards within a Retro
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lane {
    pub id: ObjectId,
    pub title: String,
    pub cards: Vec<Card>,
    pub priority: i32,
}

// Shared State: In-memory storage using Arc and RwLock for thread safety
pub type SharedRetros = Arc<RwLock<HashMap<ObjectId, Retro>>>;
pub type SharedUsers = Arc<RwLock<HashMap<ObjectId, User>>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardAdded {
    pub retro_id: ObjectId,
    pub lane_id: ObjectId,
    pub card: Card,
}

#[juniper::graphql_object(context = Context)]
impl CardAdded {
    async fn retro(&self, context: &Context) -> Retro {
        context.persistence_manager.get_retro(&self.retro_id).await.unwrap()
    }

    async fn lane(&self, context: &Context) -> Lane {
        let retro = context.persistence_manager.get_retro(&self.retro_id).await.unwrap();

        retro.lanes.iter().find(|lane| lane.id == self.lane_id).unwrap().clone()
    }

    fn card(&self) -> &Card {
        &self.card
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserListUpdated {
    pub retro_id: ObjectId,
    pub participants: Vec<RetroParticipant>,
}

#[juniper::graphql_object(context = Context)]
impl UserListUpdated {
    async fn retro(&self, context: &Context) -> Retro {
        context.persistence_manager.get_retro(&self.retro_id).await.unwrap()
    }

    fn participants(&self) -> &Vec<RetroParticipant> {
        &self.participants
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepUpdated {
    pub retro_id: ObjectId,
    pub step: RetroStep,
}

#[juniper::graphql_object(context = Context)]
impl StepUpdated {
    async fn retro(&self, context: &Context) -> Retro {
        context.persistence_manager.get_retro(&self.retro_id).await.unwrap()
    }

    fn step(&self) -> &RetroStep {
        &self.step
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, GraphQLUnion)]
#[graphql(context = Context)]
pub enum SubscriptionUpdate {
    CardAdded(CardAdded),
    UserListUpdated(UserListUpdated),
    StepUpdated(StepUpdated)
}

impl SubscriptionUpdate {
    pub fn create_card_added(retro_id: ObjectId, lane_id: ObjectId, card: Card) -> Self {
        let card_added = CardAdded {
            retro_id, lane_id, card
        };

        Self::CardAdded(card_added)
    }

    pub fn create_user_list_update(retro_id: ObjectId, participants: Vec<RetroParticipant>) -> Self {
        let user_list_update = UserListUpdated {
            retro_id, participants
        };

        Self::UserListUpdated(user_list_update)
    }

    pub fn create_step_update(retro_id: ObjectId, step: RetroStep) -> Self {
        let step_update = StepUpdated {
            retro_id, step
        };
        Self::StepUpdated(step_update)
    }
}