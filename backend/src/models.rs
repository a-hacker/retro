use juniper::{GraphQLObject, GraphQLUnion, GraphQLEnum};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::{Arc, RwLock}};
use uuid::Uuid;


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, GraphQLObject)]
pub struct User {
    pub id: Uuid,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: Uuid,
    pub creator_id: Uuid,
    pub text: String,
    pub votes: i32,
    pub subcards: Vec<Card>
}

#[derive(Debug, Clone, Serialize, Deserialize, GraphQLEnum)]
pub enum RetroStep {
    Writing,
    Grouping,
    Voting,
    Reviewing,
}

// Represents a Retro
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Retro {
    pub id: Uuid,
    pub retro_name: String,
    pub creator_id: Uuid,
    pub step: RetroStep,
    pub created_at: String, // ISO 8601 format
    pub users: Vec<Uuid>,
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
pub type SharedRetros = Arc<RwLock<Vec<Retro>>>;
pub type SharedUsers = Arc<RwLock<HashMap<Uuid, User>>>;

#[derive(Debug, Clone, Serialize, Deserialize, GraphQLObject)]
pub struct CardAdded {
    pub retro_id: Uuid,
    pub lane_id: Uuid,
    pub card: Card,
}

#[derive(Debug, Clone, Serialize, Deserialize, GraphQLObject)]
pub struct UserListUpdated {
    pub retro_id: Uuid,
    pub users: Vec<Uuid>,
}


#[derive(Debug, Clone, Serialize, Deserialize, GraphQLUnion)]
pub enum SubscriptionUpdate {
    CardAdded(CardAdded),
    UserListUpdated(UserListUpdated)
}

impl SubscriptionUpdate {
    pub fn create_card_added(retro_id: Uuid, lane_id: Uuid, card: Card) -> Self {
        let card_added = CardAdded {
            retro_id, lane_id, card
        };

        Self::CardAdded(card_added)
    }

    pub fn create_user_list_update(retro_id: Uuid, users: Vec<Uuid>) -> Self {
        let user_list_update = UserListUpdated {
            retro_id, users
        };

        Self::UserListUpdated(user_list_update)
    }
}