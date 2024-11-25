// src/schema.rs

use juniper::{EmptySubscription, RootNode};
use crate::models::{Retro, Card, Cards};
use crate::context::Context;
use chrono::prelude::*;

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
        new_retro
    }

    // Add a user to a retro
    fn add_user(context: &Context, retro_id: i32, username: String) -> Vec<String> {
        let mut retros = context.retros.write().unwrap();
        if let Some(retro) = retros.iter_mut().find(|retro| retro.id == retro_id) {
            if !retro.users.contains(&username) {
                retro.users.push(username.clone());
            }
            // Ideally, emit an event for real-time updates here
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
            // Ideally, emit an event for real-time updates here
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

            // Ideally, emit an event for real-time updates here
            Some(new_card)
        } else {
            None
        }
    }
}

// Define the schema
pub type Schema = RootNode<'static, QueryRoot, MutationRoot, EmptySubscription<Context>>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot, MutationRoot, EmptySubscription::new())
}
