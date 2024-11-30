use juniper::{GraphQLObject, GraphQLUnion, GraphQLEnum};
use serde::{Deserialize, Serialize};
use crate::context::Context;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardAdded {
    pub retro_id: Uuid,
    pub lane_id: Uuid,
    pub card: Card,
}

#[juniper::graphql_object(context = Context)]
impl CardAdded {
    fn retro(&self, context: &Context) -> Retro {
        context.retros.read().unwrap().iter()
            .filter(|retro| retro.id == self.retro_id).next()
            .unwrap().clone()
    }

    fn lane(&self, context: &Context) -> Lane {
        let retro = context.retros.read().unwrap().iter()
        .filter(|retro| retro.id == self.retro_id).next().unwrap().clone();

        retro.lanes.iter().filter(|lane| lane.id == self.lane_id).next().unwrap().clone()
    }

    fn card(&self) -> &Card {
        &self.card
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserListUpdated {
    pub retro_id: Uuid,
    pub users: Vec<Uuid>,
}

#[juniper::graphql_object(context = Context)]
impl UserListUpdated {
    fn retro(&self, context: &Context) -> Retro {
        context.retros.read().unwrap().iter()
            .filter(|retro| retro.id == self.retro_id).next()
            .unwrap().clone()
    }

    fn users(&self, context: &Context) -> Vec<User> {
        let users = context.users.read().unwrap();
        self.users.iter().map(|uid| users.get(uid).unwrap().clone()).collect()
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, GraphQLUnion)]
#[graphql(context = Context)]
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