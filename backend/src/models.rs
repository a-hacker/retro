use juniper::{GraphQLObject, GraphQLUnion};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

// Represents a Card in a Retro
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: i32,
    pub text: String,
}

// Represents a Retro
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Retro {
    pub id: i32,
    pub retro_name: String,
    pub creator_name: String,
    pub created_at: String, // ISO 8601 format
    pub users: Vec<String>,
    pub cards: Cards,
}

// Categorized Cards within a Retro
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cards {
    pub good: Vec<Card>,
    pub bad: Vec<Card>,
    pub needs_improvement: Vec<Card>,
}

// Shared State: In-memory storage using Arc and RwLock for thread safety
pub type SharedRetros = Arc<RwLock<Vec<Retro>>>;

#[derive(Debug, Clone, Serialize, Deserialize, GraphQLObject)]
pub struct CardAdded {
    pub retro_id: i32,
    pub category: String,
    pub card: Card,
}

#[derive(Debug, Clone, Serialize, Deserialize, GraphQLObject)]
pub struct UserListUpdated {
    pub retro_id: i32,
    pub users: Vec<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize, GraphQLUnion)]
pub enum SubscriptionUpdate {
    CardAdded(CardAdded),
    UserListUpdated(UserListUpdated)
}

impl SubscriptionUpdate {
    pub fn create_card_added(retro_id: i32, category: String, card: Card) -> Self {
        let card_added = CardAdded {
            retro_id, category, card
        };

        Self::CardAdded(card_added)
    }

    pub fn create_user_list_update(retro_id: i32, users: Vec<String>) -> Self {
        let user_list_update = UserListUpdated {
            retro_id, users
        };

        Self::UserListUpdated(user_list_update)
    }
}