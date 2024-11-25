// src/schema.rs

use futures::TryStreamExt;
use juniper::{RootNode, ToInputValue, graphql_subscription};
use crate::models::{Retro, SharedRetros, Card, Cards, SubscriptionUpdate, CardAdded, UserListUpdated};
use crate::context::Context;
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use chrono::prelude::*;
use tokio::sync::broadcast;
use tokio_stream::StreamExt;

// GraphQL representation of a Card
#[juniper::graphql_object]
impl Card {
    fn id(&self) -> i32 {
        self.id
    }

    fn text(&self) -> &str {
        &self.text
    }
}

// GraphQL representation of Cards
#[juniper::graphql_object]
impl Cards {
    fn good(&self) -> &Vec<Card> {
        &self.good
    }

    fn bad(&self) -> &Vec<Card> {
        &self.bad
    }

    fn needs_improvement(&self) -> &Vec<Card> {
        &self.needs_improvement
    }
}

// GraphQL representation of a Retro
#[juniper::graphql_object(context = Context)]
impl Retro {
    fn id(&self) -> i32 {
        self.id
    }

    fn retro_name(&self) -> &str {
        &self.retro_name
    }

    fn creator_name(&self) -> &str {
        &self.creator_name
    }

    fn created_at(&self) -> &str {
        &self.created_at
    }

    fn users(&self) -> &Vec<String> {
        &self.users
    }

    fn cards(&self) -> &Cards {
        &self.cards
    }
}

// Input types for mutations
#[derive(juniper::GraphQLEnum)]
pub enum Category {
    Good,
    Bad,
    NeedsImprovement,
}

#[derive(juniper::GraphQLInputObject)]
pub struct CreateRetroInput {
    pub retro_name: String,
    pub creator_name: String,
}

#[derive(juniper::GraphQLInputObject)]
pub struct AddCardInput {
    pub retro_id: i32,
    pub category: Category,
    pub text: String,
}

#[derive(juniper::GraphQLInputObject)]
pub struct LeaveRetroInput {
    pub retro_id: i32,
    pub username: String,
}

// Subscription root
type SubStream = Pin<Box<dyn futures::Stream<Item = SubscriptionUpdate> + Send>>;

pub struct SubscriptionRoot;

#[graphql_subscription(context = Context)]
impl SubscriptionRoot {
    // Subscription for added cards
    async fn card_added(context: &Context, retro_id: i32) -> SubStream {
        let rx = context.card_addition_sender.subscribe();

        let stream = tokio_stream::wrappers::BroadcastStream::new(rx)
            .filter_map(move |result| {
                match result {
                    Ok(update) => {
                        match update.clone() {
                            SubscriptionUpdate::CardAdded (card) if card.retro_id == retro_id => Some(update),
                            _ => None,
                        }
                    }
                    Err(_) => None,
                }
            });

        Box::pin(stream)
    }

    // Subscription for user list updates
    async fn user_list_updated(context: &Context, retro_id: i32) -> SubStream {
        let rx = context.user_update_sender.subscribe();

        let stream = tokio_stream::wrappers::BroadcastStream::new(rx)
            .filter_map(move |result| {
                match result {
                    Ok(update) => {
                        match update.clone() {
                            SubscriptionUpdate::UserListUpdated(u) if u.retro_id == retro_id => Some(update),
                            _ => None,
                        }
                    }
                    Err(_) => None,
                }
            });

        Box::pin(stream)
    }
}

// Root Query object
pub struct QueryRoot;

#[juniper::graphql_object(context = Context)]
impl QueryRoot {
    // Fetch all retrospectives
    fn all_retros(context: &Context) -> Vec<Retro> {
        let retros = context.retros.read().unwrap();
        retros.clone()
    }

    // Fetch a specific retro by ID
    fn retro_by_id(context: &Context, id: i32) -> Option<Retro> {
        let retros = context.retros.read().unwrap();
        retros.iter().find(|retro| retro.id == id).cloned()
    }
}

// Root Mutation object
pub struct MutationRoot;

#[juniper::graphql_object(context = Context)]
impl MutationRoot {
    // Create a new retro
    fn create_retro(context: &Context, input: CreateRetroInput) -> Retro {
        let mut retros = context.retros.write().unwrap();
        let new_id = (retros.len() + 1) as i32;
        let created_at = Utc::now().to_rfc3339();

        let new_retro = Retro {
            id: new_id,
            retro_name: input.retro_name,
            creator_name: input.creator_name,
            created_at,
            users: vec![],
            cards: Cards {
                good: vec![],
                bad: vec![],
                needs_improvement: vec![],
            },
        };

        retros.push(new_retro.clone());

        // Broadcast user list update for the new retro (initially empty)
        let _ = context.user_update_sender.send(SubscriptionUpdate::UserListUpdated ( UserListUpdated {
            retro_id: new_retro.id,
            users: new_retro.users.clone(),
        }));

        new_retro
    }

    // Add a user to a retro
    fn add_user(context: &Context, retro_id: i32, username: String) -> Vec<String> {
        let mut retros = context.retros.write().unwrap();
        if let Some(retro) = retros.iter_mut().find(|retro| retro.id == retro_id) {
            if !retro.users.contains(&username) {
                retro.users.push(username.clone());

                // Broadcast user list update
                let _ = context.user_update_sender.send(SubscriptionUpdate::create_user_list_update(
                    retro_id,
                    retro.users.clone(),
                ));
            }
            retro.users.clone()
        } else {
            vec![]
        }
    }

    // Remove a user from a retro
    fn leave_retro(context: &Context, input: LeaveRetroInput) -> Vec<String> {
        let mut retros = context.retros.write().unwrap();
        if let Some(retro) = retros.iter_mut().find(|retro| retro.id == input.retro_id) {
            retro.users.retain(|user| user != &input.username);

            // Broadcast user list update
            let _ = context.user_update_sender.send(SubscriptionUpdate::create_user_list_update(
                retro.id,
                retro.users.clone(),
            ));

            retro.users.clone()
        } else {
            vec![]
        }
    }

    // Add a card to a retro
    fn add_card(context: &Context, input: AddCardInput) -> Option<Card> {
        let mut retros = context.retros.write().unwrap();
        if let Some(retro) = retros.iter_mut().find(|retro| retro.id == input.retro_id) {
            let new_card_id = retro.cards.good.len() + retro.cards.bad.len() + retro.cards.needs_improvement.len() + 1;
            let new_card = Card {
                id: new_card_id as i32,
                text: input.text.clone(),
            };

            match input.category {
                Category::Good => retro.cards.good.push(new_card.clone()),
                Category::Bad => retro.cards.bad.push(new_card.clone()),
                Category::NeedsImprovement => retro.cards.needs_improvement.push(new_card.clone()),
            }

            // Broadcast card addition
            let category_str = match input.category {
                Category::Good => "GOOD",
                Category::Bad => "BAD",
                Category::NeedsImprovement => "NEEDS_IMPROVEMENT",
            };

            let _ = context.card_addition_sender.send(SubscriptionUpdate::create_card_added(
                retro.id,
                category_str.to_string(),
                new_card.clone(),
            ));

            Some(new_card)
        } else {
            None
        }
    }
}

// Define the schema
pub type Schema = RootNode<'static, QueryRoot, MutationRoot, SubscriptionRoot>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot, MutationRoot, SubscriptionRoot)
}
