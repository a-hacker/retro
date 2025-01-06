use juniper::{GraphQLObject, GraphQLUnion, GraphQLEnum};
use serde::{Deserialize, Serialize};
use crate::context::Context;
use std::{collections::{HashMap, HashSet}, sync::{Arc, RwLock}};
use uuid::Uuid;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceMode {
    MEMORY,
    MONGO
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub mode: ServiceMode,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        ServiceConfig {
            mode: ServiceMode::MEMORY
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, GraphQLObject)]
pub struct User {
    pub id: Uuid,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Eq)]
pub struct Card {
    pub id: Uuid,
    pub creator_id: Uuid,
    pub text: String,
    pub subcards: Vec<Card>
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
    pub user: Uuid,
    pub retro_id: Uuid,
    pub votes: HashSet<Uuid>,
}

// Represents a Retro
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Retro {
    pub id: Uuid,
    pub retro_name: String,
    pub creator_id: Uuid,
    pub step: RetroStep,
    pub created_at: String, // ISO 8601 format
    pub participants: Vec<RetroParticipant>,
    pub lanes: Vec<Lane>,
}

// Categorized Cards within a Retro
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lane {
    pub id: Uuid,
    pub title: String,
    pub cards: Vec<Card>,
    pub priority: i32,
}

// Shared State: In-memory storage using Arc and RwLock for thread safety
pub type SharedRetros = Arc<RwLock<HashMap<Uuid, Retro>>>;
pub type SharedUsers = Arc<RwLock<HashMap<Uuid, User>>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardAdded {
    pub retro_id: Uuid,
    pub lane_id: Uuid,
    pub card: Card,
}

#[juniper::graphql_object(context = Context)]
impl CardAdded {
    async fn retro(&self, context: &Context) -> Retro {
        context.persistence_manager.get_retro(&self.retro_id).await.unwrap()
    }

    async fn lane(&self, context: &Context) -> Lane {
        let retro = context.persistence_manager.get_retro(&self.retro_id).await.unwrap();

        retro.lanes.iter().filter(|lane| lane.id == self.lane_id).next().unwrap().clone()
    }

    fn card(&self) -> &Card {
        &self.card
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserListUpdated {
    pub retro_id: Uuid,
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
    pub retro_id: Uuid,
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
    pub fn create_card_added(retro_id: Uuid, lane_id: Uuid, card: Card) -> Self {
        let card_added = CardAdded {
            retro_id, lane_id, card
        };

        Self::CardAdded(card_added)
    }

    pub fn create_user_list_update(retro_id: Uuid, participants: Vec<RetroParticipant>) -> Self {
        let user_list_update = UserListUpdated {
            retro_id, participants
        };

        Self::UserListUpdated(user_list_update)
    }

    pub fn create_step_update(retro_id: Uuid, step: RetroStep) -> Self {
        let step_update = StepUpdated {
            retro_id, step
        };
        Self::StepUpdated(step_update)
    }
}